use std::io;
use std::io::{BufRead, BufReader, Lines};
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

//use chrono::Duration;
use core::time::Duration;

use cpal::{StreamInstant, OutputStreamTimestamp};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[cfg(feature = "ogg-playback")]
use lewton::inside_ogg::OggStreamReader;
use num_traits::clamp;

use crate::engine_constants::EngineConstants;
use crate::framework::context::Context;
use crate::framework::error::{GameError, GameResult};
use crate::framework::error::GameError::{AudioError, InvalidValue};
use crate::framework::filesystem;
use crate::framework::filesystem::File;
use crate::game::settings::Settings;
//use OGG if defined in the configuration
#[cfg(feature = "ogg-playback")]
use crate::sound::ogg_playback::{OggPlaybackEngine, SavedOggPlaybackState};
//this will be used either way
use crate::sound::org_playback::{OrgPlaybackEngine, SavedOrganyaPlaybackState};
use crate::sound::organya::Song;
use crate::sound::pixtone::{PixToneParameters, PixTonePlayback};
use crate::sound::wave_bank::SoundBank;

use crate::common::RunGameTime;

use std::time::SystemTime;
use self::organya::Timing;
use self::wav::WavSample;

mod fir;
#[cfg(feature = "ogg-playback")]
mod ogg_playback;
mod org_playback;
mod organya;
pub mod pixtone;
mod pixtone_sfx;
mod stuff;
mod wav;
mod wave_bank;

//structure contains current tracker state. it will be inside SoundManager
pub struct CurrentOrgState
{
    pub lengths: [u8; 8], //lengths of the notes currently being played

    pub changed: [bool; 8], //has the note changed on this tick (only used by music portion)

    pub offsets: [u16; 8], //how many ticks have gone by since the last note start (only used by game portion)

    pub swaps: [usize; 8], //what is "swap"? (we may not need this...)

    pub keys: [u8; 8], //notes currently being played ("keys" being pressed)

    pub drums: [u8; 8], //drums currently being played

    pub timestamp: SystemTime, //when was the packet made?

}
impl CurrentOrgState
{
    pub fn new() -> CurrentOrgState
    {
        return CurrentOrgState
        {
            lengths: [0; 8],
            changed: [false; 8],
            offsets: [0; 8],
            swaps: [0; 8],
            keys: [255; 8],
            drums: [0; 8],
            timestamp: SystemTime::now(), //SystemTime::UNIX_EPOCH,
        };
    } 
  

    pub fn copy_from(&mut self, other: &Self)
    {
        self.keys.clone_from(&other.keys);
        self.lengths.clone_from(&other.lengths);
        self.swaps.clone_from(&other.swaps);
        self.drums.clone_from(&other.drums);
    }

}

//this is what's exposed to the main game code.
//if we want to make ORG state avalable to the AI, we need to put our stuff in here
pub struct SoundManager {
    soundbank: Option<SoundBank>,
    tx: Sender<PlaybackMessage>, //pushes message to audio backend thread
    state_rx: Receiver<CurrentOrgState>, //gets telemetry from the audio backend thread
    prev_song_id: usize,
    current_song_id: usize,
    no_audio: bool,
    load_failed: bool,
    stream: Option<cpal::Stream>,
    note_state: CurrentOrgState, //back-telemetry from the commander
    //duplicate vars for the commander state
    prev_commander_id: usize,
    current_commander_id: usize,
    commander_tempo: u16,

    //lock/unlock sound manager states
    play_lock: bool, //true if the music was paused from TSC instead of a defocus
    timer_lock_soft: bool, //true if the music was locked from a soft pause
    timer_lock_hard: bool, //true if the music was locked from a hard pause
    music_paused_at: SystemTime, //time when the music was paused (used for delayed resumes)

    //time relative to music player's state
    pub music_time: RunGameTime,

    //string paths for songs
    current_song_path: String,
    current_commander_path: String,
}

pub enum SongFormat {
    Organya,
    #[cfg(feature = "ogg-playback")]
    OggSinglePart,
    #[cfg(feature = "ogg-playback")]
    OggMultiPart,
}

#[derive(Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InterpolationMode {
    Nearest,
    Linear,
    Cosine,
    Cubic,
    Polyphase,
}

impl SoundManager {
    pub fn new(ctx: &mut Context) -> GameResult<SoundManager> {


        //create rx and tx handles for multithread communication (for sending commands to the audio backend)
        let (tx, rx): (Sender<PlaybackMessage>, Receiver<PlaybackMessage>) = mpsc::channel();

        //create telemetry handles
        let (state_tx, state_rx): (Sender<CurrentOrgState>, Receiver<CurrentOrgState>) = mpsc::channel();

        //in headless mode, audio rendering is ignored
        if ctx.headless {
            log::info!("Running in headless mode, skipping initialization.");

            return Ok(SoundManager {
                soundbank: None,
                tx: tx.clone(),
                state_rx: state_rx,
                prev_song_id: 0,
                current_song_id: 0,
                no_audio: true,
                load_failed: false,
                stream: None,
                note_state: CurrentOrgState::new(), 
                prev_commander_id: 0,
                current_commander_id: 0,
                commander_tempo: 0,
                play_lock: false,
                current_commander_path: String::new(),
                current_song_path: String::new(),

                music_time: RunGameTime::new(),
                timer_lock_soft: false,
                timer_lock_hard: false,
                music_paused_at: SystemTime::UNIX_EPOCH,
            });
        }

        //let bnk = wave_bank::SoundBank::load_from(filesystem::open(ctx, "/builtin/organya-wavetable-doukutsu.bin")?)?;
        let bnk = wave_bank::SoundBank::load_from(filesystem::open(ctx, "/builtin/organya-wavetable-orgmaker.bin")?)?;
        Ok(SoundManager::bootstrap(&bnk, tx, rx, state_tx, state_rx)?)
    }

