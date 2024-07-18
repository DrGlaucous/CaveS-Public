use std::cell::{RefCell};
use crate::common::{Direction, Rect};
use crate::entity::GameEntity;
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponType, WeaponLevel, TargetShooter};
use crate::util::rng::RNG;
use crate::game::npc::NPCList;
use crate::game::weapon::bullet::BulletManager;



impl NPC {

    //reads formatted record frames from a file and immitates the player
    pub(crate) fn tick_n371_fake_pc(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        bullet_manager: &mut BulletManager,
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


        //make sure we've bee initialized
        if self.child_ids.len() == 0 {

            let npc = NPC::create(372, &state.npc_table);
            let other_id = npc_list.spawn(min_id, npc)
            let us = npc_list.get_npc(self.id).unwrap();

            //return Ok(())
        }



        //automatically initialize our weapon
        if self.weapon.is_none() {
           self.weapon = Some(Weapon::new(WeaponType::None, WeaponLevel::Level1, 0, 0, 0));
        }



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
                        self.vel_x = self.x; //use old positions to derive veloctiy
                        self.vel_y = self.y;
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


                        self.shooter_vals.shoot = frame.flags.shoot();
                        self.shooter_vals.trigger_shoot = frame.flags.trigger_shoot();
                        self.shooter_vals.cond = self.cond;
                        self.shooter_vals.x = self.x;
                        self.shooter_vals.y = self.y;
                        //velocity is derived from delta D
                        self.shooter_vals.vel_x = self.x - self.vel_x;
                        self.shooter_vals.vel_y = self.y - self.vel_y;
                        //todo: equip
                        self.shooter_vals.direction = self.direction;
                        self.shooter_vals.up = frame.flags.up();
                        self.shooter_vals.down = frame.flags.down();
                        //stars variable doesn't need set


                        // if self.shooter_vals.trigger_shoot {
                        //     let mut adam = self.x;
                        //     let mut eve = self.x + adam;
                        // }
                        //npc_list.spawn(min_id, npc)

                        
                        
                        //update weapon
                        if let Some(mut weapon) = self.weapon.take() {

                            let eve_num = self.event_num as u32;
                            weapon.tick(state, self, TargetShooter::NPC(eve_num), bullet_manager);

                            weapon.wtype = frame.weapon;
                            weapon.level = frame.weapon_level;
                            weapon.ammo = frame.ammo;
                            weapon.max_ammo = frame.max_ammo;

                            //give it back
                            self.weapon = Some(weapon);
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

        /*
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
        };*/

        //don't render unless we've got a skin to render from or is not animation number 0
        self.anim_rect = match (&self.pc_skin, self.action_num != 0) {
            
            (Some(skin), true) => {
                //ensure the display box is correct to match the metadata
                let rc = skin.metadata.display_box;
                self.display_bounds = Rect::new(
                    rc.left as u32 * 0x200,
                    rc.top as u32 * 0x200,
                    rc.right as u32 * 0x200,
                    rc.bottom as u32 * 0x200,
                );
                skin.get_anim_rect(self.anim_num, dir_offset)
            }
            _ => {
                //Rect::new(0,0,16,16)
                Rect::new(0,0,0,0)
            }

        };


        Ok(())
    }


    pub(crate) fn tick_n372_fake_pc_gun(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        bullet_manager: &mut BulletManager,
    ) -> GameResult {

        let weapon_type = WeaponType::from_u8(self.action_num as u8);

        let weapon_rect = Player::get_weapon_rect(
            self.action_num as u8,
            false,
            self.direction,
            self.shooter_vals.up,
            self.shooter_vals.down,
        );

        if let Some(npc) = npc_list.get_npc(self.parent_id as usize) {
        }

        Ok(())
    }








}







