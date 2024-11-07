use std::ffi::{c_void, CStr};
use std::io;
use std::io::ErrorKind;
use std::mem::{MaybeUninit, size_of};
use std::os::raw::{c_char, c_int, c_long, c_uchar};
use std::ptr::null_mut;

use crate::player::Player;

const XMP_NAME_SIZE: usize = 64;
const XMP_MAX_CHANNELS: usize = 64;
const XMP_MAX_MOD_LENGTH: usize = 256;

#[repr(C)]
struct XmpChannel {
    pan: c_int,
    vol: c_int,
    flg: c_int,
}

#[repr(C)]
struct XmpModule {
    name: [c_uchar; XMP_NAME_SIZE],
    r#type: [c_uchar; XMP_NAME_SIZE],
    pat: c_int,
    trk: c_int,
    chn: c_int,
    ins: c_int,
    smp: c_int,
    spd: c_int,
    bpm: c_int,
    len: c_int,
    rst: c_int,
    gvl: c_int,
    xxp: *mut *mut c_void,
    xxt: *mut *mut c_void,
    xxi: *mut *mut c_void,
    xxs: *mut *mut c_void,
    xxc: [XmpChannel; XMP_MAX_CHANNELS],
    xxo: [c_uchar; XMP_MAX_MOD_LENGTH],
}

#[repr(C)]
struct XmpModuleInfo {
    md5: [u8; 16],
    vol_base: c_int,
    r#mod: *mut XmpModule,
    comment: *mut c_char,
    num_sequences: c_int,
    seq_data: *mut *mut c_void,
}

type XmpContext = *mut c_char;

extern "C" {
    fn xmp_create_context() -> XmpContext;
    fn xmp_free_context(ctx: XmpContext);
    fn xmp_load_module_from_memory(ctx: XmpContext, data: *const c_void, length: c_long) -> c_int;
    fn xmp_test_module_from_memory(buffer: *mut c_void, length: c_long, info: *mut c_void) -> c_int;
    fn xmp_start_player(ctx: XmpContext, sample_rate: c_int, flags: c_int) -> c_int;
    fn xmp_play_buffer(ctx: XmpContext, buffer: *mut c_void, size: c_int, r#loop: c_int) -> c_int;
    fn xmp_get_module_info(ctx: XmpContext, mod_info: *mut XmpModuleInfo);
}

pub struct LibXmpPlayer {
    context: XmpContext,
    looping: bool,
}

fn xmp_assert(code: c_int) -> io::Result<()> {
    match code {
        0 => Ok(()),
        -2 => Err(io::Error::new(ErrorKind::InvalidData, "Internal error")),
        -3 => Err(io::Error::new(ErrorKind::InvalidData, "Unsupported module format")),
        -4 => Err(io::Error::new(ErrorKind::InvalidData, "Error loading file")),
        -5 => Err(io::Error::new(ErrorKind::InvalidData, "Error depacking file")),
        -6 => Err(io::Error::new(ErrorKind::InvalidData, "System error")),
        -7 => Err(io::Error::new(ErrorKind::InvalidData, "Invalid parameter")),
        -8 => Err(io::Error::new(ErrorKind::InvalidData, "Invalid player state")),
        _ => Err(io::Error::new(ErrorKind::InvalidData, "Unknown error code")),
    }
}

impl LibXmpPlayer {
    pub fn new(data: &[u8]) -> io::Result<LibXmpPlayer> {
        unsafe {
            let context = xmp_create_context();
            if context.is_null() {
                return Err(io::Error::new(ErrorKind::Other, "xmp_create_context() returned null."));
            }

            xmp_assert(xmp_load_module_from_memory(context, data.as_ptr() as _, data.len() as _))?;
            xmp_assert(xmp_start_player(context, 48000, 0))?;

            Ok(LibXmpPlayer {
                context,
                looping: false,
            })
        }
    }

    pub fn probe(data: &[u8]) -> bool {
        unsafe {
            xmp_test_module_from_memory(data.as_ptr() as _, data.len() as _, null_mut()) == 0
        }
    }
}

impl Drop for LibXmpPlayer {
    fn drop(&mut self) {
        unsafe {
            xmp_free_context(self.context);
        }
    }
}

impl Player for LibXmpPlayer {
    fn get_name(&self) -> Option<String> {
        unsafe {
            let mut mod_info = MaybeUninit::<XmpModuleInfo>::zeroed().assume_init();
            xmp_get_module_info(self.context, &mut mod_info as *mut XmpModuleInfo);

            let module = &*mod_info.r#mod;

            // we trust libxmp to null terminate the string.
            let name = CStr::from_ptr(module.name.as_ptr() as _);
            let name_str = name.to_string_lossy().to_string();

            // assert!(name_str.len() < XMP_NAME_SIZE);

            Some(name_str)
        }
    }

    fn get_duration_millis(&self) -> i64 {
        0
    }

    fn get_position(&self) -> i64 {
        0
    }

    fn is_looping(&self) -> bool {
        self.looping
    }

    fn set_looping(&mut self, state: bool) {
        self.looping = state;
    }

    fn seek(&mut self, pos: i64) {}

    fn get_buffer(&mut self, samples: &mut [i16]) -> bool {
        unsafe {
            xmp_play_buffer(self.context,
                            samples.as_ptr() as _,
                            (samples.len() * size_of::<i16>()) as _,
                            if self.looping { 0 } else { 1 }) == 0
        }
    }
}