    fn bootstrap(
        soundbank: &SoundBank,
        //we keep one and give the other to the second thread... (keep tx, give rx)
        tx: Sender<PlaybackMessage>,
        rx: Receiver<PlaybackMessage>,

        //new, ditto (keep rx, give tx)
        state_tx: Sender<CurrentOrgState>,
        state_rx: Receiver<CurrentOrgState>
    ) -> GameResult<SoundManager> {

        let mut sound_manager = SoundManager {
            soundbank: Some(soundbank.to_owned()),
            tx,
            state_rx: state_rx,
            prev_song_id: 0,
            current_song_id: 0,
            no_audio: false,
            load_failed: false,
            stream: None,
            note_state: CurrentOrgState::new(), 

            prev_commander_id: 0,
            current_commander_id: 0,
            commander_tempo: 0,

            music_time: RunGameTime::new(),
            play_lock: false,
            timer_lock_soft: false,
            timer_lock_hard: false,
            music_paused_at: SystemTime::UNIX_EPOCH,

            current_commander_path: String::new(),
            current_song_path: String::new(),
        };

        let host = cpal::default_host();

        let device_result =
            host.default_output_device().ok_or_else(|| AudioError("Error initializing audio device.".to_owned()));

        if device_result.is_err() {
            log::error!("{}", device_result.err().unwrap().to_string());
            sound_manager.load_failed = true;
            return Ok(sound_manager);
        }

        let device = device_result.unwrap();

        let config_result = device.default_output_config();

        if config_result.is_err() {
            log::error!("{}", config_result.err().unwrap().to_string());
            sound_manager.load_failed = true;
            return Ok(sound_manager);
        }

        let config = config_result.unwrap();

        //start async task to render audio, depending on sample format
        let res = match config.sample_format() {
            cpal::SampleFormat::I8 => run::<i8>(rx, state_tx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::I16 => run::<i16>(rx, state_tx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::I32 => run::<i32>(rx, state_tx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::I64 => run::<i64>(rx, state_tx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::U8 => run::<u8>(rx, state_tx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::U16 => run::<u16>(rx, state_tx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::U32 => run::<u32>(rx, state_tx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::U64 => run::<u64>(rx, state_tx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::F32 => run::<f32>(rx, state_tx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::F64 => run::<f64>(rx, state_tx, soundbank.to_owned(), device, config.into()),
            _ => Err(AudioError("Unsupported sample format.".to_owned())),
        };



        if let Err(res) = &res {
            log::error!("Error initializing audio: {}", res);
        }

        sound_manager.stream = res.ok();
        Ok(sound_manager)
    }

    pub fn reload(&mut self) -> GameResult<()> {
        if self.no_audio {
            log::info!("Skipping sound manager reload because audio is not enabled.");
            return Ok(());
        }

        log::info!("Reloading sound manager.");

        let (tx, rx): (Sender<PlaybackMessage>, Receiver<PlaybackMessage>) = mpsc::channel();
        
        //create telemetry handles
        let (state_tx, state_rx): (Sender<CurrentOrgState>, Receiver<CurrentOrgState>) = mpsc::channel();


        let soundbank = self.soundbank.take().unwrap();
        *self = SoundManager::bootstrap(&soundbank, tx, rx, state_tx, state_rx)?;

        Ok(())
    }

    //send raw commands to the player
    fn send(&mut self, message: PlaybackMessage) -> GameResult<()> {
        if self.no_audio {
            return Ok(());
        }

        if self.tx.send(message).is_err() {
            if !self.load_failed {
                log::error!("Error sending message to audio thread. Press Ctrl + F3 to reload sound manager.");
                self.reload()?;
            }
        }

        Ok(())
    }


    //feed 'true' if the command is from a soft resume
    //moved to general wrapper function below
    pub fn try_resume_music_timer(&mut self, is_soft: bool)
    {
        //change the state of the proper variable
        if is_soft {self.timer_lock_soft = false}
        else {self.timer_lock_hard = false}
        if !self.timer_lock_soft && !self.timer_lock_hard
        {
            self.music_time.resume();
        }
    }

    //feed 'true' if the command is from a soft lock
    pub fn pause_music_timer(&mut self, is_soft: bool, pause: bool)
    {
        //true if timer was running
        let prev_state = !self.timer_lock_soft && !self.timer_lock_hard;
        
        //run pasuse operation
        if pause
        {
            //change the state of the proper variable
            if is_soft
            {
                //check if we are already soft-paused
                if self.timer_lock_soft { return }

                self.send(PlaybackMessage::SaveState).unwrap();
                self.send(PlaybackMessage::SaveCommanderState).unwrap();
                self.send(PlaybackMessage::Stop).unwrap();
                self.send(PlaybackMessage::StopCommander).unwrap();

                self.timer_lock_soft = true;
                self.music_time.pause();
            }
            else
            {
                //check if we are already soft-paused
                if self.timer_lock_hard { return }

                if let Some(stream) = &mut self.stream {
                    let _ = stream.pause();
                    self.timer_lock_hard = true;
                    self.music_time.pause();
                }
            }

            //log the moment when the music was stopped
            //should only run if the total state was changed
            // if prev_state == true && //last state is true
            // ! (self.timer_lock_soft || self.timer_lock_hard) //current state is false
            // {
            //     self.music_paused_at = SystemTime::now();
            // }

            if prev_state //last state is true
            {
                if self.timer_lock_soft || self.timer_lock_hard
                {
                    self.music_paused_at = SystemTime::now();
                }
            }


        }
        //run resume operation
        else
        {
            if is_soft
            {
                //check for existing operation
                if !self.timer_lock_soft {return}

                self.timer_lock_soft = false;
            
                self.send(PlaybackMessage::RestoreState(false)).unwrap();
                self.send(PlaybackMessage::RestoreCommanderState(false)).unwrap();                
            
            }
            else
            {
                //check for existing operation
                if !self.timer_lock_hard {return}

                if let Some(stream) = &mut self.stream {
                    let _ = stream.play();
                    self.timer_lock_hard = false;
                }
            }

            //resume timer if both hard and soft locks are removed
            if !self.timer_lock_soft && !self.timer_lock_hard
            {
                self.music_time.resume();

                let time_paused = SystemTime::now().duration_since(self.music_paused_at).unwrap();
                self.send(PlaybackMessage::SetFreezeSongOffset(time_paused)).unwrap();
                self.send(PlaybackMessage::SetFreezeTrackerOffset(time_paused)).unwrap();
            }
        }


    }


    //pause with locking, so TSC events keep it paused even if screen events try to resume it
    pub fn pause_lock(&mut self) {
        self.play_lock = true;
        self.pause_music_timer(false, true);
    }
    pub fn resume_lock(&mut self) {
        self.play_lock = false;
        self.pause_music_timer(false, false);
    }

    pub fn pause(&mut self) {
        self.pause_music_timer(false, true);
    }

    pub fn resume(&mut self) {
        if self.play_lock {return} //do not resume if locked
        self.pause_music_timer(false, false);
    }

    //send pause and play commands to the player and tracker, keeping the rest of the sound enabled
    pub fn soft_pause(&mut self)
    {
        self.pause_music_timer(true, true);
    }
    pub fn soft_resume(&mut self)
    {
        self.pause_music_timer(true, false);
    }


    pub fn play_sfx(&mut self, id: u8) {
        if self.no_audio {
            return;
        }

        self.send(PlaybackMessage::PlaySample(id)).unwrap();
    }

    pub fn loop_sfx(&self, id: u8) {
        if self.no_audio {
            return;
        }

        self.tx.send(PlaybackMessage::LoopSample(id)).unwrap();
    }

    pub fn loop_sfx_freq(&mut self, id: u8, freq: f32) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::LoopSampleFreq(id, freq)).unwrap();
    }

    pub fn stop_sfx(&mut self, id: u8) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::StopSample(id)).unwrap();
    }

    pub fn set_org_interpolation(&mut self, interpolation: InterpolationMode) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetOrgInterpolation(interpolation)).unwrap();

    }

    pub fn set_song_volume(&mut self, volume: f32) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetSongVolume(volume.powf(3.0))).unwrap();
    }

    pub fn set_sfx_volume(&mut self, volume: f32) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetSampleVolume(volume.powf(3.0))).unwrap();
    }

    pub fn set_sfx_samples(&mut self, id: u8, data: Vec<i16>) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetSampleData(id, data)).unwrap();
    }

    pub fn reload_songs(&mut self, constants: &EngineConstants, settings: &Settings, ctx: &mut Context) -> GameResult {
        let prev_song = self.prev_song_id;
        let current_song = self.current_song_id;

        self.play_song(0, constants, settings, ctx, false)?;
        self.play_song(prev_song, constants, settings, ctx, false)?;
        self.save_state()?;
        self.play_song(current_song, constants, settings, ctx, false)?;

        Ok(())
    }

    pub fn play_song(
        &mut self,
        song_id: usize,
        constants: &EngineConstants,
        settings: &Settings,
        ctx: &mut Context,
        fadeout: bool,
    ) -> GameResult {
        if self.current_song_id == song_id || self.no_audio {
            return Ok(());
        }

        if song_id == 0 {
            log::info!("Stopping BGM");

            self.prev_song_id = self.current_song_id;
            self.current_song_id = 0;

            self.send(PlaybackMessage::SetOrgInterpolation(settings.organya_interpolation)).unwrap();
            self.send(PlaybackMessage::SaveState).unwrap();



            if fadeout {
                self.send(PlaybackMessage::FadeoutSong).unwrap();
            } else {
                self.send(PlaybackMessage::Stop).unwrap();
            }
        } else if let Some(song_name) = constants.music_table.get(song_id) {
            let mut paths = constants.organya_paths.clone();

            paths.insert(0, "/Soundtracks/".to_owned() + &settings.soundtrack + "/");

            if let Some(soundtrack) =
            constants.soundtracks.iter().find(|s| s.available && s.name == settings.soundtrack)
            {
                paths.insert(0, soundtrack.path.clone());
            }

            let songs_paths = paths.iter().map(|prefix| {
                [
                        #[cfg(feature = "ogg-playback")]
                    (
                        SongFormat::OggMultiPart,
                        vec![format!("{}{}_intro.ogg", prefix, song_name), format!("{}{}_loop.ogg", prefix, song_name)],
                    ),
                        #[cfg(feature = "ogg-playback")]
                    (SongFormat::OggSinglePart, vec![format!("{}{}.ogg", prefix, song_name)]),
                    (SongFormat::Organya, vec![format!("{}{}.org", prefix, song_name)]),
                ]
            });

            for songs in songs_paths {
                for (format, paths) in
                songs.iter().filter(|(_, paths)| paths.iter().all(|path| filesystem::exists(ctx, path)))
                {
                    match format {
                        SongFormat::Organya => {
                            // we're sure that there's one element
                            let path = unsafe { paths.get_unchecked(0) };

                            match filesystem::open(ctx, path).map(organya::Song::load_from) {
                                Ok(Ok(org)) => {
                                    log::info!("Playing Organya BGM: {} {}", song_id, path);

                                    self.current_song_path = path.clone();
                                    self.prev_song_id = self.current_song_id;
                                    self.current_song_id = song_id;
                                    let _ = self
                                        .send(PlaybackMessage::SetOrgInterpolation(settings.organya_interpolation))
                                        .unwrap();
                                    self.send(PlaybackMessage::SaveState).unwrap();
                                    self.send(PlaybackMessage::PlayOrganyaSong(Box::new(org.clone()))).unwrap();

                                    return Ok(());
                                }
                                Ok(Err(err)) | Err(err) => {
                                    log::warn!("Failed to load Organya BGM {}: {}", song_id, err);
                                }
                            }
                        }
                        #[cfg(feature = "ogg-playback")]
                        SongFormat::OggSinglePart => {
                            // we're sure that there's one element
                            let path = unsafe { paths.get_unchecked(0) };

                            match filesystem::open(ctx, path).map(|f| {
                                OggStreamReader::new(f).map_err(|e| GameError::ResourceLoadError(e.to_string()))
                            }) {
                                Ok(Ok(song)) => {
                                    log::info!("Playing single part Ogg BGM: {} {}", song_id, path);

                                    self.current_song_path = path.clone();
                                    self.prev_song_id = self.current_song_id;
                                    self.current_song_id = song_id;
                                    self.send(PlaybackMessage::SaveState).unwrap();
                                    self.send(PlaybackMessage::PlayOggSongSinglePart(Box::new(song))).unwrap();

                                    return Ok(());
                                }
                                Ok(Err(err)) | Err(err) => {
                                    log::warn!("Failed to load single part Ogg BGM {}: {}", song_id, err);
                                }
                            }
                        }
                        #[cfg(feature = "ogg-playback")]
                        SongFormat::OggMultiPart => {
                            // we're sure that there are two elements
                            let path_intro = unsafe { paths.get_unchecked(0) };
                            let path_loop = unsafe { paths.get_unchecked(1) };

                            match (
                                filesystem::open(ctx, path_intro).map(|f| {
                                    OggStreamReader::new(f).map_err(|e| GameError::ResourceLoadError(e.to_string()))
                                }),
                                filesystem::open(ctx, path_loop).map(|f| {
                                    OggStreamReader::new(f).map_err(|e| GameError::ResourceLoadError(e.to_string()))
                                }),
                            ) {
                                (Ok(Ok(song_intro)), Ok(Ok(song_loop))) => {
                                    log::info!(
                                        "Playing multi part Ogg BGM: {} {} + {}",
                                        song_id,
                                        path_intro,
                                        path_loop
                                    );

                                    self.current_song_path = path_intro.clone();
                                    self.prev_song_id = self.current_song_id;
                                    self.current_song_id = song_id;
                                    self.send(PlaybackMessage::SaveState).unwrap();
                                    self.send(PlaybackMessage::PlayOggSongMultiPart(
                                        Box::new(song_intro),
                                        Box::new(song_loop),
                                    ))
                                        .unwrap();

                                    return Ok(());
                                }
                                (Ok(Err(err)), _) | (Err(err), _) | (_, Ok(Err(err))) | (_, Err(err)) => {
                                    log::warn!("Failed to load multi part Ogg BGM {}: {}", song_id, err);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    //load song using file path
    pub fn play_song_filepath(
        &mut self,
        song_path: &String,
        constants: &EngineConstants,
        file_format: SongFormat,
        settings: &Settings,
        ctx: &mut Context,
        fadeout: bool,
    ) -> GameResult {


        if self.current_song_path == *song_path || self.no_audio {
            return Ok(());
        }

        if song_path.is_empty() {
            log::info!("Stopping BGM");

            self.prev_song_id = self.current_song_id;
            self.current_song_id = 0;

            self.send(PlaybackMessage::SetOrgInterpolation(settings.organya_interpolation)).unwrap();
            self.send(PlaybackMessage::SaveState).unwrap();



            if fadeout {
                self.send(PlaybackMessage::FadeoutSong).unwrap();
            } else {
                self.send(PlaybackMessage::Stop).unwrap();
            }
        }
        else
        {

            match file_format {
                SongFormat::Organya => {

                    match filesystem::open(ctx, song_path).map(organya::Song::load_from) {
                        Ok(Ok(org)) => {
                            log::info!("Playing Organya BGM: {}", song_path);

                            self.current_song_path = song_path.clone();
                            self.prev_song_id = self.current_song_id;
                            self.current_song_id = 0;
                            let _ = self
                                .send(PlaybackMessage::SetOrgInterpolation(settings.organya_interpolation))
                                .unwrap();
                            self.send(PlaybackMessage::SaveState).unwrap();
                            self.send(PlaybackMessage::PlayOrganyaSong(Box::new(org.clone()))).unwrap();

                            return Ok(());
                        }
                        Ok(Err(err)) | Err(err) => {
                            log::warn!("Failed to load Organya BGM {}: {}", song_path, err);
                        }
                    }
                }
                #[cfg(feature = "ogg-playback")]
                SongFormat::OggSinglePart => {

                    match filesystem::open(ctx, song_path).map(|f| {
                        OggStreamReader::new(f).map_err(|e| GameError::ResourceLoadError(e.to_string()))
                    }) {
                        Ok(Ok(song)) => {
                            log::info!("Playing single part Ogg BGM: {}", song_path);

                            self.current_song_path = song_path.clone();
                            self.prev_song_id = self.current_song_id;
                            self.current_song_id = 0;
                            self.send(PlaybackMessage::SaveState).unwrap();
                            self.send(PlaybackMessage::PlayOggSongSinglePart(Box::new(song))).unwrap();

                            return Ok(());
                        }
                        Ok(Err(err)) | Err(err) => {
                            log::warn!("Failed to load single part Ogg BGM {}: {}", song_path, err);
                        }
                    }
                }
                #[cfg(feature = "ogg-playback")]
                SongFormat::OggMultiPart => {
                    // we're sure that there are two elements
                    let path_intro = format!("{}_intro.ogg", song_path);
                    let path_loop = format!("{}_loop.ogg", song_path);

                    match (
                        filesystem::open(ctx, path_intro).map(|f| {
                            OggStreamReader::new(f).map_err(|e| GameError::ResourceLoadError(e.to_string()))
                        }),
                        filesystem::open(ctx, path_loop).map(|f| {
                            OggStreamReader::new(f).map_err(|e| GameError::ResourceLoadError(e.to_string()))
                        }),
                    ) {
                        (Ok(Ok(song_intro)), Ok(Ok(song_loop))) => {
                            log::info!(
                                "Playing multi part Ogg BGM: {} {} + {}",
                                -1,
                                song_path,
                                song_path,
                                //&path_intro,
                                //&path_loop
                            );

                            self.current_song_path = song_path.clone();
                            self.prev_song_id = self.current_song_id;
                            self.current_song_id = 0;
                            self.send(PlaybackMessage::SaveState).unwrap();
                            self.send(PlaybackMessage::PlayOggSongMultiPart(
                                Box::new(song_intro),
                                Box::new(song_loop),
                            ))
                                .unwrap();

                            return Ok(());
                        }
                        (Ok(Err(err)), _) | (Err(err), _) | (_, Ok(Err(err))) | (_, Err(err)) => {
                            log::warn!("Failed to load multi part Ogg BGM {}: {}", -1, err);
                        }
                    }
                }
                _ =>{}
            }
        }

        Ok(())
    }



    pub fn save_state(&mut self) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        self.send(PlaybackMessage::SaveState).unwrap();
        self.prev_song_id = self.current_song_id;

        Ok(())
    }

    pub fn restore_state(&mut self) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        self.send(PlaybackMessage::RestoreState(true)).unwrap();
        self.current_song_id = self.prev_song_id;

        Ok(())
    }

    pub fn set_speed(&mut self, speed: f32) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        if speed <= 0.0 {
            return Err(InvalidValue("Speed must be bigger than 0.0!".to_owned()));
        }

        self.send(PlaybackMessage::SetSpeed(speed)).unwrap();

        Ok(())
    }

    pub fn current_song(&self) -> usize {
        self.current_song_id
    }

    pub fn set_sample_params_from_file<R: io::Read>(&mut self, id: u8, data: R) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        let mut reader = BufReader::new(data).lines();
        let mut params = PixToneParameters::empty();

        fn next_string<T: FromStr, R: io::Read>(reader: &mut Lines<BufReader<R>>) -> GameResult<T> {
            while let Some(Ok(str)) = reader.next() {
                let str = str.trim();
                if str.is_empty() || str.starts_with('#') {
                    continue;
                }

                let mut splits = str.split(':');

                let _ = splits.next();
                if let Some(str) = splits.next() {
                    return str.trim().parse::<T>().map_err(|_| {
                        GameError::ParseError("failed to parse the value as specified type.".to_string())
                    });
                } else {
                    break;
                }
            }

            Err(GameError::ParseError("unexpected end.".to_string()))
        }

        for channel in &mut params.channels {
            channel.enabled = next_string::<u8, R>(&mut reader)? != 0;
            channel.length = next_string::<u32, R>(&mut reader)?;

            channel.carrier.waveform_type = next_string::<u8, R>(&mut reader)?;
            channel.carrier.pitch = next_string::<f32, R>(&mut reader)?;
            channel.carrier.level = next_string::<i32, R>(&mut reader)?;
            channel.carrier.offset = next_string::<i32, R>(&mut reader)?;

            channel.frequency.waveform_type = next_string::<u8, R>(&mut reader)?;
            channel.frequency.pitch = next_string::<f32, R>(&mut reader)?;
            channel.frequency.level = next_string::<i32, R>(&mut reader)?;
            channel.frequency.offset = next_string::<i32, R>(&mut reader)?;

            channel.amplitude.waveform_type = next_string::<u8, R>(&mut reader)?;
            channel.amplitude.pitch = next_string::<f32, R>(&mut reader)?;
            channel.amplitude.level = next_string::<i32, R>(&mut reader)?;
            channel.amplitude.offset = next_string::<i32, R>(&mut reader)?;

            channel.envelope.initial = next_string::<i32, R>(&mut reader)?;
            channel.envelope.time_a = next_string::<i32, R>(&mut reader)?;
            channel.envelope.value_a = next_string::<i32, R>(&mut reader)?;
            channel.envelope.time_b = next_string::<i32, R>(&mut reader)?;
            channel.envelope.value_b = next_string::<i32, R>(&mut reader)?;
            channel.envelope.time_c = next_string::<i32, R>(&mut reader)?;
            channel.envelope.value_c = next_string::<i32, R>(&mut reader)?;
        }

        self.set_sample_params(id, params)
    }

    pub fn set_sample_params(&mut self, id: u8, params: PixToneParameters) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        self.send(PlaybackMessage::SetSampleParams(id, params)).unwrap();

        Ok(())
    }

    pub fn load_custom_sound_effects(&mut self, ctx: &mut Context, roots: &Vec<String>) -> GameResult {
        for path in roots.iter().rev() {
            let wavs = filesystem::read_dir(ctx, [path, "sfx/"].join(""))?
                .filter(|f| f.to_string_lossy().to_lowercase().ends_with(".wav"));

            for filename in wavs {
                if let Ok(mut file) = filesystem::open(ctx, &filename) {
                    let wav = wav::WavSample::read_from(&mut file)?;
                    let id = filename
                        .file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                        .parse::<u8>()
                        .unwrap_or(0);
                    if id == 0 {
                        continue;
                    }
                    let step = (wav.format.channels * 2) as usize;
                    let data = wav
                        .data
                        .chunks_exact(2)
                        .into_iter()
                        .step_by(step)
                        .map(|a| i16::from_ne_bytes([a[0], a[1]]))
                        .collect();

                    self.set_sfx_samples(id, data);
                }
            }
        }
        Ok(())
    }

    //read the data from the backend and update local, readable variables from it
    pub fn get_latest_track_state(&mut self) ->CurrentOrgState
    {
        //if a note changed between calls, this will add it, it's length, and its current offset to the return struct
        //only if a note is being played (changing to 255 is discarded)
        let mut return_state = CurrentOrgState::new();

        let rx_iterator = self.state_rx.try_iter();


        let mut i = 0;
        for note in rx_iterator
        {
            for j in 0..8
            {
                //note started and there wasn't already a note start event found this frame
                if note.changed[j] == true && note.keys[j] != 0xFF && return_state.changed[j] == false
                {
                    return_state.keys[j] = note.keys[j];
                    return_state.lengths[j] = note.lengths[j];
                    return_state.offsets[j] = i;
                    return_state.changed[j] = true;
                }

                //also check for drum notes, but this doesn't need to be as special because lengths don't matter
                //these are only used for event tracking
                if note.drums[j] != 0xFF
                {
                    return_state.drums[j] = note.drums[j];
                }

            }
            return_state.timestamp = note.timestamp;

            i += 1;
        }

        //correct offsets (ticks from back of frame)
        for j in 0..8
        {
            if return_state.changed[j]
            {
                return_state.offsets[j] = i - return_state.offsets[j];
            }
        }



        //there isn't a single tick where the note is considered "not" playing when same notes are placed back-to-back


        //if there are entries in there, update the local state
        //if not, keep the local state as-is
        // let lastt = rx_iterator.last();
        // if !(lastt.is_none())
        // {
        //     let unwr = lastt.unwrap();
        //     self.note_state.copy_from(&unwr);
        // }

        //me might not need this anymore...
        //self.note_state.copy_from(&return_state); //update note cache?


        return return_state;


    }

    //tracker controller functions

    //actually don't think we need this: it has to do with audio smoothing, which isn't needed becuase *there's no sound*
    pub fn set_commander_interpolation(&mut self, interpolation: InterpolationMode) {
        //do I need this
        if self.no_audio {
            return;
        }

        self.send(PlaybackMessage::SetCommanderInterpolation(interpolation)).unwrap();
    }

    //load song from hard-baked index
    pub fn play_commander(
        &mut self,
        song_id: usize,
        constants: &EngineConstants,
        settings: &Settings,
        ctx: &mut Context,
    ) -> GameResult {

        if self.current_commander_id == song_id || self.no_audio {
            return Ok(());
        }

        if song_id == 0 {
            log::info!("Stopping Tracker");

            self.prev_commander_id = self.current_commander_id;
            self.current_commander_id = 0;

            self.send(PlaybackMessage::SetCommanderInterpolation(settings.organya_interpolation)).unwrap();
            self.send(PlaybackMessage::SaveCommanderState).unwrap();

        }
        else if let Some(song_name) = constants.music_table.get(song_id) {
            let mut paths = constants.organya_paths.clone();

            paths.insert(0, "/Soundtracks/".to_owned() + &settings.soundtrack + "/");

            if let Some(soundtrack) =
            constants.soundtracks.iter().find(|s| s.available && s.name == settings.soundtrack)
            {
                paths.insert(0, soundtrack.path.clone());
            }

            //different places and search formats for the song to load in
            let songs_paths = paths.iter().map(|prefix| {
                [
                    (SongFormat::Organya, vec![format!("{}{}.org", prefix, song_name)]),
                ]
            });

            //load in a song
            for songs in songs_paths {
                for (format, paths) in
                songs.iter().filter(|(_, paths)| paths.iter().all(|path| filesystem::exists(ctx, path)))
                {
                    match format {
                        SongFormat::Organya => {
                            // we're sure that there's one element
                            let path = unsafe { paths.get_unchecked(0) };

                            match filesystem::open(ctx, path).map(organya::Song::load_from) {
                                Ok(Ok(org)) => {
                                    log::info!("Tracking from Organya File: {} {}", song_id, path);

                                    self.current_commander_path = path.clone();
                                    self.prev_commander_id = self.current_commander_id;
                                    self.current_commander_id = song_id;
                                    self.commander_tempo = org.time.wait;

                                    self.send(PlaybackMessage::SetCommanderInterpolation(settings.organya_interpolation)).unwrap();
                                    self.send(PlaybackMessage::SaveCommanderState).unwrap();
                                    self.send(PlaybackMessage::PlayCommanderSong(Box::new(org))).unwrap();

                                    return Ok(());
                                }
                                Ok(Err(err)) | Err(err) => {
                                    log::warn!("Failed to load Trackerfile {}: {}", song_id, err);
                                }
                            }
                        }
                        _ =>{}

                    }
                }
            }
        }

        Ok(())
    }

    //load in song, modified to take a string directly
    pub fn play_commander_filepath(
        &mut self,
        song_path: &String,
        //constants: &EngineConstants,
        settings: &Settings,
        ctx: &mut Context,
    ) -> GameResult {

        if self.current_commander_path == *song_path {
            return Ok(());
        }

        if song_path.is_empty() {
            log::info!("Stopping Tracker");

            self.prev_commander_id = self.current_commander_id;
            self.current_commander_id = 0;

            self.send(PlaybackMessage::SetCommanderInterpolation(settings.organya_interpolation)).unwrap();
            self.send(PlaybackMessage::SaveCommanderState).unwrap();

        }
        else
        {

            //paths.insert(0, "/Soundtracks/".to_owned() + &settings.soundtrack + "/");

            //load in a song
            match filesystem::open(ctx, song_path).map(organya::Song::load_from) {
            Ok(Ok(org)) => {
                log::info!("Tracking from Organya File: {}", song_path);
                
                self.current_commander_path = song_path.clone();
                self.prev_commander_id = self.current_commander_id;
                self.current_commander_id = 1; //nasty hack: we're not actually playing track 1, but we need to keep 0 open
                self.commander_tempo = org.time.wait;
                
                self.send(PlaybackMessage::SetCommanderInterpolation(settings.organya_interpolation)).unwrap();
                self.send(PlaybackMessage::SaveCommanderState).unwrap();
                self.send(PlaybackMessage::PlayCommanderSong(Box::new(org))).unwrap();
                
                    return Ok(());
            }
            Ok(Err(err)) | Err(err) => {
                log::warn!("Failed to load Trackerfile {}: {}", song_path, err);
                }
            }
        }

        Ok(())

    }
    


    pub fn set_commander_speed(&mut self, speed: f32) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        if speed <= 0.0 {
            return Err(InvalidValue("Speed must be bigger than 0.0!".to_owned()));
        }

        self.send(PlaybackMessage::SetCommanderSpeed(speed)).unwrap();

        Ok(())
    }

    pub fn current_commander_song(&self) -> usize {
        self.current_commander_id
    }

    pub fn current_commander_tempo(&self) ->u16
    {
        return self.commander_tempo;
    }

    //track synchro functions, halts execution of either the tracker of the music player for a time to allow the other to catch up
    pub fn freeze_song_for(&mut self, time: Duration)
    {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::FreezeSong(time)).unwrap();
    }
    pub fn freeze_tracker_for(&mut self, time: Duration)
    {
        self.send(PlaybackMessage::FreezeTracker(time)).unwrap();
    }



}

