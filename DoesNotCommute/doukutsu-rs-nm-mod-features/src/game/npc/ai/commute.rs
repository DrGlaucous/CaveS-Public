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

    //manages NPC sub-parts,
    //reads formatted record frames from a file and immitates the player
    pub(crate) fn tick_n371_fake_pc_manager(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        bullet_manager: &mut BulletManager,
    ) -> GameResult {


        //find NPC of type and return its id in a list of children
        fn find_npc(
            id_list: &Vec<u16>,
            npc_list: &NPCList,
            npc_type: u16,
        ) -> Option<u16> {

            for u in id_list.as_slice()
            {
                if let Some(npc) = npc_list.get_npc(*u as usize)
                {
                    if npc.npc_type == npc_type //matches NPC type
                    {
                        return Some(*u);
                    }
                }
            }

            None
        }


        //initialize sub-npc parts
        if self.child_ids.len() == 0 {

            //create body
            let mut body = NPC::create(372, &state.npc_table);
            body.cond.set_alive(true);
            body.parent_id = self.id;

            //create gun
            let mut gun = NPC::create(373, &state.npc_table);
            gun.cond.set_alive(true);
            gun.parent_id = self.id;

            if let Ok(body_id) = npc_list.spawn(0x100, gun)
            {self.child_ids.push(body_id);} //gun is index 0
            if let Ok(gun_id) = npc_list.spawn(0x100, body)
            {self.child_ids.push(gun_id);} //body is index 1

            //return Ok(())
        }


        //initialize weapon
        if self.more_items.weapon.is_none() {
           self.more_items.weapon = Some(Weapon::new(WeaponType::None, WeaponLevel::Level1, 0, 0, 0));
        }



        match self.action_num {
            
            //start recorder + run recorder
            1 | 2
            => {
                //check for sub-npcs and recorder:
                if let Some(recorder) = &mut self.more_items.recorder {  
                    
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
                        self.anim_counter = if frame.flags.shock_frame() {1} else {0};
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


                        //required for the weapon
                        self.more_items.shooter_vals.shoot = frame.flags.shoot();
                        self.more_items.shooter_vals.trigger_shoot = frame.flags.trigger_shoot();
                        self.more_items.shooter_vals.cond = self.cond;


                        //let skin_offset = if let Some(skin) = self.more_items.pc_skin {
                        //    (skin.metadata.gun_offset_x as i32, skin.metadata.gun_offset_y as i32)
                        //} else {(0,0)};
                        self.more_items.shooter_vals.x = self.x;// + skin_offset.0 * 0x200;
                        self.more_items.shooter_vals.y = self.y;// + skin_offset.1 * 0x200;



                        //velocity is derived from delta D
                        self.more_items.shooter_vals.vel_x = self.x - self.vel_x;
                        self.more_items.shooter_vals.vel_y = self.y - self.vel_y;
                        //todo: equip
                        self.more_items.shooter_vals.direction = self.direction;
                        self.more_items.shooter_vals.up = frame.flags.up();
                        self.more_items.shooter_vals.down = frame.flags.down();
                        //stars variable doesn't need set

                        //update peripherals
                        if let (
                            Some(gun),
                            Some(body),
                            Some(mut weapon),
                
                        ) = (
                            npc_list.get_npc(self.child_ids[0] as usize),
                            npc_list.get_npc(self.child_ids[1] as usize),
                            self.more_items.weapon.take(), //taking this so we can tick it (while feeding it "self")
                        ) {
                            //set sub-part rects and positions
                            {
                                //give our skin metadata to our "body" child, ensures any new skins set via TSC get passed down
                                if let Some(skin) = self.more_items.pc_skin.take() {
                                    body.more_items.pc_skin = Some(skin);
                                }
                    
                    
                                let dir_offset = if self.direction == Direction::Left { 0 } else { 1 };
                    
                                //don't render unless we've got a skin to render from or is not animation number 0 (idle state) or are NOT in a shock state (anim_counter is NOT 0)
                                //note: final condition may need to be moved inside to help with the interpolation mode
                                match (&body.more_items.pc_skin, self.action_num != 0, self.anim_counter == 0) {
                                    
                                    (Some(skin), true, true) => {
                    
                                        //set body rect and position
                                        {
                                            //ensure the display box is correct to match the metadata
                                            let rc = skin.metadata.display_box;
                                            body.display_bounds = Rect::new(
                                                rc.left as u32 * 0x200,
                                                rc.top as u32 * 0x200,
                                                rc.right as u32 * 0x200,
                                                rc.bottom as u32 * 0x200,
                                            );
                                            body.anim_rect = skin.get_anim_rect(self.anim_num, dir_offset);
                                            body.x = self.x;
                                            body.y = self.y;
                                        }
                    
                                        //set gun rect and position
                                        {
                                            let (rc, y_offset) = Player::get_weapon_rect(
                                                weapon.wtype as u8,
                                                self.anim_num == 1 || self.anim_num == 2 || self.anim_num == 4 || self.anim_num == 5 || self.anim_num == 6,
                                                self.direction,
                                                self.more_items.shooter_vals.up,
                                                self.more_items.shooter_vals.down
                                            );
                                            gun.display_bounds = Rect::new(
                                                0,
                                                0,
                                                rc.width() as u32 * 0x200,
                                                rc.height() as u32 * 0x200,
                                            );
                    
                                            let (gun_off_x, gun_off_y) = if let Some(skin) = &mut body.more_items.pc_skin {
                                                (skin.metadata.gun_offset_x as i32 * 0x200, skin.metadata.gun_offset_y as i32 * 0x200)
                                            } else {(0,0)};
                    
                                            gun.anim_rect = rc;
                    
                                            gun.x = self.x
                                            + if self.direction == Direction::Left { - (rc.width() as i32 * 0x200) - gun_off_x} else { gun_off_x};
                    
                                            gun.y = self.y + (y_offset as i32 * 0x200) + gun_off_y;
                    
                                            self.more_items.shooter_vals.gun_offset_x = gun.x;
                                            self.more_items.shooter_vals.gun_offset_y = gun.y;
                    
                                        }
                    
                    
                    
                                    }
                                    _ => {
                                        //Rect::new(0,0,16,16)
                                        body.anim_rect = Rect::new(0,0,0,0);
                    
                                        gun.anim_rect = Rect::new(0,0,0,0);
                                    }
                                }
                        
                            }

                            //update weapon
                            {
                                let eve_num = self.event_num as u32;
                                weapon.tick(state, self, TargetShooter::NPC(eve_num), bullet_manager);
    
                                weapon.wtype = frame.weapon;
                                weapon.level = frame.weapon_level;
                                weapon.ammo = frame.ammo;
                                weapon.max_ammo = frame.max_ammo;
    
                                //give it back
                                self.more_items.weapon = Some(weapon);
                            }
                        
                        }
                

                    } else {
                        //record finished, return to idle
                        self.action_num = 0;
                    }

                }


            }
            //idle
            0 | _ => {

                //rewind recorder
                if self.action_num == 3 {
                    self.action_num = 0;
                    if let Some(recorder) = &mut self.more_items.recorder {
                        recorder.index = 0;
                    }
                }

                //hide peripherals
                if let (
                    Some(gun),
                    Some(body),
        
                ) = (
                    npc_list.get_npc(self.child_ids[0] as usize),
                    npc_list.get_npc(self.child_ids[1] as usize),
                ) {
                    body.anim_rect = Rect::new(0,0,0,0);
                    gun.anim_rect = Rect::new(0,0,0,0);
                }
                
            }
        }

        //may not be needed; hide parent NPC
        self.anim_rect = Rect::new(0,0,0,0);


        Ok(())

    }


    //main part: is the fPC's body/gun
    pub(crate) fn tick_n372_n373_fake_pc_sub(
        &mut self,
    ) -> GameResult {

        Ok(())
    }



    





}







