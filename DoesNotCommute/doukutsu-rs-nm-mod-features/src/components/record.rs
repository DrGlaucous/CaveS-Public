use std::any::Any;
use std::io::{Cursor, Read, Write};
use std::mem::size_of;
use crate::bitfield;
use crate::framework::error::GameError;

use byteorder::{LE, ReadBytesExt, WriteBytesExt};

use crate::common::Direction;
use crate::entity::GameEntity;
use crate::framework::context::Context;
use crate::framework::error::{GameResult, GameError::ResourceLoadError};
use crate::framework::filesystem;
//use crate::framework::keyboard::ScanCode;
use crate::framework::vfs::OpenOptions;
use crate::game::frame::Frame;
use crate::game::shared_game_state::SharedGameState; //{ReplayKind, ReplayState, SharedGameState};
//use crate::input::replay_player_controller::{KeyState, ReplayController};
use crate::game::player::Player;
use crate::game::npc::NPC;
use crate::game::inventory::Inventory;
use crate::game::weapon::{self, WeaponLevel, WeaponType};
//use crate::graphics::font::Font;


//doesn't record keypresses, but instead records raw player values for each frame (more expensive, but simpler on the upshot)
//one instance of this will exist for the player, and one for each NPC that needs active readback

const MAGIC: &str = "NPR"; //NPC Recording

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct RecordStateFlags(u8);
    impl Debug;

    pub shoot, set_shoot: 0; // 0x01
    pub trigger_shoot, set_trigger_shoot: 1; // 0x01
    pub shock_frame, set_shock_frame: 2; // 0x02
    pub up, set_up: 3; // 0x04
    pub down, set_down: 4; // 0x08

    //special options for booster (direction of heading only; we re-use the sound flag for boost boolean)
    pub boost_a, set_boost_a: 5; //0x16
    pub boost_b, set_boost_b: 6; //0x32

}

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct SoundFlags(u8);
    impl Debug;

    pub jump_15, set_jump_15: 0; // 0x01 //jump
    pub hurt_16, set_hurt_16: 1; // 0x02 //hurt
    pub die_17, set_die_17: 2; // 0x04 //die
    pub walk_24, set_walk_24: 3; // 0x08 //walk
    pub splash_56, set_splash_56: 4; // 0x10 //splash
    pub booster_113, set_booster_113: 5; // 0x20 //booster

}



#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RecordState {
    Idle,
    Recording,
    Playing,
}

#[derive(Debug, Clone, Copy)]
#[repr(packed(1))]
pub struct RecordFrame {
    pub flags: RecordStateFlags, //1
    pub weapon: WeaponType, //1
    pub weapon_level: WeaponLevel, //1
    pub ammo: u16, //2
    pub max_ammo: u16, //2
    pub x: i32, //4
    pub y: i32, //4
    pub anim_num: u16, //2
    pub direct: u8, //1
    pub sound_flags: SoundFlags, //1
    //3+4+8+2+2=19
}


#[derive(Debug, Clone)]
pub struct Record {
    record_version: u16,
    pub frame_list: Vec<RecordFrame>,
    pub index: usize, //set this to 0 to rewind player
    current_frame: Option<RecordFrame>,
    record_state: RecordState,
    last_record_state: RecordState, //for pausing
}

impl Record {
    pub fn new() -> Record {
        Record {
            record_version: 1,
            frame_list: Vec::new(),
            index: 0,
            current_frame: None,
            record_state: RecordState::Idle,
            last_record_state: RecordState::Idle,
        }
    }

    //puts the recorder in record mode. mode cannot be changed untill stop_recorder is called
    pub fn start_recording(&mut self) {
        if self.record_state == RecordState::Idle
        && self.last_record_state == RecordState::Idle
        {
            self.record_state = RecordState::Recording;
        }
    }

    //resets playback cursor and begins iterating through loaded file (does NOT load file)
    pub fn start_playback(&mut self) {
        if self.record_state == RecordState::Idle 
        && self.last_record_state == RecordState::Idle
        {
            self.record_state = RecordState::Playing;
        }
    }

    //halts the recorder with the intent of changing modes
    pub fn stop_recorder(
        &mut self,
    ) {
        self.record_state = RecordState::Idle;
        self.last_record_state = RecordState::Idle;
    }

    //pauses the recorder with the intent of picking up where you left off (enforces same mode)
    pub fn pause_recorder(&mut self) {
        if self.record_state != RecordState::Idle {
            self.last_record_state = self.record_state;
            self.record_state = RecordState::Idle;
        }
    }
    pub fn resume_recorder(&mut self) {
        if self.record_state == RecordState::Idle {
            self.record_state = self.last_record_state;
            self.last_record_state = RecordState::Idle;
        }
    }
 
