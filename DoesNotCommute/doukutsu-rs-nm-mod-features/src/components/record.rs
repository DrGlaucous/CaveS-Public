use std::io::{Cursor, Read, Write};
use crate::bitfield;

use byteorder::{LE, ReadBytesExt, WriteBytesExt};

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
//use crate::graphics::font::Font;


//doesn't record keypresses, but instead records raw player values for each frame (more expensive, but simpler on the upshot)
//one instance of this will exist for the player, and one for each NPC that needs active readback

const MAGIC: &str = "NPR"; //NPC Recording

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct RecordStateFlags(u8);
    impl Debug;

    pub trigger_frame, set_trigger_frame: 0; // 0x01
    pub shock_frame, set_shock_frame: 1; // 0x02

}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RecordState {
    Idle,
    Recording,
    Playing,
}

#[derive(Debug, Clone, Copy)]
pub struct RecordFrame {
    flags: RecordStateFlags,
    current_weapon: u8,
    x: i32,
    y: i32,
    anim_num: u16,
}


#[derive(Debug, Clone)]
pub struct Record {
    record_version: u16,
    frame_list: Vec<RecordFrame>,
    index: usize,
    current_frame: Option<RecordFrame>,
    record_state: RecordState,
    last_record_state: RecordState, //for pausing
    tick: usize,
    resume_tick: usize,
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
            tick: 0,
            resume_tick: 0,
        }
    }

    //may be redundant (could set this variable directly...) I guess this stops us from trying to record in playback mode (and vice-versa)
    pub fn start_recording(&mut self) {
        if self.record_state == RecordState::Idle
        && self.last_record_state == RecordState::Idle
        {
            self.record_state = RecordState::Recording;
        }
    }

    //to pause these, set record_state to RecordState::Idle;
    pub fn stop_recording(
        &mut self,
        ctx: &mut Context,
        filename: &mut str,
    ) -> GameResult {
        self.record_state = RecordState::Idle;
        self.last_record_state = RecordState::Idle;

        self.write_replay(ctx, format!{"/{}", filename}.as_str())?;


        Ok(())
    }

    pub fn start_playback(
        &mut self,
        ctx: &mut Context,
        filename: &mut str,
    ) -> GameResult {
        if self.record_state == RecordState::Idle 
        && self.last_record_state == RecordState::Idle
        {
            self.index = 0;
            self.read_replay(ctx, format!{"/{}", filename}.as_str())?;
            self.record_state = RecordState::Playing;
        }
        Ok(())
    }
    pub fn stop_playback(
        &mut self,
    ) {
        self.record_state = RecordState::Idle;
        self.last_record_state = RecordState::Idle;
    }

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
    pub fn get_state(&self) -> RecordState {
        self.record_state
    }

    pub fn get_frame(&self) -> Option<RecordFrame> {
        self.current_frame
    }

    //push stored record out to file
    fn write_replay(&mut self, ctx: &mut Context, custom_filename: &str) -> GameResult {

        //[state.get_rec_filename(), replay_kind.get_suffix()].join("")
        let path = format!{"/{}", custom_filename};

        if let Ok(mut file) = filesystem::open_options(
            ctx,
            path,
            OpenOptions::new().write(true).create(true),
        ) {
            file.write_all(MAGIC.as_bytes())?;

            file.write_u16::<LE>(self.record_version)?;


            for input in &self.frame_list {

                // flags: RecordStateFlags,
                // weapon_index: u16,
                // x: f32,
                // y: f32,
                // anim_no: u16,

                file.write_u8(input.flags.0)?;
                file.write_u8(input.current_weapon)?;
                file.write_i32::<LE>(input.x)?;
                file.write_i32::<LE>(input.y)?;
                file.write_u16::<LE>(input.anim_num)?;

            }
        }
        Ok(())
    }
    //get stored record from file
    fn read_replay(&mut self, ctx: &mut Context, custom_filename: &str) -> GameResult {

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

            let count = data.len() / 2;
            let mut inputs = Vec::new();
            let mut f = Cursor::new(data);


            for _ in 0..count {

                // file.write_u8(input.flags.0)?;
                // file.write_u16::<LE>(input.weapon_index)?;
                // file.write_f32::<LE>(input.x)?;
                // file.write_f32::<LE>(input.y)?;
                // file.write_u16::<LE>(input.anim_no)?;
                //let ttt = RecordStateFlags{0: 5}; //another way to initialize the bifiteld

                inputs.push(
                    RecordFrame{
                        flags: RecordStateFlags(f.read_u8()?),
                        current_weapon: f.read_u8()?,
                        x: f.read_i32::<LE>()?,
                        y: f.read_i32::<LE>()?,
                        anim_num: f.read_u16::<LE>()?,
                    }
                );

                //inputs.push(f.read_u16::<LE>()?);
            }

            self.frame_list = inputs;
        }
        Ok(())
    }


    //automatically takes the variables out of the player struct and packages a RecordFrame
    //because we can't do it in tick since we get multi-borrow errors
    pub fn extract_player_rec_frame(player: &Player) -> RecordFrame {
        let mut flags = RecordStateFlags(0);
        flags.set_shock_frame(player.shock_counter / 2 % 2 != 0);
        flags.set_trigger_frame(player.controller.trigger_shoot());

        RecordFrame {
            flags: flags,
            current_weapon: player.current_weapon,
            x: player.x,
            y: player.y,
            anim_num: player.anim_num,
        }
    }

}

//custom args: player and NPC
impl GameEntity<(Option<RecordFrame>)> for Record {

    fn tick(&mut self, _state: &mut SharedGameState, frame_to_save: Option<RecordFrame>) -> GameResult {

        match self.record_state {
            RecordState::Idle => {},
            RecordState::Playing => {
                self.current_frame = if let Some(frame) = self.frame_list.get(self.index) {
                    self.index += 1;
                    Some(frame.clone())
                } else { //finished, halt reader
                    self.stop_playback();
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



