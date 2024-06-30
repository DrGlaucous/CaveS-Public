use std::{
    borrow::Borrow, ffi::{c_void, CStr}, io, cell::RefCell, rc::Rc, sync::{
        atomic::{AtomicBool, Ordering}, Arc, Mutex, RwLock
    }
};

use crate::{framework::filesystem::File, game::Game};
use crate::framework::error::{GameError, GameResult};

use crate::sound::stuff::cubic_interp;
use crate::sound::wav::WavFormat;


use xmrs::prelude::*;
use xmrs::xm::xmmodule::XmModule;
use crate::sound::xmrs_player::player::XmrsPlayerMod;

use oxdz::{self, Oxdz};


//use xmrsplayer::prelude::*;

// pub struct Runner {
//     module: Option<Module>
// }
// impl Runner {

//     pub fn load_from<R: io::Read + io::Seek>(mut f: R) -> GameResult<Module> {

//         //get file size
//         let file_len = f.seek(io::SeekFrom::End(0))?;
//         let _ = f.seek(io::SeekFrom::Start(0));

//         let mut mod_data = vec![0; file_len as usize];
//         f.read(&mut mod_data)?;

//         let xm = XmModule::load(&mod_data)?;

//         let module = xm.to_module();

//         Ok(module)

//     }


// }
// unsafe impl Send for Runner {}
// unsafe impl Sync for Runner {}

pub(crate) struct TrackerPlaybackEngine<'a> {

    player: Option<Oxdz<'a>>, //Arc<RwLock<Oxdz<'a>>>>,
    output_format: WavFormat,
    //position: f64, // seconds
    position: (i32, i32), //order, row

    buffer: Vec<i16>,
}

pub struct SavedTrackerPlaybackState {
    curr_music: Box<Rc<Module>>,   //Option<Arc<RwLock<Box<Module>>>>,
    //position: u64,
    position: (i32, i32), //order, row
}


impl<'a> TrackerPlaybackEngine<'a> {

    pub fn new() -> TrackerPlaybackEngine<'a> {
        TrackerPlaybackEngine {
            player: None,
            //curr_music: None,
            output_format: WavFormat { channels: 2, sample_rate: 44100, bit_depth: 16 },
            position: (0,0),
            buffer: Vec::with_capacity(4096),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: usize) {
        self.output_format.sample_rate = sample_rate as u32;
    }

    pub fn get_state(&self) -> SavedTrackerPlaybackState {
        SavedTrackerPlaybackState {
            curr_music: self.curr_music,
            position: self.position,
        }
    }

    pub fn set_state(&mut self, state: SavedTrackerPlaybackState) {
        self.position = state.position;
        self.curr_music = state.curr_music;
    }

    pub fn start_song(&mut self, music: Box<Module>) {



        //create new tracker player object
        let comment = &music.as_ref().comment;
        let is_ft2 = comment == "FastTracker v2.00 (1.04)";

        //self.music = music
        self.position = (0,0);
        self.curr_music = Some(music);
        //let mm = self.curr_music.as_ref();


        self.player = Some(XmrsPlayerMod::new(
            self.borrow().curr_music.as_ref().unwrap(),
            self.output_format.sample_rate as f32,
            is_ft2
        ));

        self.rewind();

    }

    //send music back to start of file
    pub fn rewind(&mut self) {

        if let Some(music) = &self.curr_music {
            let module = music.write().unwrap();

        }



        // unsafe {
        //     if let Some(music) = &self.curr_music {
        //         let module = music.write().unwrap();
        //         let _ = openmpt_sys::openmpt_module_set_position_order_row(
        //             module.handle,
        //             0,
        //             0,
        //         );
        //     }
        // }

        // if let Some(music) = &self.intro_music {
        //     let _ = music.write().unwrap().seek_absgp_pg(0);
        //     self.position = 0;
        //     self.playing_intro = true;
        // } else {
        //     if let Some(music) = &self.loop_music {
        //         let _ = music.write().unwrap().seek_absgp_pg(0);
        //     }
        //     self.position = 0;
        //     self.playing_intro = false;
        // }



    }

    pub fn render_to(&mut self, buf: &mut [u16]) -> usize {

        //fill internal buffer
        //while self.buffer.len() < buf.len() {
        //    self.decode();
        //}
        //then send it out to the external one
        //self.buffer.drain(0..buf.len()).map(|n| n as u16 ^ 0x8000).zip(buf.iter_mut()).for_each(|(n, tgt)| *tgt = n);
        //buf.len()

        //self.read(self.output_format.sample_rate as i32, unsafe { &mut *(buf as *mut [u16] as *mut [i16]) })

        unsafe {
            let data = &mut *(buf as *mut [u16] as *mut [i16]);
            let rate = self.output_format.sample_rate as i32;

            let mut n_read = 0;
            if let Some(music) = &self.curr_music {
                let module = music.write().unwrap();
                n_read = openmpt_sys::openmpt_module_read_interleaved_stereo(
                    module.handle,
                    rate,
                    data.len() / 2,
                    data.as_mut_ptr(),
                ); 
            }

            n_read
        }

    }




    pub fn load_from<R: io::Read + io::Seek>(mut f: R) -> GameResult<TrackerPlaybackEngine<'a>> {

        //get file size
        let file_len = f.seek(io::SeekFrom::End(0))?;
        let _ = f.seek(io::SeekFrom::Start(0));

        let mut mod_data = vec![0; file_len as usize];
        f.read(&mut mod_data)?;

        let xm = if let Ok(ll) = Oxdz::new(&mod_data, 44100, "") {ll}
        else {
            return Err(GameError::FilesystemError(format!("Could not parse module")))
        };

        let push = TrackerPlaybackEngine{
            player: Some(xm),
            output_format: WavFormat { channels: 2, sample_rate: 44100, bit_depth: 16 },
            position: (0,0),
            buffer: Vec::with_capacity(4096),
        };
        Ok(push)

        //player: Option<XmrsPlayerMod<'a>>, //Option<Arc<RwLock<XmrsPlayerMod>>>
        //Ok(module)

    }


}









