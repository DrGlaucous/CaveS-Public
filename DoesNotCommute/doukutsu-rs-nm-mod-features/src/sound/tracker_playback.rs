use std::{
    ffi::{c_void, CStr},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
        RwLock,
    },
    time::Duration,
    io,
};

use openmpt_sys;

use crate::{framework::filesystem::File, game::Game};
use crate::framework::error::{GameError, GameResult};

use crate::sound::stuff::cubic_interp;
use crate::sound::wav::WavFormat;


//get log messages from openmpt
extern "C" fn logfunc(message: *const ::std::os::raw::c_char, _user: *mut ::std::os::raw::c_void) {
    let openmpt_log_msg = unsafe { CStr::from_ptr(message) };
    //dbg!(openmpt_log_msg);

    log::info!("MPT INFO");
}

pub struct Module {
    handle: *mut openmpt_sys::openmpt_module,
}
impl Module {
        
    pub fn load_from<R: io::Read + io::Seek>(mut f: R) -> GameResult<Module> {

        //get file size
        let file_len = f.seek(io::SeekFrom::End(0))?;
        let _ = f.seek(io::SeekFrom::Start(0));

        let mut mod_data = vec![0; file_len as usize];
        f.read(&mut mod_data)?;

        let mod_handle = unsafe {
            openmpt_sys::openmpt_module_create_from_memory2(
                mod_data.as_ptr() as *const c_void,
                mod_data.len(),
                Some(logfunc),
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };

        if mod_handle.is_null() {
            return Err(GameError::ResourceLoadError(format!("Failed to load tracker file!")));
        }

        //let playback_over = Arc::new(AtomicBool::new(false));

        Ok( Module {
            handle: mod_handle,
        })

    }


}
unsafe impl Send for Module {}
unsafe impl Sync for Module {}

pub(crate) struct TrackerPlaybackEngine {
    curr_music: Option<Arc<RwLock<Box<Module>>>>,
    output_format: WavFormat,
    //position: f64, // seconds
    position: (i32, i32), //order, row

    buffer: Vec<i16>,
}

pub struct SavedTrackerPlaybackState {
    curr_music: Option<Arc<RwLock<Box<Module>>>>,
    //position: u64,
    position: (i32, i32), //order, row
}


impl TrackerPlaybackEngine {
    pub fn new() -> TrackerPlaybackEngine {
        TrackerPlaybackEngine {
            curr_music: None,
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
            curr_music: self.curr_music.clone(),
            position: self.position,
        }
    }

    pub fn set_state(&mut self, state: SavedTrackerPlaybackState) {
        self.position = state.position;
        self.curr_music = state.curr_music;
    }

    pub fn start_song(&mut self, music: Box<Module>) {
        //self.music = music
        self.position = (0,0);
        self.curr_music = Some(Arc::new(RwLock::new(music)));
        self.rewind();

    }

    //send music back to start of file
    pub fn rewind(&mut self) {

        unsafe {
            if let Some(music) = &self.curr_music {
                let module = music.write().unwrap();
                let _ = openmpt_sys::openmpt_module_set_position_order_row(
                    module.handle,
                    0,
                    0,
                );
            }
        }



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




}









