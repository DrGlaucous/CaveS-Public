mod player;

use module::Module;
use player::{Options, PlayerListEntry, PlayerInfo, FormatPlayer};
use ::*;

pub struct Ft101;

impl PlayerListEntry for Ft101 {
   fn info(&self) -> PlayerInfo {
       PlayerInfo {
          id         : "ft",
          name       : "FastTracker 1.01",
          description: "Based on the FastTracker 1.01 replayer",
          author     : "Claudio Matsuoka",
          accepts    : &[ "m.k.", "xchn" ],
       }
   }

   fn player(&self, module: &Module, options: Options) -> Box<dyn FormatPlayer> {
       Box::new(self::player::FtPlayer::new(module, options))
   }

   fn import(&self, module: Module) -> Result<Module, Error> {
       Ok(module)
   }
}


