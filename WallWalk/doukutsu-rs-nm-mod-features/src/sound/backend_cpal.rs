use std::any::Any;
use std::io;
use std::io::{BufRead, BufReader, Lines};
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

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
#[cfg(feature = "tracker-playback")]
use crate::sound::tracker_playback::{TrackerPlaybackEngine, SavedTrackerPlaybackState};
#[cfg(feature = "ogg-playback")]
use crate::sound::ogg_playback::{OggPlaybackEngine, SavedOggPlaybackState};
use crate::sound::org_playback::{OrgPlaybackEngine, SavedOrganyaPlaybackState};
use crate::sound::organya;
use crate::sound::organya::Song;
use crate::sound::pixtone::{PixToneParameters, PixTonePlayback};
use crate::sound::{wave_bank, wave_bank::SoundBank, wav};
use crate::sound::backend::*;

pub struct SoundManagerCpal {
    soundbank: Option<SoundBank>,
    tx: Sender<PlaybackMessage>,
    // prev_song_id: usize,
    // current_song_id: usize,
    no_audio: bool,
    load_failed: bool,
    stream: Option<cpal::Stream>,

    c_song_id: SongId,
    p_song_id: SongId,
}

impl SoundManagerCpal {

    pub fn new(ctx: &mut Context) -> GameResult<Box<dyn SoundManager>> {
        let (tx, rx): (Sender<PlaybackMessage>, Receiver<PlaybackMessage>) = mpsc::channel();

        // null backend moved to its own struct
        // if ctx.headless {
        //     log::info!("Running in headless mode, skipping initialization.");
        //     return Ok(Box::new(SoundManagerCpal {
        //         soundbank: None,
        //         tx: tx.clone(),
        //         prev_song_id: 0,
        //         current_song_id: 0,
        //         no_audio: true,
        //         load_failed: false,
        //         stream: None,
        //     }));
        // }


        let bnk = wave_bank::SoundBank::load_from(filesystem::open(ctx, "/builtin/organya-wavetable-doukutsu.bin")?)?;
        Ok(Box::new(SoundManagerCpal::bootstrap(&bnk, tx, rx)?))
    }

