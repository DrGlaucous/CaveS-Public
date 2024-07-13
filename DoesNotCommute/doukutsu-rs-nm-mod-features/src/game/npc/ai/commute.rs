use crate::common::{Direction, Rect};
use crate::entity::GameEntity;
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::util::rng::RNG;
use crate::game::npc::NPCList;


fn ttf(npc: &mut NPC, state: &mut SharedGameState) {
    npc.x += 2;
    npc.tick_n021_chest_open(state);
}


impl NPC {
    pub(crate) fn tick_n371_fake_pc(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {

        /*
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                let player = self.get_closest_pseudo_player_mut(players, npc_list);
                if (self.x - player.x()).abs() < 0x4000
                    && self.y - 0x4000 < player.y()
                    && self.y + 0x2000 > player.y() {
                    self.direction = if self.x > player.x() { Direction::Left } else { Direction::Right };
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            3 | 4 => {
                if self.action_num == 3 {
                    self.action_num = 4;
                    self.anim_num = 2;
                    self.anim_counter = 0;
                }

                self.animate(4, 2, 5);

                self.x += self.direction.vector_x() * 0x200;
            }
            10 => {
                self.anim_num = 6;

                self.action_counter += 1;
                if self.action_counter > 200 {
                    self.action_counter = 0;

                    state.create_caret(self.x, self.y, CaretType::Zzz, Direction::Left);
                }
            }
            _ => (),
        }
        */


        //ttf(self, state);

        match self.action_num {
            
            //start recorder + run recorder
            1 | 2
            => {
                if let Some(recorder) = &mut self.recorder {
                    
                    //start
                    if self.action_num == 1 {
                        self.action_num += 1;
                        recorder.start_playback();
                    }
                    //run
                    //do readback here
                    recorder.tick(state, None)?;

                    if let Some(frame) = recorder.get_frame(){
                        self.x = frame.x;
                        self.y = frame.y;
                        self.anim_num = frame.anim_num;
                        self.direction = Direction::from_int(frame.direct as usize).unwrap();

                        //play sounds (this looks nasty)
                        {
                            if frame.sound_flags.jump_15() {
                                state.sound_manager.play_sfx(15);
                            }
                            if frame.sound_flags.hurt_16() {
                                state.sound_manager.play_sfx(16);
                            }
                            if frame.sound_flags.die_17() {
                                state.sound_manager.play_sfx(17);
                            }
                            if frame.sound_flags.walk_24() {
                                state.sound_manager.play_sfx(24);
                            }
                            if frame.sound_flags.splash_56() {
                                state.sound_manager.play_sfx(56);
                            }
                            if frame.sound_flags.booster_113() {
                                state.sound_manager.play_sfx(113);
                            }
                        }

                    } else {
                        //record finished, return to idle
                        self.action_num = 0;
                    }



                }


            }
            //idle
            0 | _ => {}
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 1 };

        //don't render unless we've got a skin to render from
        self.anim_rect = if let Some(skin) = &self.pc_skin {

            //ensure the display box is correct to match the metadata
            let rc = skin.metadata.display_box;
            self.display_bounds = Rect::new(
                rc.left as u32 * 0x200,
                rc.top as u32 * 0x200,
                rc.right as u32 * 0x200,
                rc.bottom as u32 * 0x200,
            );


            skin.get_anim_rect(self.anim_num, dir_offset)
        } else {
            Rect::new(0,0,16,16)
        };


        Ok(())
    }









}