    //is the recorder paused? recording? playing?
    pub fn get_state(&self) -> RecordState {
        self.record_state
    }
    //get current frame of the recorder when in playback mode
    pub fn get_frame(&self) -> Option<RecordFrame> {
        self.current_frame
    }

    //push stored record out to file
    pub fn write_replay(&mut self, ctx: &mut Context, custom_filename: &str) -> GameResult {

        //[state.get_rec_filename(), replay_kind.get_suffix()].join("")
        let path = format!{"/{}", custom_filename};

        match filesystem::open_options(
            ctx,
            path,
            OpenOptions::new().write(true).truncate(true).create(true),
        ) {
            Ok(mut file) => {
                file.write_all(MAGIC.as_bytes())?;

                file.write_u16::<LE>(self.record_version)?;
    
    
                for input in &self.frame_list {

                    file.write_u8(input.flags.0)?;
                    file.write_u8(input.weapon as u8)?;
                    file.write_u8(input.weapon_level as u8)?;
                    file.write_u16::<LE>(input.ammo)?;
                    file.write_u16::<LE>(input.max_ammo)?;
                    file.write_i32::<LE>(input.x)?;
                    file.write_i32::<LE>(input.y)?;
                    file.write_u16::<LE>(input.anim_num)?;
                    file.write_u8(input.direct)?;
                    file.write_u8(input.sound_flags.0)?;
    
                }
            },
            Err(e) => {
                log::warn!("ERR: {}", e);
            }
        }

        Ok(())
    }
    //get stored record from file
    pub fn read_replay(&mut self, ctx: &mut Context, custom_filename: &str) -> GameResult {

        //[state.get_rec_filename(), replay_kind.get_suffix()].join("")
        let path = format!{"/{}", custom_filename};

        if let Ok(mut file) = filesystem::user_open(ctx, path)
        {
            let mut magic_buf = [0 as u8; 3];
            file.read_exact(&mut magic_buf)?;
            if &magic_buf != MAGIC.as_bytes() {
                return Err(ResourceLoadError("Invalid magic".to_owned()));
            }

            self.record_version = file.read_u16::<LE>()?;


            let mut data = Vec::new();
            file.read_to_end(&mut data)?;

            let size = size_of::<RecordFrame>();
            let count = data.len() / size;
            
            let mut inputs = Vec::new();
            let mut f = Cursor::new(data);

            let mut tt = || -> Result<(), GameError> {
                for _ in 0..count {

                    // file.write_u8(input.flags.0)?;
                    // file.write_u8(input.weapon_index)?;
                    // file.write_f32::<LE>(input.x)?;
                    // file.write_f32::<LE>(input.y)?;
                    // file.write_u16::<LE>(input.anim_no)?;
                    //let ttt = RecordStateFlags{0: 5}; //another way to initialize the bifiteld
                    
                    let flags = f.read_u8()?;

                    inputs.push(
                        RecordFrame{
                            flags: RecordStateFlags(flags),
                            weapon: WeaponType::from_u8(f.read_u8()?),
                            weapon_level: WeaponLevel::from_u8(f.read_u8()?),
                            ammo: f.read_u16::<LE>()?,
                            max_ammo: f.read_u16::<LE>()?,
                            x: f.read_i32::<LE>()?,
                            y: f.read_i32::<LE>()?,
                            anim_num: f.read_u16::<LE>()?,
                            direct: f.read_u8()?,
                            sound_flags: SoundFlags(f.read_u8()?),
                        }
                    );

                    //inputs.push(f.read_u16::<LE>()?);
                }
                Ok(())
                
            };
            if let Err(t) = tt() {
                log::warn!("ERR {}", t);
            }

            self.frame_list = inputs;
        }
        Ok(())
    }