//everyone gets it now



pub(in crate::sound) enum PlaybackMessage {
    Stop,
    PlayOrganyaSong(Box<Song>),
    #[cfg(feature = "ogg-playback")]
    PlayOggSongSinglePart(Box<OggStreamReader<File>>),
    #[cfg(feature = "ogg-playback")]
    PlayOggSongMultiPart(Box<OggStreamReader<File>>, Box<OggStreamReader<File>>),
    PlaySample(u8),
    LoopSample(u8),
    LoopSampleFreq(u8, f32),
    StopSample(u8),
    SetSpeed(f32),
    SetSongVolume(f32),
    SetSampleVolume(f32),
    FadeoutSong,
    SaveState,
    RestoreState(bool),
    SetSampleParams(u8, PixToneParameters),
    SetOrgInterpolation(InterpolationMode),
    SetSampleData(u8, Vec<i16>),
    FreezeSong(Duration),
    SetFreezeSongOffset(Duration), //tells the tracker how much time has passed when we wake it back up

    //commander-related commands
    StopCommander,
    PlayCommanderSong(Box<Song>),
    SetCommanderSpeed(f32),
    SaveCommanderState,
    RestoreCommanderState(bool),
    SetCommanderInterpolation(InterpolationMode),
    FreezeTracker(Duration),
    SetFreezeTrackerOffset(Duration),

}