    fn bootstrap(
        soundbank: &SoundBank,
        tx: Sender<PlaybackMessage>,
        rx: Receiver<PlaybackMessage>,
    ) -> GameResult<SoundManagerCpal> {
        let mut sound_manager = SoundManagerCpal {
            soundbank: Some(soundbank.to_owned()),
            tx,
            // prev_song_id: 0,
            // current_song_id: 0,
            no_audio: false,
            load_failed: false,
            stream: None,

            c_song_id: SongId::new(),
            p_song_id: SongId::new(),

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

        let res = match config.sample_format() {
            cpal::SampleFormat::I8 => run::<i8>(rx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::I16 => run::<i16>(rx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::I32 => run::<i32>(rx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::I64 => run::<i64>(rx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::U8 => run::<u8>(rx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::U16 => run::<u16>(rx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::U32 => run::<u32>(rx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::U64 => run::<u64>(rx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::F32 => run::<f32>(rx, soundbank.to_owned(), device, config.into()),
            cpal::SampleFormat::F64 => run::<f64>(rx, soundbank.to_owned(), device, config.into()),
            _ => Err(AudioError("Unsupported sample format.".to_owned())),
        };

        if let Err(res) = &res {
            log::error!("Error initializing audio: {}", res);
        }

        sound_manager.stream = res.ok();
        Ok(sound_manager)
    }

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

}

impl SoundManager for SoundManagerCpal {

    fn reload(&mut self) -> GameResult<()> {
        if self.no_audio {
            log::info!("Skipping sound manager reload because audio is not enabled.");
            return Ok(());
        }

        log::info!("Reloading sound manager.");

        let (tx, rx): (Sender<PlaybackMessage>, Receiver<PlaybackMessage>) = mpsc::channel();
        let soundbank = self.soundbank.take().unwrap();
        *self = SoundManagerCpal::bootstrap(&soundbank, tx, rx)?;

        Ok(())
    }


    fn pause(&mut self) {
        if let Some(stream) = &mut self.stream {
            let _ = stream.pause();
        }
    }

    fn resume(&mut self) {
        if let Some(stream) = &mut self.stream {
            let _ = stream.play();
        }
    }

    fn play_sfx(&mut self, id: u8) {
        if self.no_audio {
            return;
        }

        self.send(PlaybackMessage::PlaySample(id)).unwrap();
    }

    fn loop_sfx(&self, id: u8) {
        if self.no_audio {
            return;
        }

        self.tx.send(PlaybackMessage::LoopSample(id)).unwrap();
    }

    fn loop_sfx_freq(&mut self, id: u8, freq: f32) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::LoopSampleFreq(id, freq)).unwrap();
    }

    fn stop_sfx(&mut self, id: u8) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::StopSample(id)).unwrap();
    }

    fn set_org_interpolation(&mut self, interpolation: InterpolationMode) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetOrgInterpolation(interpolation)).unwrap();
    }

    fn set_song_volume(&mut self, volume: f32) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetSongVolume(volume.powf(3.0))).unwrap();
    }

    fn set_sfx_volume(&mut self, volume: f32) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetSampleVolume(volume.powf(3.0))).unwrap();
    }

    fn set_sfx_samples(&mut self, id: u8, data: Vec<i16>) {
        if self.no_audio {
            return;
        }
        self.send(PlaybackMessage::SetSampleData(id, data)).unwrap();
    }

    fn reload_songs(&mut self, constants: &EngineConstants, settings: &Settings, ctx: &mut Context) -> GameResult {
        let mut prev_song = self.p_song_id.clone();
        let mut current_song = self.c_song_id.clone();

        // self.play_song(0, constants, settings, ctx, false)?;
        // self.play_song(prev_song, constants, settings, ctx, false)?;
        // self.save_state()?;
        // self.play_song(current_song, constants, settings, ctx, false)?;

        self.play_song_from_id(&mut SongId::new(), constants, settings, ctx, false);
        self.play_song_from_id(&mut prev_song, constants, settings, ctx, false);
        self.save_state()?;
        self.play_song_from_id(&mut current_song, constants, settings, ctx, false);

        Ok(())
    }

    fn play_song_from_id(
        &mut self,
        song_id: &mut SongId,
        constants: &EngineConstants,
        settings: &Settings,
        ctx: &mut Context,
        fadeout: bool,
    ) -> GameResult {

        if song_id.loaded_from_path {
            self.play_song_filepath(&song_id.path, song_id.song_format, constants, settings, ctx, fadeout)
        } else {
            self.play_song(song_id.id, constants, settings, ctx, fadeout)
        }

    }

    //load song using numeric ID
    fn play_song(
        &mut self,
        song_id: usize,
        constants: &EngineConstants,
        settings: &Settings,
        ctx: &mut Context,
        fadeout: bool,
    ) -> GameResult {


        if (self.c_song_id.id == song_id || self.no_audio) && !self.c_song_id.loaded_from_path {
            return Ok(());
        }

        if song_id == 0 {
            log::info!("Stopping BGM");

            self.p_song_id = self.c_song_id.clone();
            self.c_song_id = SongId::new();


            self.send(PlaybackMessage::SetOrgInterpolation(settings.organya_interpolation)).unwrap();
            self.send(PlaybackMessage::SaveState).unwrap();

            if fadeout {
                self.send(PlaybackMessage::FadeoutSong).unwrap();
            } else {
                self.send(PlaybackMessage::Stop).unwrap();
            }
        } else if let Some(song_name) = constants.music_table.get(song_id) {
        //} else {

            //un-comment this if...
            //we no longer go off the internal name table: we now address the song directly by its id number in the files
            //let sid = format!("{}", song_id);
            //let song_name = &sid;
            
            
            let mut paths = constants.organya_paths.clone();

            paths.insert(0, "/Soundtracks/".to_owned() + &settings.soundtrack + "/");

            if let Some(soundtrack) = constants.soundtracks.iter().find(|s| s.available && s.id == settings.soundtrack)
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
                    #[cfg(feature = "tracker-playback")]
                    (SongFormat::Tracker, vec![format!("{}{}", prefix, song_name)]),
                    (SongFormat::Organya, vec![format!("{}{}.org", prefix, song_name)]),
                ]
            });

            for songs in songs_paths {

                for (format, paths) in songs.iter() {
                    log::info!("{}", paths[0]);
                }



                for (format, paths) in {

                    #[cfg(feature = "tracker-playback")]
                    {
                        songs.iter().filter(|(sformat, paths)| paths.iter().all(|path| filesystem::exists(ctx, path) || *sformat == SongFormat::Tracker))
                    }
                    
                    #[cfg(not(feature = "tracker-playback"))]
                    {
                        songs.iter().filter(|(_, paths)| paths.iter().all(|path| filesystem::exists(ctx, path)))
                    }
                }
                {
                    match format {
                        SongFormat::Organya => {
                            // we're sure that there's one element
                            let path = unsafe { paths.get_unchecked(0) };

                            match filesystem::open(ctx, path).map(organya::Song::load_from) {
                                Ok(Ok(org)) => {
                                    log::info!("Playing Organya BGM: {} {}", song_id, path);

                                    self.p_song_id = self.c_song_id.clone();
                                    self.c_song_id = SongId{
                                        loaded_from_path: false,
                                        song_format: *format,
                                        path: path.clone(),
                                        id: song_id,
                                    };
                                    // self.current_song_path = path.clone();
                                    // self.prev_song_id = self.current_song_id;
                                    // self.current_song_id = song_id;
                                    let _ = self
                                        .send(PlaybackMessage::SetOrgInterpolation(settings.organya_interpolation))
                                        .unwrap();
                                    self.send(PlaybackMessage::SaveState).unwrap();
                                    self.send(PlaybackMessage::PlayOrganyaSong(Box::new(org))).unwrap();

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

                                    self.p_song_id = self.c_song_id.clone();
                                    self.c_song_id = SongId{
                                        loaded_from_path: false,
                                        song_format: *format,
                                        path: path.clone(),
                                        id: song_id,
                                    };
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

                                    self.p_song_id = self.c_song_id.clone();
                                    self.c_song_id = SongId{
                                        loaded_from_path: false,
                                        song_format: *format,
                                        path: path_intro.clone(),
                                        id: song_id,
                                    };
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
                        #[cfg(feature = "tracker-playback")]
                        SongFormat::Tracker => {

                            //try all the different potential extensions
                            for tracker_extension in constants.tracker_extensions.iter() {
                            //{
                                // we're sure that there's one element
                                //let path = unsafe { paths.get_unchecked(0) };
                                let path = format!("{}{}", unsafe { paths.get_unchecked(0) }, tracker_extension);

                                match filesystem::open(ctx, &path).map(TrackerPlaybackEngine::load_from) {
                                    Ok(Ok(module_s)) => {
                                        log::info!("Playing Tracker: {} {}", song_id, &path);

                                        self.p_song_id = self.c_song_id.clone();
                                        self.c_song_id = SongId{
                                            loaded_from_path: false,
                                            song_format: *format,
                                            path: path.clone(),
                                            id: song_id,
                                        };
                                        self.send(PlaybackMessage::SaveState).unwrap();
                                        self.send(PlaybackMessage::PlayTrackerSong(Box::new(module_s))).unwrap();

                                        return Ok(());
                                    }
                                    Ok(Err(_err)) | Err(_err) => {
                                        //log::warn!("Failed to load Tracker BGM {}: {}", song_id, err);
                                    }
                                }

                            }
                            log::warn!("Failed to load Tracker BGM {}", song_id);

                        }
                    }
                }
            }
        }

        Ok(())
    }


    //load song using file path
    fn play_song_filepath(
        &mut self,
        song_path: &String,
        file_format: SongFormat,
        constants: &EngineConstants,
        settings: &Settings,
        ctx: &mut Context,
        fadeout: bool,
    ) -> GameResult {


        if self.c_song_id.path == *song_path && self.c_song_id.loaded_from_path || self.no_audio {
            return Ok(());
        }

        if song_path.is_empty() {
            log::info!("Stopping BGM");

            self.p_song_id = self.c_song_id.clone();
            self.c_song_id = SongId::new();

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

                            self.p_song_id = self.c_song_id.clone();
                            self.c_song_id = SongId{
                                loaded_from_path: true,
                                song_format: file_format,
                                path: song_path.clone(),
                                id: 0,
                            };

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

                            self.p_song_id = self.c_song_id.clone();
                            self.c_song_id = SongId{
                                loaded_from_path: true,
                                song_format: file_format,
                                path: song_path.clone(),
                                id: 0,
                            };

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

                            self.p_song_id = self.c_song_id.clone();
                            self.c_song_id = SongId{
                                loaded_from_path: true,
                                song_format: file_format,
                                path: song_path.clone(),
                                id: 0,
                            };

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
                #[cfg(feature = "tracker-playback")]
                SongFormat::Tracker => {

                    match filesystem::open(ctx, song_path).map(TrackerPlaybackEngine::load_from) {
                        Ok(Ok(module_s)) => {
                            log::info!("Playing Tracker: {} {}", song_path, song_path);

                            self.p_song_id = self.c_song_id.clone();
                            self.c_song_id = SongId{
                                loaded_from_path: true,
                                song_format: file_format,
                                path: song_path.clone(),
                                id: 0,
                            };

                            self.send(PlaybackMessage::SaveState).unwrap();
                            self.send(PlaybackMessage::PlayTrackerSong(Box::new(module_s))).unwrap();

                            return Ok(());
                        }
                        Ok(Err(err)) | Err(err) => {
                            log::warn!("Failed to load Tracker BGM {}: {}", -1, err);
                        }
                    }
                }
                //_ =>{}
            }
        }

        Ok(())
    }

        
    fn save_state(&mut self) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        self.send(PlaybackMessage::SaveState).unwrap();
        self.p_song_id = self.c_song_id.clone();

        Ok(())
    }

    fn restore_state(&mut self) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        self.send(PlaybackMessage::RestoreState).unwrap();
        self.c_song_id = self.p_song_id.clone();

        Ok(())
    }

    fn set_speed(&mut self, speed: f32) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        if speed <= 0.0 {
            return Err(InvalidValue("Speed must be bigger than 0.0!".to_owned()));
        }

        self.send(PlaybackMessage::SetSpeed(speed)).unwrap();

        Ok(())
    }

    fn current_song(&self) -> SongId {
        self.c_song_id.clone()
    }

    
    fn set_sample_params_from_file(&mut self, id: u8, data: Box<dyn io::Read>) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        let mut reader = BufReader::new(data).lines();
        let mut params = PixToneParameters::empty();

        fn next_string<T: FromStr>(reader: &mut Lines<BufReader<Box::<dyn io::Read>>>) -> GameResult<T> {
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
            channel.enabled = next_string::<u8>(&mut reader)? != 0;
            channel.length = next_string::<u32>(&mut reader)?;

            channel.carrier.waveform_type = next_string::<u8>(&mut reader)?;
            channel.carrier.pitch = next_string::<f32>(&mut reader)?;
            channel.carrier.level = next_string::<i32>(&mut reader)?;
            channel.carrier.offset = next_string::<i32>(&mut reader)?;

            channel.frequency.waveform_type = next_string::<u8>(&mut reader)?;
            channel.frequency.pitch = next_string::<f32>(&mut reader)?;
            channel.frequency.level = next_string::<i32>(&mut reader)?;
            channel.frequency.offset = next_string::<i32>(&mut reader)?;

            channel.amplitude.waveform_type = next_string::<u8>(&mut reader)?;
            channel.amplitude.pitch = next_string::<f32>(&mut reader)?;
            channel.amplitude.level = next_string::<i32>(&mut reader)?;
            channel.amplitude.offset = next_string::<i32>(&mut reader)?;

            channel.envelope.initial = next_string::<i32>(&mut reader)?;
            channel.envelope.time_a = next_string::<i32>(&mut reader)?;
            channel.envelope.value_a = next_string::<i32>(&mut reader)?;
            channel.envelope.time_b = next_string::<i32>(&mut reader)?;
            channel.envelope.value_b = next_string::<i32>(&mut reader)?;
            channel.envelope.time_c = next_string::<i32>(&mut reader)?;
            channel.envelope.value_c = next_string::<i32>(&mut reader)?;
        }

        self.set_sample_params(id, params)
    }

    fn set_sample_params(&mut self, id: u8, params: PixToneParameters) -> GameResult {
        if self.no_audio {
            return Ok(());
        }

        self.send(PlaybackMessage::SetSampleParams(id, params)).unwrap();

        Ok(())
    }

    fn load_custom_sound_effects(&mut self, ctx: &mut Context, roots: &Vec<String>) -> GameResult {
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

    fn as_any(&self) -> &dyn Any {
        self
    }

}