    //automatically takes the variables out of the player struct and packages a RecordFrame
    //because we can't do it in "tick" since we get multi-borrow errors
    pub fn extract_player_rec_frame(player: &Player, inventory: &Inventory) -> RecordFrame {
        let mut flags = RecordStateFlags(0);
        flags.set_shock_frame(player.shock_counter / 2 % 2 != 0);
        flags.set_shoot(player.controller.shoot());
        flags.set_trigger_shoot(player.controller.trigger_shoot());
        flags.set_up(player.up);
        flags.set_down(player.down);

        match player.booster_dir {
            Direction::Left => {
                flags.set_boost_a(false);
                flags.set_boost_b(false);
            },
            Direction::Right => {
                flags.set_boost_a(true);
                flags.set_boost_b(false);
            },
            Direction::Up => {
                flags.set_boost_a(false);
                flags.set_boost_b(true);
            },
            Direction::Bottom => {
                flags.set_boost_a(true);
                flags.set_boost_b(true);
            },
            _ => {},
        }

        //flags.set_boost_b(true);

        // if player.anim_num == 11 {
        //     let mut da = flags.0;
        //     da += 1;
        // }
        let (
            weapon_type,
            weapon_lvl,
            weapon_ammo,
            weapon_max_ammo,
        ) = if let Some(weapon) = inventory.get_current_weapon() {
            (
                weapon.wtype,
                weapon.level,
                weapon.ammo,
                weapon.max_ammo
            )
        } else {
            (
                WeaponType::None,
                WeaponLevel::None,
                0,
                0,
            )
        };

        RecordFrame {
            flags: flags,
            weapon: weapon_type,
            weapon_level: weapon_lvl,
            ammo: weapon_ammo,
            max_ammo: weapon_max_ammo,
            x: player.x,
            y: player.y,
            anim_num: player.skin.get_raw_frame_index(),
            direct: player.direction.as_int() as u8,
            sound_flags: player.sound_flags,
        }
    }

}

//custom args: player and NPC
impl GameEntity<Option<RecordFrame>> for Record {

    fn tick(&mut self, _state: &mut SharedGameState, frame_to_save: Option<RecordFrame>) -> GameResult {

        match self.record_state {
            RecordState::Idle => {},
            RecordState::Playing => {
                self.current_frame = if let Some(frame) = self.frame_list.get(self.index) {
                    self.index += 1;
                    Some(frame.clone())
                } else { //finished, halt reader
                    self.stop_recorder();
                    None
                };
            },
            RecordState::Recording => {
                //only record if we passed in a frame to save
                if let Some(frame) = frame_to_save {
                    self.frame_list.push(frame);
                }
            },
        }

        /* 
        match state.replay_state {
            ReplayState::Recording => {
                // This mimics the KeyState bitfield
                let inputs = player.controller.move_left() as u16
                    + ((player.controller.move_right() as u16) << 1)
                    + ((player.controller.move_up() as u16) << 2)
                    + ((player.controller.move_down() as u16) << 3)
                    + ((player.controller.trigger_map() as u16) << 4)
                    + ((player.controller.trigger_inventory() as u16) << 5)
                    + (((player.controller.jump() || player.controller.trigger_menu_ok()) as u16) << 6)
                    + (((player.controller.shoot() || player.controller.trigger_menu_back()) as u16) << 7)
                    + ((player.controller.next_weapon() as u16) << 8)
                    + ((player.controller.prev_weapon() as u16) << 9)
                    + ((player.controller.trigger_menu_ok() as u16) << 11)
                    + ((player.controller.skip() as u16) << 12)
                    + ((player.controller.strafe() as u16) << 13);

                self.keylist.push(inputs);
            }
            ReplayState::Playback(_) => {
                let pause = ctx.keyboard_context.is_key_pressed(ScanCode::Escape) && (self.tick - self.resume_tick > 3);

                let next_input = if pause { 1 << 10 } else { *self.keylist.get(self.tick).unwrap_or(&0) };

                self.controller.state = KeyState(next_input);
                self.controller.old_state = self.last_input;
                player.controller = Box::new(self.controller);

                if !pause {
                    self.last_input = KeyState(next_input);
                    self.tick += 1;
                } else {
                    self.resume_tick = self.tick;
                };

                if self.tick >= self.keylist.len() {
                    state.replay_state = ReplayState::None;
                    player.controller = state.settings.create_player1_controller();
                }
            }
            ReplayState::None => {}
        }
        */

        Ok(())
    }

    //nothing to do here...
    fn draw(&self, _state: &mut SharedGameState, _ctx: &mut Context, _frame: &Frame) -> GameResult {
        // let x = state.canvas_size.0 - 32.0;
        // let y = 8.0 + if state.settings.fps_counter { 12.0 } else { 0.0 };

        // match state.replay_state {
        //     ReplayState::None => {}
        //     ReplayState::Playback(_) => {
        //         state.font.builder()
        //             .position(x, y)
        //             .draw("PLAY", ctx, &state.constants, &mut state.texture_set)?;
        //     }
        //     ReplayState::Recording => {
        //         state.font.builder()
        //             .position(x, y)
        //             .draw("REC", ctx, &state.constants, &mut state.texture_set)?;
        //     }
        // }

        Ok(())
    }



}