#[derive(PartialEq, Eq, Clone)]
enum PlaybackState {
    Stopped,
    PlayingOrg,
    #[cfg(feature = "ogg-playback")]
    PlayingOgg,
}

enum PlaybackStateType {
    None,
    Organya(SavedOrganyaPlaybackState),
    #[cfg(feature = "ogg-playback")]
    Ogg(SavedOggPlaybackState),
}

impl Default for PlaybackStateType {
    fn default() -> Self {
        Self::None
    }
}


//simplified ORG runner to be embedded inside run<>
//will run alongside whatever audio engine is playing. It will not be rendered to the audio buffer
struct OrgTelemCommander
{
    state: PlaybackState,
    saved_state: PlaybackStateType,
    speed: f32,
    org_engine: Box<OrgPlaybackEngine>,
    bank: SoundBank, //sounds to use (for initialization simplicity, they will not actually be played)
    //here for simplicity, but will not be able to be changed
    sample_rate: f32,
    //only plays if current time is larger than this
    resume_at: SystemTime,
}
impl OrgTelemCommander
{
    pub fn new(config: cpal::StreamConfig, sndbnk: SoundBank) -> OrgTelemCommander
    {
        //the sample rate is how many frames the org engine renders at a single time
        //if we set this to 1, 
        let mut org_engine = Box::new(OrgPlaybackEngine::new());
        org_engine.set_sample_rate(config.sample_rate.0 as usize);

        OrgTelemCommander
        {
            state: PlaybackState::Stopped,
            saved_state: PlaybackStateType::None,
            speed: 1.0,
            org_engine: org_engine,
            sample_rate: config.sample_rate.0 as f32,
            bank: sndbnk,//SoundBank { wave100: Box::new([0; 25600]), samples: vec![WavSample { format: (), data: () }; 16] },

            resume_at: SystemTime::UNIX_EPOCH,
        }
    }