pub(in crate::sound) enum PlaybackMessage {
    Stop,
    PlayOrganyaSong(Box<Song>),
    #[cfg(feature = "ogg-playback")]
    PlayOggSongSinglePart(Box<OggStreamReader<File>>),
    #[cfg(feature = "ogg-playback")]
    PlayOggSongMultiPart(Box<OggStreamReader<File>>, Box<OggStreamReader<File>>),
    #[cfg(feature = "tracker-playback")]
    PlayTrackerSong(Box<Vec<u8>>),
    PlaySample(u8),
    LoopSample(u8),
    LoopSampleFreq(u8, f32),
    StopSample(u8),
    SetSpeed(f32),
    SetSongVolume(f32),
    SetSampleVolume(f32),
    FadeoutSong,
    SaveState,
    RestoreState,
    SetSampleParams(u8, PixToneParameters),
    SetOrgInterpolation(InterpolationMode),
    SetSampleData(u8, Vec<i16>),
}

#[derive(PartialEq, Eq)]
enum PlaybackState {
    Stopped,
    PlayingOrg,
    #[cfg(feature = "ogg-playback")]
    PlayingOgg,
    #[cfg(feature = "tracker-playback")]
    PlayingTracker,
}

enum PlaybackStateType { //<'a> {
    None,
    Organya(SavedOrganyaPlaybackState),
    #[cfg(feature = "ogg-playback")]
    Ogg(SavedOggPlaybackState),
    #[cfg(feature = "tracker-playback")]
    Tracker(SavedTrackerPlaybackState), //<'a>),
}

