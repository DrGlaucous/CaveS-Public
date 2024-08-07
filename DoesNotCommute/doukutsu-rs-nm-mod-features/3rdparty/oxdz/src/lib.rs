//!
//! ## Formats and players
//!
//! Oxidrizzle can load different module formats, and play them using
//! different format replayers.
//!
//! Module formats have a preferred player, but they can be played by
//! alternative players as long as they're compatible. For example, a
//! Noisetracker module is played using the NoiseReplay player by default,
//! but it can also be played by Protracker, FastTracker, His Master's
//! NoiseTracker or Scream Tracker 3 players. Depending on the module,
//! the result may not be identical, since each player has its own
//! algorithms and quirks.
//!

extern crate byteorder;
extern crate md5;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate save_restore_derive;

#[macro_use]
mod util;

mod player;
mod mixer;

pub mod format;
pub mod module;
pub use player::FrameInfo;
pub use player::PlayerInfo;
pub use format::FormatInfo;
pub use module::Module;

use std::error;
use std::fmt;
use std::io;

pub const PERIOD_BASE  : f64 = 13696.0;  // C0 period
/// The maximum sampling supported by the sound mixer.
pub const MAX_RATE     : i32 = 96000;
/// The minimum sampling supported by the sound mixer.
pub const MIN_RATE     : i32 = 4000;
pub const MIN_BPM      : i32 = 20;
// frame rate = (50 * bpm / 125) Hz
// frame size = (sampling rate * channels) / frame rate
pub const MAX_FRAMESIZE: usize = (5 * MAX_RATE / MIN_BPM) as usize;
/// The maximum number of notes supported in pattern events.
pub const MAX_KEYS     : usize = 128;
/// The maximum number of mixer voices.
pub const MAX_CHANNELS : usize = 64;
pub const MAX_SEQUENCES: usize = 16;

#[derive(Default)]
pub struct ModuleInfo {
    /// The module title.
    pub title: String,
    /// The module format identifier.
    pub format_id: &'static str,
    /// The module format description.
    pub description: String,
    /// The program used to create this module (usually a tracker).
    pub creator: String,
    /// The number of channels used in the module.
    pub channels: usize,
    /// The primary player for this format.
    pub player: &'static str,
    /// Total replay time in ms.
    pub total_time : u32,
}

impl ModuleInfo {
    /// Create a new `ModuleInfo`.
    pub fn new() -> Self {
        Default::default()
    }
}


pub struct Oxdz<'a> {
    pub player   : player::Player<'a>,
    pub rate     : u32,
    pub player_id: String,
    pub md5sum   : [u8; 16],
}

impl<'a> Oxdz<'a> {
    pub fn new(b: &[u8], rate: u32, player_id: &str) -> Result<Self, Error> {
        let mut module = format::load(&b, &player_id)?;
        let id = (if player_id.is_empty() { module.player } else { player_id }).to_owned();

        // import the module if needed
        module = player::list_by_id(&id)?.import(module)?;

        // store digest as an array of bytes
        let md5::Digest(md5sum) = md5::compute(b);
	debug!("md5sum: {}", md5sum.iter().fold("".to_owned(), |mut s, x| { s.push_str(&format!("{:x}", x)); s } ));

        let mut player = player::Player::find(module, rate, &id, "")?;
        player.scan();  // scan calls start() before proceeding, and reset() at end

        Ok(Oxdz {
            player,
            rate,
            player_id: id,
            md5sum,
        })
    }

    pub fn module(&'a self) -> &'a module::Module {
        &self.player.module
    }

    /// Retrieve player information.
    pub fn player_info(&self) -> Result<player::PlayerInfo, Error> {
        Ok(player::list_by_id(&self.player_id)?.info())
    }

    /// Retrieve module information.
    pub fn module_info(&self, mi: &mut ModuleInfo) {
        mi.title = self.player.module.title().to_owned();
        mi.format_id = self.player.module.format_id;
        mi.description = self.player.module.description.to_owned();
        mi.creator = self.player.module.creator.to_owned();
        mi.channels = self.player.module.channels;
        mi.player = self.player.module.player;
        mi.total_time = self.player.total_time;
    }

    /// Retrieve frame information.
    pub fn frame_info(&mut self, mut fi: &mut FrameInfo) -> &mut Self {
        self.player.info(&mut fi);
        self
    }

    pub fn fill_buffer(&mut self, mut buffer: &mut [i16], loops: usize) -> &mut Self {
        self.player.fill_buffer(&mut buffer, loops);
        self
    }

    /// Play a module frame and renders the output on an internal buffer.
    /// The buffer can be retrieved using the `buffer()` function.
    pub fn play_frame(&mut self) -> &mut Self {
        self.player.play_frame();
        self
    }

    pub fn buffer(&self) -> &[i16] {
        self.player.buffer()
    }

    pub fn set_mute(&mut self, chn: usize, val: bool) -> &mut Self {
        self.player.set_mute(chn, val);
        self
    }

    pub fn set_mute_all(&mut self, val: bool) -> &mut Self {
        self.player.set_mute_all(val);
        self
    }

    pub fn set_position(&mut self, pos: usize) -> &mut Self {
        self.player.set_position(pos);
        self
    }

    pub fn set_interpolator(&mut self, name: &str) -> Result<&mut Self, Error> {
        self.player.set_interpolator(name)?;
        Ok(self)
    }

/*
    pub fn player(&'a mut self) -> &'a mut player::Player {
        &mut self.player
    }
*/
}

/// Retrieve the list of supported module formats.
pub fn format_list() -> Vec<FormatInfo> {
    format::list()
}

/// Retrieve the list of available players.
pub fn player_list() -> Vec<PlayerInfo> {
    player::list()
}


#[derive(Debug)]
pub enum Error {
    Format(String),
    Player(String),
    Load(String),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Format(ref descr) => write!(f, "{}", descr),
            &Error::Player(ref descr) => write!(f, "{}", descr),
            &Error::Load(ref descr)   => write!(f, "{}", descr),
            &Error::Io(ref err)       => write!(f, "{}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Format(_)   => "Unsupported module format",
            Error::Player(_)   => "Can't play module",
            Error::Load(_)     => "Can't load module data",
            Error::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            _                  => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