    pub fn run(&mut self) -> bool
    {
        if self.state != PlaybackState::Stopped //stop/go
        && self.resume_at < SystemTime::now()
        {
            return self.org_engine.track_only();
        }
        return false;
    }

    pub fn get_telem(&mut self) -> CurrentOrgState
    {
        let state = self.org_engine.get_latest_track_state();
        //state.timestamp = SystemTime::now();
        return state;
    }

    pub fn get_state(&self) -> PlaybackState
    {
        self.state.clone()
    }

    pub fn set(&mut self, message: PlaybackMessage)
    {
        match message
        {

            PlaybackMessage::PlayOrganyaSong(song)=> {
                if self.state == PlaybackState::Stopped {
                    self.saved_state = PlaybackStateType::None;
                }

                self.org_engine.start_song(*song, &self.bank);

                //let _ = self.org_engine.render_to(&mut bgm_buf);
                self.org_engine.track_only();

                self.state = PlaybackState::PlayingOrg;

            }
            PlaybackMessage::SetSpeed(new_speed) => {
                assert!(new_speed > 0.0);
                    self.speed = new_speed;
                self.org_engine.set_sample_rate((self.sample_rate / new_speed) as usize);
            }
            PlaybackMessage::SaveState => {

                //save the main music state
                self.saved_state = match self.state {
                    PlaybackState::PlayingOrg => PlaybackStateType::Organya(self.org_engine.get_state()),
                    _ => PlaybackStateType::None,
                };
            }
            PlaybackMessage::RestoreState(should_rewind) => {

                let saved_state_loc = std::mem::take(&mut self.saved_state);
                //restore the correct audio backend
                match saved_state_loc {
                    PlaybackStateType::Organya(playback_state) => {
                        self.org_engine.set_state(playback_state, &self.bank);

                        if should_rewind && self.state == PlaybackState::Stopped {
                            self.org_engine.rewind();
                        }

                        //let _ = self.org_engine.render_to(&mut bgm_buf);
                        self.org_engine.track_only();

                        self.state = PlaybackState::PlayingOrg;
                    }
                    _ => {
                        self.state = PlaybackState::Stopped;
                    }

                }
            }
            PlaybackMessage::SetOrgInterpolation(interpolation) => {
                //this is useless because we don't actually make sound
                self.org_engine.interpolation = interpolation;
            }

            PlaybackMessage::Stop =>
            {
                if self.state == PlaybackState::Stopped {
                    self.saved_state = PlaybackStateType::None;
                }

                self.state = PlaybackState::Stopped;
            }

            PlaybackMessage::FreezeSong(duration) =>
            {
                self.resume_at = SystemTime::now() + duration;
            }

            PlaybackMessage::SetFreezeSongOffset(duration) =>
            {
                self.resume_at = self.resume_at + duration;
            }


            _ => {
                //do nothing
            }

        }
    }


}