impl<'a> Default for PlaybackStateType { //<'a> {
    fn default() -> Self {
        Self::None
    }
}

fn run<T>(
    rx: Receiver<PlaybackMessage>,
    bank: SoundBank,
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
    let mut org_engine = Box::new(OrgPlaybackEngine::new());
    #[cfg(feature = "ogg-playback")]
    let mut ogg_engine = Box::new(OggPlaybackEngine::new());
    #[cfg(feature = "tracker-playback")]
    let mut tracker_engine = Box::new(TrackerPlaybackEngine::new());
    let mut pixtone = Box::new(PixTonePlayback::new());
    pixtone.create_samples();

    log::info!("Audio format: {} {}", sample_rate, channels);
    org_engine.set_sample_rate(sample_rate as usize);
    #[cfg(feature = "ogg-playback")]
    {
        org_engine.loops = usize::MAX;
        ogg_engine.set_sample_rate(sample_rate as usize);
    }
    #[cfg(feature = "tracker-playback")]
    {
        org_engine.loops = usize::MAX;
        tracker_engine.set_sample_rate(sample_rate as usize);
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
            loop {
                if bgm_fadeout && bgm_vol > 0.0 {
                    bgm_vol -= 0.02;
                }

                if bgm_vol < 0.0 {
                    bgm_vol = 0.0;
                }

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
                    #[cfg(feature = "tracker-playback")]
                    Ok(PlaybackMessage::PlayTrackerSong(module_s)) => {
                        if state == PlaybackState::Stopped {
                            saved_state = PlaybackStateType::None;
                        }

                        if bgm_fadeout {
                            bgm_fadeout = false;
                            bgm_vol = bgm_vol_saved;
                        }

                        tracker_engine.start_song(module_s);

                        for i in &mut bgm_buf[0..samples] {
                            *i = 0x8000
                        }
                        samples = tracker_engine.render_to(&mut bgm_buf);
                        bgm_index = 0;

                        state = PlaybackState::PlayingTracker;
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
                        #[cfg(feature = "tracker-playback")]
                        tracker_engine.set_sample_rate((sample_rate / new_speed) as usize);
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
                        saved_state = match state {
                            PlaybackState::Stopped => PlaybackStateType::None,
                            PlaybackState::PlayingOrg => PlaybackStateType::Organya(org_engine.get_state()),
                            #[cfg(feature = "ogg-playback")]
                            PlaybackState::PlayingOgg => PlaybackStateType::Ogg(ogg_engine.get_state()),
                            #[cfg(feature = "tracker-playback")]
                            PlaybackState::PlayingTracker => PlaybackStateType::Tracker(tracker_engine.get_state()),    
                        };
                    }
                    Ok(PlaybackMessage::RestoreState) => {
                        let saved_state_loc = std::mem::take(&mut saved_state);

                        match saved_state_loc {
                            PlaybackStateType::None => {
                                state = PlaybackState::Stopped;
                            }
                            PlaybackStateType::Organya(playback_state) => {
                                org_engine.set_state(playback_state, &bank);

                                if state == PlaybackState::Stopped {
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

                                if state == PlaybackState::Stopped {
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
                            #[cfg(feature = "tracker-playback")]
                            PlaybackStateType::Tracker(playback_state) => {
                                tracker_engine.set_state(playback_state);

                                if state == PlaybackState::Stopped {
                                    tracker_engine.rewind();
                                }

                                for i in &mut bgm_buf[0..samples] {
                                    *i = 0x8000
                                }
                                samples = tracker_engine.render_to(&mut bgm_buf);
                                bgm_index = 0;

                                if bgm_fadeout {
                                    bgm_fadeout = false;
                                    bgm_vol = bgm_vol_saved;
                                }

                                state = PlaybackState::PlayingTracker;
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
                    Err(_) => {
                        break;
                    }
                }
            }

            for frame in data.chunks_mut(channels) {
                let (bgm_sample_l, bgm_sample_r): (u16, u16) = {
                    if state == PlaybackState::Stopped {
                        (0x8000, 0x8000)
                    } else if bgm_index < samples {
                        let samples = (bgm_buf[bgm_index], bgm_buf[bgm_index + 1]);
                        bgm_index += 2;
                        samples
                    } else {
                        for i in &mut bgm_buf[0..samples] {
                            *i = 0x8000
                        }

                        match state {
                            PlaybackState::PlayingOrg => {
                                samples = org_engine.render_to(&mut bgm_buf);
                            }
                            #[cfg(feature = "ogg-playback")]
                            PlaybackState::PlayingOgg => {
                                samples = ogg_engine.render_to(&mut bgm_buf);
                            }
                            #[cfg(feature = "tracker-playback")]
                            PlaybackState::PlayingTracker => {
                                samples = tracker_engine.render_to(&mut bgm_buf);
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
        },
        err_fn,
        None,
    );

    if stream_result.is_err() {
        return Err(GameError::AudioError(stream_result.err().unwrap().to_string()));
    }

    let stream = stream_result.unwrap();
    let _ = stream.play();

    Ok(stream)
}
