use std::{
    borrow::Borrow, ffi::{c_void, CStr}, io, cell::RefCell, rc::Rc, sync::{
        atomic::{AtomicBool, Ordering}, Arc, Mutex, RwLock
    }
};

use crate::{framework::filesystem::File, game::Game};
use crate::framework::error::{GameError, GameResult};

use crate::sound::stuff::cubic_interp;
use crate::sound::wav::WavFormat;


use oxdz::Oxdz;


pub(crate) struct TrackerPlaybackEngine<'a> {

    player: Option<Arc<RwLock<Box<Oxdz<'a>>>>>, //Option<Box<Oxdz <- original
    output_format: WavFormat,

    position: u32,

    buffer: Vec<i16>,
}

pub struct SavedTrackerPlaybackState {
    // curr_music: Box<Rc<Module>>,   //Option<Arc<RwLock<Box<Module>>>>,
    // //position: u64,
    // position: (i32, i32), //order, row
}


impl<'a> TrackerPlaybackEngine<'a> {

    pub fn new() -> TrackerPlaybackEngine<'a> {
        TrackerPlaybackEngine {
            player: None,
            //curr_music: None,
            output_format: WavFormat { channels: 2, sample_rate: 44100, bit_depth: 16 },
            position: 0,
            buffer: Vec::with_capacity(4096),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: usize) {
        self.output_format.sample_rate = sample_rate as u32;
        if let Some(player) = self.player.as_mut() {
            let _ = player.write().unwrap().rate = sample_rate as u32;
        }
    }

    pub fn get_state(&self) -> SavedTrackerPlaybackState {
        SavedTrackerPlaybackState {
            // curr_music: self.curr_music,
            // position: self.position,
        }
    }

    pub fn set_state(&mut self, state: SavedTrackerPlaybackState) {
        // self.position = state.position;
        // self.curr_music = state.curr_music;
    }

    pub fn start_song(&mut self, music: Box<Vec<u8>>) {



        // //create new tracker player object
        // let comment = &music.as_ref().comment;
        // let is_ft2 = comment == "FastTracker v2.00 (1.04)";

        // //self.music = music
        // self.position = (0,0);
        // self.curr_music = Some(music);
        // //let mm = self.curr_music.as_ref();

        if let Ok(mut ll) = Oxdz::new(&music, self.output_format.sample_rate, "") {
            ll.set_interpolator("linear");
            self.player = Some(Arc::new(RwLock::new(Box::new(ll))));
            self.rewind();
        }




    }

    //send music back to start of file
    pub fn rewind(&mut self) {

        if let Some(player) = &mut self.player {
            let _ = player.write().unwrap().set_position(0);
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


        if let Some(player) = &mut self.player {
            //direct cast, leads to wiiware-type sound
            //player.fill_buffer( unsafe { &mut *(buf as *mut [u16] as *mut [i16]) }, 0);
            let mut player = player.write().unwrap();


            self.buffer.resize(buf.len(), 0);
            player.fill_buffer( &mut self.buffer, 0);
            self.buffer.drain(0..buf.len()).map(|n| n as u16 ^ 0x8000).zip(buf.iter_mut()).for_each(|(n, tgt)| *tgt = n);
            buf.len()

        } else {0}

    }




    pub fn load_from<R: io::Read + io::Seek>(mut f: R) -> GameResult<Vec<u8>> {

        //get file size
        let file_len = f.seek(io::SeekFrom::End(0))?;
        let _ = f.seek(io::SeekFrom::Start(0));

        let mut mod_data = vec![0; file_len as usize];
        let _ = f.read(&mut mod_data);


        Ok(mod_data)

    }


}