//runs an audio bgm server/thread
fn run<T>(
    rx: Receiver<PlaybackMessage>, //gets this from the game
    //NEW: send stuff back to the game from the audio thread
    tx: Sender<CurrentOrgState>,
    bank: SoundBank, //sounds to use
    device: cpal::Device,
    config: cpal::StreamConfig,
) -> GameResult<cpal::Stream>
    where
        T: cpal::SizedSample + cpal::FromSample<u16>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let mut state = PlaybackState::Stopped;
    let mut saved_state: PlaybackStateType = PlaybackStateType::None;
    let mut speed = 1.0;

    /////////////////////////
    //startup delay managers
    /////////////////////////
    //only plays if current time is more than this
    let mut resume_at = SystemTime::UNIX_EPOCH;



    //create an org engine to do ORG stuff
    let mut org_engine = Box::new(OrgPlaybackEngine::new());

    //create silent tracker
    let mut tracker_engine = OrgTelemCommander::new(config.clone(), bank.clone());
    let mut tracker_ticker = 0; //used to run in sync with the sample rate





    //if OGG, create an OGG engine to do playback as well
    #[cfg(feature = "ogg-playback")]
    let mut ogg_engine = Box::new(OggPlaybackEngine::new());
    let mut pixtone = Box::new(PixTonePlayback::new());
    pixtone.create_samples();

    log::info!("Audio format: {} {}", sample_rate, channels);
    org_engine.set_sample_rate(sample_rate as usize);
    #[cfg(feature = "ogg-playback")]
    {
        //the number of times the ORG engine will loop before stopping (set to max integer size, will take several thousand years to run down)
        org_engine.loops = usize::MAX;
        //setup OGG sample rate
        ogg_engine.set_sample_rate(sample_rate as usize);
    }

    let buf_size = sample_rate as usize * 10 / 1000;
    let mut bgm_buf = vec![0x8080; buf_size * 2];
    let mut pxt_buf = vec![0x8000; buf_size];
    let mut bgm_index = 0;
    let mut pxt_index = 0;
    let mut samples = 0;
    let mut bgm_vol = 1.0_f32;
    let mut bgm_vol_saved = 1.0_f32;
    let mut sfx_vol = 1.0_f32;
    let mut bgm_fadeout = false;
    pixtone.mix(&mut pxt_buf, sample_rate);

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream_result = device.build_output_stream(
        &config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {


            //check for new commands
            //will run until the recieve buffer is empty
            loop {
                if bgm_fadeout && bgm_vol > 0.0 {
                    bgm_vol -= 0.02;
                }

                if bgm_vol < 0.0 {
                    bgm_vol = 0.0;
                }

                //detect messages from the game on how to behave
                match rx.try_recv() {
                    Ok(PlaybackMessage::PlayOrganyaSong(song)) => {
                        if state == PlaybackState::Stopped {
                            saved_state = PlaybackStateType::None;
                        }

                        if bgm_fadeout {
                            bgm_fadeout = false;
                            bgm_vol = bgm_vol_saved;
                        }

                        org_engine.start_song(*song, &bank);

                        for i in &mut bgm_buf[0..samples] {
                            *i = 0x8000
                        }
                        samples = org_engine.render_to(&mut bgm_buf);
                        bgm_index = 0;

                        state = PlaybackState::PlayingOrg;
                    }
                    #[cfg(feature = "ogg-playback")]
                    Ok(PlaybackMessage::PlayOggSongSinglePart(data)) => {
                        if state == PlaybackState::Stopped {
                            saved_state = PlaybackStateType::None;
                        }

                        if bgm_fadeout {
                            bgm_fadeout = false;
                            bgm_vol = bgm_vol_saved;
                        }

                        ogg_engine.start_single(data);

                        for i in &mut bgm_buf[0..samples] {
                            *i = 0x8000
                        }
                        samples = ogg_engine.render_to(&mut bgm_buf);
                        bgm_index = 0;

                        state = PlaybackState::PlayingOgg;
                    }
                    #[cfg(feature = "ogg-playback")]
                    Ok(PlaybackMessage::PlayOggSongMultiPart(data_intro, data_loop)) => {
                        if state == PlaybackState::Stopped {
                            saved_state = PlaybackStateType::None;
                        }

                        if bgm_fadeout {
                            bgm_fadeout = false;
                            bgm_vol = bgm_vol_saved;
                        }

                        ogg_engine.start_multi(data_intro, data_loop);

                        for i in &mut bgm_buf[0..samples] {
                            *i = 0x8000
                        }
                        samples = ogg_engine.render_to(&mut bgm_buf);
                        bgm_index = 0;

                        state = PlaybackState::PlayingOgg;
                    }
                    Ok(PlaybackMessage::PlaySample(id)) => {
                        pixtone.play_sfx(id);
                    }

                    Ok(PlaybackMessage::LoopSample(id)) => {
                        pixtone.loop_sfx(id);
                    }
                    Ok(PlaybackMessage::LoopSampleFreq(id, freq)) => {
                        pixtone.loop_sfx_freq(id, freq);
                    }
                    Ok(PlaybackMessage::StopSample(id)) => {
                        pixtone.stop_sfx(id);
                    }
                    Ok(PlaybackMessage::Stop) => {
                        if state == PlaybackState::Stopped {
                            saved_state = PlaybackStateType::None;
                        }

                        state = PlaybackState::Stopped;
                    }
                    Ok(PlaybackMessage::SetSpeed(new_speed)) => {
                        assert!(new_speed > 0.0);
                        speed = new_speed;
                        #[cfg(feature = "ogg-playback")]
                            ogg_engine.set_sample_rate((sample_rate / new_speed) as usize);
                        org_engine.set_sample_rate((sample_rate / new_speed) as usize);
                    }
                    Ok(PlaybackMessage::SetSongVolume(new_volume)) => {
                        assert!(bgm_vol >= 0.0);
                        if bgm_fadeout {
                            bgm_vol_saved = new_volume;
                        } else {
                            bgm_vol = new_volume;
                        }
                    }
                    Ok(PlaybackMessage::SetSampleVolume(new_volume)) => {
                        assert!(sfx_vol >= 0.0);
                        sfx_vol = new_volume;
                    }
                    Ok(PlaybackMessage::FadeoutSong) => {
                        bgm_fadeout = true;
                        bgm_vol_saved = bgm_vol;
                    }
                    Ok(PlaybackMessage::SaveState) => {
                        
                        //save the main music state
                        saved_state = match state {
                            PlaybackState::Stopped => PlaybackStateType::None,
                            PlaybackState::PlayingOrg => PlaybackStateType::Organya(org_engine.get_state()),
                            #[cfg(feature = "ogg-playback")]
                            PlaybackState::PlayingOgg => PlaybackStateType::Ogg(ogg_engine.get_state()),
                        };
                        
                    }
                    Ok(PlaybackMessage::RestoreState(should_rewind)) => {

                        let saved_state_loc = std::mem::take(&mut saved_state);
                        //restore the correct audio backend
                        match saved_state_loc {
                            PlaybackStateType::None => {
                                state = PlaybackState::Stopped;
                            }
                            PlaybackStateType::Organya(playback_state) => {
                                org_engine.set_state(playback_state, &bank);

                                if should_rewind && state == PlaybackState::Stopped {
                                    org_engine.rewind();
                                }

                                for i in &mut bgm_buf[0..samples] {
                                    *i = 0x8000
                                }
                                samples = org_engine.render_to(&mut bgm_buf);
                                bgm_index = 0;

                                if bgm_fadeout {
                                    bgm_fadeout = false;
                                    bgm_vol = bgm_vol_saved;
                                }

                                state = PlaybackState::PlayingOrg;
                            }
                            #[cfg(feature = "ogg-playback")]
                            PlaybackStateType::Ogg(playback_state) => {
                                ogg_engine.set_state(playback_state);

                                if should_rewind && state == PlaybackState::Stopped {
                                    ogg_engine.rewind();
                                }

                                for i in &mut bgm_buf[0..samples] {
                                    *i = 0x8000
                                }
                                samples = ogg_engine.render_to(&mut bgm_buf);
                                bgm_index = 0;

                                if bgm_fadeout {
                                    bgm_fadeout = false;
                                    bgm_vol = bgm_vol_saved;
                                }

                                state = PlaybackState::PlayingOgg;
                            }
                        }
                    
                    }
                    Ok(PlaybackMessage::SetSampleParams(id, params)) => {
                        pixtone.set_sample_parameters(id, params);
                    }
                    Ok(PlaybackMessage::SetOrgInterpolation(interpolation)) => {
                        org_engine.interpolation = interpolation;
                    }
                    Ok(PlaybackMessage::SetSampleData(id, data)) => {
                        pixtone.set_sample_data(id, data);
                    }

                    Ok(PlaybackMessage::FreezeSong(duration)) => {
                        resume_at = SystemTime::now() + duration;
                    }
                    Ok(PlaybackMessage::SetFreezeSongOffset(duration)) => {
                        resume_at = resume_at + duration;
                    }

 
                    //nuevo
                    Ok(PlaybackMessage::PlayCommanderSong(song)) => {
                        tracker_engine.set(PlaybackMessage::PlayOrganyaSong(song));
                    }
                    Ok(PlaybackMessage::SetCommanderSpeed(speed)) => {
                        tracker_engine.set(PlaybackMessage::SetSpeed(speed));
                    }
                    Ok(PlaybackMessage::SaveCommanderState) => {
                        tracker_engine.set(PlaybackMessage::SaveState);
                    }
                    Ok(PlaybackMessage::RestoreCommanderState(should_rewind)) => {
                        tracker_engine.set(PlaybackMessage::RestoreState(should_rewind));
                    }
                    Ok(PlaybackMessage::SetCommanderInterpolation(interpolation)) => {
                        tracker_engine.set(PlaybackMessage::SetOrgInterpolation(interpolation));
                    }
                    Ok(PlaybackMessage::StopCommander) => {
                        tracker_engine.set(PlaybackMessage::Stop);
                    }
                    Ok(PlaybackMessage::FreezeTracker(duration)) => {
                        tracker_engine.set(PlaybackMessage::FreezeSong(duration));
                    }

                    Ok(PlaybackMessage::SetFreezeTrackerOffset(duration)) => {
                        tracker_engine.set(PlaybackMessage::SetFreezeSongOffset(duration));
                    }


                    Ok(_)=>{}
                    
                    Err(_) => {
                        break;
                    }
                }
            
            }

            let rsm_breakout = resume_at.duration_since(SystemTime::UNIX_EPOCH).unwrap();
            let tsm_breakout  = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
            let rmm = rsm_breakout.as_secs();
            let tmm = tsm_breakout.as_secs();


            //send audio frames to backend, dols out audio data evenly between frames
            for frame in data.chunks_mut(channels) {
                let (bgm_sample_l, bgm_sample_r): (u16, u16) = {
                    //fill left and right buffers with static value
                    if state == PlaybackState::Stopped
                    || resume_at > SystemTime::now()
                    {

                        (0x8000, 0x8000)
                    }
                    else if bgm_index < samples //empty out the buffer 2 samples at a time
                    {
                        let samples = (bgm_buf[bgm_index], bgm_buf[bgm_index + 1]);
                        bgm_index += 2; //increment by l/r?
                        samples
                    }
                    else //re-fill the buffer with more fresh data and reset the read head
                    {
                        //initialize only the part of the buffer we're going to use
                        for i in &mut bgm_buf[0..samples] {
                            *i = 0x8000
                        }

                        //new: run the tracker engine in lockstep with the main music player,
                        //moved outside for the sake of pausing seprately
                        tracker_ticker += 1;
                        //tracker_engine.run();

                        //fill the initialized buffer
                        
                        match state {
                            PlaybackState::PlayingOrg => {
                                //give it a vector, it returns the number of slots it filled
                                samples = org_engine.render_to(&mut bgm_buf);
                            }
                            #[cfg(feature = "ogg-playback")]
                            PlaybackState::PlayingOgg => {
                                samples = ogg_engine.render_to(&mut bgm_buf);
                            }
                            _ => unreachable!(),
                        }
                        bgm_index = 2;
                        (bgm_buf[0], bgm_buf[1])
                    }

                };

                let pxt_sample: u16 = pxt_buf[pxt_index];

                if pxt_index < (pxt_buf.len() - 1) {
                    pxt_index += 1;
                } else {
                    pxt_index = 0;
                    pxt_buf.fill(0x8000);
                    pixtone.mix(&mut pxt_buf, sample_rate / speed);
                }

                if frame.len() >= 2 {
                    let sample_l = clamp(
                        (((bgm_sample_l ^ 0x8000) as i16) as f32 * bgm_vol) as isize
                            + (((pxt_sample ^ 0x8000) as i16) as f32 * sfx_vol) as isize,
                        -0x7fff,
                        0x7fff,
                    ) as u16
                        ^ 0x8000;
                    let sample_r = clamp(
                        (((bgm_sample_r ^ 0x8000) as i16) as f32 * bgm_vol) as isize
                            + (((pxt_sample ^ 0x8000) as i16) as f32 * sfx_vol) as isize,
                        -0x7fff,
                        0x7fff,
                    ) as u16
                        ^ 0x8000;

                    frame[0] = T::from_sample(sample_l);
                    frame[1] = T::from_sample(sample_r);
                } else {
                    let sample = clamp(
                        ((((bgm_sample_l ^ 0x8000) as i16) + ((bgm_sample_r ^ 0x8000) as i16)) as f32 * bgm_vol / 2.0)
                            as isize
                            + (((pxt_sample ^ 0x8000) as i16) as f32 * sfx_vol) as isize,
                        -0x7fff,
                        0x7fff,
                    ) as u16
                        ^ 0x8000;

                    frame[0] = T::from_sample(sample);
                }
            }


            //the only reason we have this out here is because we want to be able to play this even if the music is halted... for some reason...
            //we *technically* can do this, but it results in possible desync from the parent
            //this should keep it matched up, but only while the actual audio is playing:
            if tracker_ticker < 1 {tracker_ticker = 1}

            for _i in 0..tracker_ticker
            {
                if tracker_engine.get_state() != PlaybackState::Stopped
                {
                    //send a telemetry packet when the ORG updates
                    if tracker_engine.run()
                    {}//{let _ = tx.send(tracker_engine.get_state());}
                    let _ = tx.send(tracker_engine.get_telem());
                }
            
            }

            tracker_ticker = 0;


            // let time_passage_micros: f64 = (tracker_ticker as f64 * 1000.0 / (sample_rate as f64 * channels as f64)) * 1000000.0; //last nbr to convert to micros
            // curr_time += time_passage_micros;
            // tracker_engine.set_time(time_passage_micros);
            // //data callback runs at sample_rate * channels / 1000 per second?
            // //it's very close, but obviously off, as timers end up out of sync within a minute (this timer is slower than realTime)
            // //maybe this is better, since it is in time with the actual trackers?
            // //dummy simplification (for yours truly): 96 ticks equate to 1 second at sample rate of 48000 with 2 channels
            // //callback_count += 1;
            // if callback_count % (sample_rate as u32 * channels as u32 / 1000) == 0
            // {
            //     print!("ran {} seconds \n", callback_count / (sample_rate as u32 * channels as u32 / 1000));
            // }
            // let seconds_ticks = callback_count as f32 * 1000.0 / (sample_rate * channels as f32);
            // let duras = Duration::from_millis();
            // //reset to prevent overflow
            // if callback_count % (sample_rate as u32 * channels as u32 / 1000) == 0
            // {
            //     callback_count = 0;
            // }


        },
        err_fn,
        None
    );

    if stream_result.is_err() {
        return Err(GameError::AudioError(stream_result.err().unwrap().to_string()));
    }

    let stream = stream_result.unwrap();
    let _ = stream.play();

    Ok(stream)
}



//principle of operation:
//should the commander be completely separate from the main audio
//I think yes.

//a series of commands will be used to start and stop main song playback, and another set will be used to start, stop, and rewind the commander
//do we need save and reload for the commander?
//might as well, we already have it.
