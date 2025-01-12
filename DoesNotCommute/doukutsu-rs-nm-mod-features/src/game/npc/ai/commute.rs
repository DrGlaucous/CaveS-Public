use std::any::Any;
use std::borrow::Borrow;
use std::cell::{RefCell};
use std::f32::consts::PI;
use crate::common::{Direction, Rect, CDEG_RAD, Color};
use crate::entity::GameEntity;
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::{NPCLayer, NPCLightType, NPC};
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponType, WeaponLevel, TargetShooter};
use crate::scene::game_scene;
use crate::util::rng::RNG;
use crate::game::npc::NPCList;
use crate::game::weapon::bullet::BulletManager;
use crate::game::stage::Stage;
use crate::game::frame::Frame;
//use crate::game::npc::ai::misc;


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

                        //make boost particles
                        if frame.sound_flags.booster_113() {
                            let smoke_dir = match (frame.flags.boost_a(), frame.flags.boost_b()) {
                                (false, false) => Direction::Left,
                                (true, false) => Direction::Right,
                                (false, true) => Direction::Up,
                                _ => Direction::Bottom, //TT
                            }.opposite();

                            state.create_caret(
                                self.x,
                                self.y + self.hit_bounds.bottom as i32 / 2,
                                CaretType::Exhaust,
                                smoke_dir,
                            );

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
                        
                            //update lighting
                            {
                                self.light_options.light_angle = match () {
                                    _ if self.more_items.shooter_vals.up => 60..120,
                                    _ if self.more_items.shooter_vals.down => 240..300,
                                    _ if self.more_items.shooter_vals.direction == Direction::Left => -30..30,
                                    _ if self.more_items.shooter_vals.direction == Direction::Right => 150..210,
                                    _ => 0..0,
                                };

                                let (color, power) = match frame.weapon {
                                    WeaponType::Fireball => ((170u8, 80u8, 0u8), 0.92),
                                    WeaponType::PolarStar => ((150u8, 150u8, 160u8), 0.92),
                                    WeaponType::Spur => ((170u8, 170u8, 200u8), 0.92),
                                    WeaponType::Blade | WeaponType::None => ((0u8, 0u8, 0u8), 0.0),
                                    _ => ((150u8, 150u8, 150u8), 0.92),
                                };
                                self.light_options.light_color = Color::from(color);
                                self.light_options.light_power = power;

                                self.light_options.x = self.x;
                                self.light_options.y = self.y;
                                self.light_options.prev_x = self.prev_x;
                                self.light_options.prev_y = self.prev_y;

                                self.light_options.light_type = NPCLightType::Cone;


                            }
                        }
                

                    } else {
                        //record finished, return to idle
                        self.action_num = 0;
                    }

                }


            }
            //idle + rewind recorder
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
                

                //mute light
                self.light_options.light_type = NPCLightType::None;
            }
        }


        //may not be needed; hide parent NPC
        self.anim_rect = Rect::new(0,0,0,0);


        Ok(())

    }


    //sub-part: is the fPC's body/gun
    pub(crate) fn tick_n372_n373_fake_pc_sub(
        &mut self,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {

        // //debug the closeness functions
        // let rc = Rect::new(168, 96, 192, 112);
        // let rc2 = Rect::new(168, 112, 192, 128);

        // let mut aa = 0;
        // aa += 1;


        // let player = self.get_closest_pseudo_player_mut(players, npc_list);
        // self.face_player(player);

        // if self.direction == Direction::Left {
        //     self.anim_rect = rc;
        // } else {
        //     self.anim_rect = rc2;
        // }

        Ok(())
    }


    //will switch between the list of active commuter NPCs given player input
    pub(crate) fn tick_n374_pc_switcher(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {

        //search for commuter NPCs to add to the list
        if self.child_ids.len() == 0 {
            for npc in npc_list.iter() {
                if npc.npc_type == 371 {
                    self.child_ids.push(npc.id);
                }
            }
            self.target_y = 0; //"no-follow" mode
            return Ok(());
        }

        //check for any NPCs not in "idle" mode.
        let mut some_active = false;
        for idx in &self.child_ids {
            if let Some(npc) = npc_list.get_npc(*idx as usize) {
                if npc.action_num != 0 {
                    some_active = true;
                    continue;
                }
            }

        }

        //run event when this button is pressed or if all npcs are idle (how we "skip" replays)
        if (players[0].controller.trigger_jump() && self.direction == Direction::Left)
        || !some_active {
            state.textscript_vm.start_script(self.event_num);
        }


        //switch observed index
        if players[0].controller.trigger_prev_weapon() {
            self.target_x -= 1;
            self.target_y = 1; //put into "follow" mode
        } else if players[0].controller.trigger_next_weapon() {
            self.target_x += 1;
            self.target_y = 1;
        }

        //wrapping
        if self.target_x < 0 {self.target_x = (self.child_ids.len() - 1) as i32}
        if self.target_x >= self.child_ids.len() as i32 {self.target_x = 0}

        //snap to this NPC's location
        if self.target_y != 0 {
            if let Some(npc) = npc_list.get_npc(self.child_ids[self.target_x as usize] as usize) {
                self.x = npc.x;
                self.y = npc.y;
                self.anim_rect = Rect::new(224,240, 256, 272);
                self.layer = NPCLayer::Foreground;
            }
        } else {
            self.anim_rect = Rect::new(0,0,0,0);
        }

        Ok(())
    }



    //adds its event number in seconds to the player's counter
    pub(crate) fn tick_n375_time_collectible(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {

        //increment animation number
        self.anim_counter += 1;
        if self.anim_counter > 2 {
            self.anim_counter = 0;
            self.anim_num += 1;
        }

        //useing the action counter to determine if the clock should be in despawn mode
        if self.action_counter2 > 0 {
            
            self.action_counter += 1;
            if self.action_counter >= self.action_counter3 {
                //turn into smoke
                let mut smoke = NPC::create(4, &state.npc_table);
                smoke.cond.set_alive(true);
                (smoke.x, smoke.y) = (self.x, self.y);
                let _ = npc_list.spawn(0x100, smoke)?;

                self.cond.set_alive(false);
                
            } else if self.action_counter > self.action_counter3.saturating_sub(200) {
                //extra blinking
                if self.anim_num > 3 {
                    self.anim_num = 0
                }
            } else {
                //normal blinking
                if self.anim_num > 1 {
                    self.anim_num = 0
                }
            }

        } else {
            //normal blinking
            if self.anim_num > 1 {
                self.anim_num = 0
            }
        }

        //hack: hitting a floor or ceiling will zero vertical velocity, so we save it in target_y
        if self.vel_y != 0 {
            self.target_y = self.vel_y;
        }

        //bounce off the wall
        if self.flags.hit_bottom_wall() || self.flags.hit_top_wall() {
            self.vel_y = -1 * self.target_y;
        }
        if self.flags.hit_left_wall() || self.flags.hit_right_wall() {
            self.vel_x *= -1;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;




        self.anim_rect = state.constants.npc.n375_time_collectible[self.anim_num as usize];

        //mechanic for adding time is in player_hit.rs (with hearts, exp, and missiles)

        Ok(())

    }




    //points to coordinate specified by ANP
    pub(crate) fn tick_n376_direction_arrow(
        &mut self,
        state: &mut SharedGameState,
        stage: &mut Stage,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        frame: &Frame,
    ) -> GameResult {

        self.layer = NPCLayer::Foreground;


        match self.action_num {
            //set x and y
            1 => {
                self.target_x = self.tsc_direction as i32 * 0x200 * 0x10;
                self.target_y = self.action_counter as i32 * 0x200 * 0x10;
                self.action_num = 0;
            },
            //set pointer hover direction (sits one block offset, this controls which), (left means the arrow will point left and be on the right side)
            2 => {
                self.direction = Direction::from_int_facing(self.tsc_direction as usize).unwrap_or(Direction::Bottom);
                if self.direction == Direction::FacingPlayer {self.direction = Direction::Bottom}
            },

            _ => {},
        }


        /*
            unit circle:
              3pi/2
                |
            pi-------0
                |
               pi/2
        */


        //calculate NPC position
        {

            //this is where the NPC wants to go (self.target_xy is where it wants to point)
            let (tgt_x, tgt_y) = match self.direction {
                Direction::Left => (self.target_x + 0x2000, self.target_y),
                Direction::Right => (self.target_x - 0x2000, self.target_y),
                Direction::Up => (self.target_x, self.target_y + 0x2000),
                Direction::Bottom => (self.target_x, self.target_y - 0x2000),
                _ => (self.target_x, self.target_y)
            };

            //the maximum cartesian distance the arrow is allowed to travel from the center of the frame
            let rect = state.get_drawn_edge_rect(stage);
            let max_x = rect.width() / 2.0 * 0x200 as f32;
            let max_y = rect.height() / 2.0 * 0x200 as f32;

            let (frame_x, frame_y) = (frame.x as f32 + max_x, frame.y as f32 + max_y);

            //trim edge offset so we can still see the NPC
            let max_x = max_x - (512.0 * 8.0);
            let max_y = max_y - (512.0 * 8.0);

            let angle = (frame_y - tgt_y as f32).atan2(frame_x - tgt_x as f32);
            
            //direct distance to object from center of frame
            let dist = ((frame_y - tgt_y as f32).powi(2) + (frame_x - tgt_x as f32).powi(2)).sqrt();
    

            //calculate length of hypotinuse for vertical and horizontal right triangles (to see witch edge is the limiting factor)
            let mut angg = angle.cos();
            angg += 1.0;
            
            let hyp_x = (max_x / angle.cos()).abs();
            let hyp_y = (max_y / angle.sin()).abs();
    
            let max_dist = if hyp_x.abs() < hyp_y.abs() {hyp_x} else {hyp_y};

            //endpoint is within frame limit, simply place on there
            if dist < max_dist.abs() {
                self.x = tgt_x;
                self.y = tgt_y;
            } else {
                self.x = (frame_x - angle.cos() * max_dist) as i32;
                self.y = (frame_y - angle.sin() * max_dist) as i32;
            }


        }

        //calculate point direction
        {
            let angle = PI + ((self.y - self.target_y) as f32).atan2((self.x - self.target_x) as f32);

            //map angle from 0-2pi to 0-7
            let angle_of_int = (angle * 8.0 / (2.0 * PI)) as usize;

            self.anim_rect = state.constants.npc.n376_direction_arrow[angle_of_int];



        }


        //animate
        self.anim_counter += 1;
        if self.anim_counter > 4 {
            self.anim_counter = 0;

            self.anim_num += 1;
            if self.anim_num > 3 {self.anim_num = 0};

        }
        let height = self.anim_rect.height();
        self.anim_rect.top += height * self.anim_num as u16;
        self.anim_rect.bottom += height * self.anim_num as u16;


        //npc.action_num = action_num;
        //npc.tsc_direction = tsc_direction;
        //npc.action_counter = action_counter;

        Ok(())
    }



    pub(crate) fn tick_n377_door_outline(
        &mut self,
        state: &mut SharedGameState,
    ) -> GameResult {


        self.anim_counter += 1;
        if self.anim_counter > 4 {
            self.anim_counter = 0;

            self.anim_num += 1;
            if self.anim_num > 3 {self.anim_num = 0};

        }
        self.anim_rect = state.constants.npc.n377_door_outline[self.anim_num as usize];

        Ok(())


    }



    pub(crate) fn tick_n378_wind_left(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        self.tick_n096_fan_left(state, players, npc_list)?;
        self.anim_rect = Rect::new(0,0,0,0);
        Ok(())
    }
    pub(crate) fn tick_n379_wind_up(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        self.tick_n097_fan_up(state, players, npc_list)?;
        self.anim_rect = Rect::new(0,0,0,0);
        Ok(())
    }
    pub(crate) fn tick_n380_wind_right(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        self.tick_n098_fan_right(state, players, npc_list)?;
        self.anim_rect = Rect::new(0,0,0,0);
        Ok(())
    }
    pub(crate) fn tick_n381_wind_down(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        self.tick_n099_fan_down(state, players, npc_list)?;
        self.anim_rect = Rect::new(0,0,0,0);
        Ok(())
    }


    pub(crate) fn tick_n382_omnidirectional_hockaloogie(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {


        let rect = [
            Rect::new(0, 0, 0, 0),//invisible
            Rect::new(0, 0, 48, 48),//frame 1 fly
            Rect::new(48, 0, 96, 48),//frame 2
            Rect::new(96, 0, 144, 48),//frame 3
            Rect::new(144, 0, 192, 48),//frame 1 chew
            Rect::new(192, 0, 240, 48),//frame 2
            Rect::new(240, 0, 288, 48),//frame 3
            Rect::new(0, 48, 48, 96),//frame 1 spit
            Rect::new(48, 48, 96, 96),//frame 2
            Rect::new(96, 48, 144, 96),//frame 3
        ];

        //let apple = [3,4,5,];


	    //for shooting at angle
	    //let (deg, xm, ym);

	    //for finding PC location
	    //let (xx, yy, direct);

        let player = self.get_closest_pseudo_player_mut(players, npc_list);

        //idle if far away from PC or if the PC is keyed (AND we're not in replay mode)
        if (!state.control_flags.control_enabled() && !state.control_flags.replay_mode())
        || (player.x() < self.x - 0x200 * 16 * 22
        || player.x() > self.x + 0x200 * 16 * 22
        || player.y() < self.y - 0x200 * 16 * 22
        || player.y() > self.y + 0x200 * 16 * 22)
        {
            self.animate(2, 1, 3);
            self.anim_rect = rect[self.anim_num as usize];
            return Ok(());
        }


        match self.action_num {
            0 => {
                //snap desired location to current location to start with
                self.target_x = self.x;
                self.target_y = self.y;
                self.npc_flags.set_shootable(true);
                self.action_num = 10; //todo: fallthrough
            }
            //float
            10 => {

                //animation
                self.animate(2, 1, 3);

                //after 200 ticks, make a spitball
                self.action_counter += 1;
                if self.action_counter > 200 {
                    self.action_num = 20;
                    self.action_counter = 0;
                    self.anim_num = 4;
                }

            }
            //chew
            20 => {

                //animation
                self.animate(2, 4, 6);

                //after 50 ticks, make a spitball
                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 30;
                    self.action_counter = 0;
                    self.anim_num = 7;
                }

            }
            //spit
            30 => {
                //animation
                self.animate(2, 7, 9);

                //every 5 ticks, spawn a spitball (total 10)
                self.action_counter += 1;
                if self.action_counter % 5 == 1 {

                    /*
                        unit circle:
                          3pi/2
                            |
                        pi-------0
                            |
                           pi/2
                    */

                    let xx = self.x - player.x();// - self.x;
                    let yy = self.y - player.y();// - self.y;
                    let direction;

                    //if more above/below NPC than side to side
                    if xx.abs() < yy.abs() {
                        //player is below
                        if yy.is_negative() {
                            direction = Direction::Bottom;
                        } else {
                            direction = Direction::Up;
                        }
                    } else {
                        //player is to the right
                        if xx.is_negative() {
                            direction = Direction::Right;
                        } else {
                            direction = Direction::Left;
                        }
                    }

                    let mut deg = (yy as f32).atan2(xx as f32);
                    //range by pi/2 (+- PI/4) (78.539)
                    deg += self.rng.range(-78..78) as f32 / 100.0;
                    let vel_x = deg.cos() * 512.0;
                    let vel_y = deg.sin() * 512.0;

                    let mut npc = NPC::create(214, &state.npc_table);
                    //npc.action_num = 2;
                    npc.x = self.x;
                    npc.y = self.y;
                    npc.vel_x = vel_x as i32;
                    npc.vel_y = vel_y as i32;
                    npc.direction = direction;
                    npc.cond.set_alive(true);
                    let _ = npc_list.spawn(0x100, npc);

                    state.sound_manager.play_sfx(21);
                }

                if self.action_counter > 50 {
                    self.action_num = 10;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

            }

            _ => {}
        }

        //targeting
        {
            let follow_zone = Rect::new( 128 * 0x200, 128 * 0x200, 128 * 0x200, 128 * 0x200 );
            let nogo_zone =  Rect::new( 64 * 0x200, 64 * 0x200, 64 * 0x200, 64 * 0x200 );

            //if PC is outside the follow zone
            if !follow_zone.check_overlaps_point(self.x, self.y, player.x(), player.y()) {
                
                if self.target_x < player.x() {
                    self.target_x += 0x400;
                } else {
                    self.target_x -= 0x400;
                }

                if self.target_y < player.y() {
                    self.target_y += 0x400;
                } else {
                    self.target_y -= 0x400;
                }
            } else if nogo_zone.check_overlaps_point(self.x, self.y, player.x(), player.y()){
                //TOO close to the PC

                if self.target_x < player.x() {
                    self.target_x -= 0x400;
                } else {
                    self.target_x += 0x400;
                }

                if self.target_y < player.y() {
                    self.target_y -= 0x400;
                } else {
                    self.target_y += 0x400;
                }

            }
        }


        //move to target
        {

            if self.x < self.target_x {
                self.vel_x += 15;
            } else {
                self.vel_x -= 15;
            }

            if self.y < self.target_y {
                self.vel_y += 15;
            } else {
                self.vel_y -= 15;
            }

            //speed limit
            if self.vel_x > 0x400 {
                self.vel_x = 0x400;
            } else if self.vel_x < -0x400 {
                self.vel_x = -0x400;
            }

            if self.vel_y > 0x400 {
                self.vel_y = 0x400;
            } else if self.vel_y < -0x400 {
                self.vel_y = -0x400;
            }

            //bumping
            if self.flags.hit_top_wall() {
                self.vel_y = 0x200;
            }
            if self.flags.hit_bottom_wall() {
                self.vel_y = -0x200;
            }
            if self.flags.hit_left_wall() {
                self.vel_x = 0x200;
            }
            if self.flags.hit_right_wall() {
                self.vel_x = -0x200;
            }

        }


        //debug
        //state.settings.noclip = true;
        //self.x = self.target_x;
        //self.y = self.target_y;

        self.x += self.vel_x;
        self.y += self.vel_y;

        //left looking
        self.anim_rect = rect[self.anim_num as usize];

        //right looking
        if self.x < player.x() {
            //let height = self.anim_rect.height();
            self.anim_rect.top += 128;
            self.anim_rect.bottom += 128;
        }
        


        Ok(())

    }


    pub(crate) fn tick_n383_shield_generator(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {

        let rc_list = [
            Rect::new( 240, 48, 256, 64 ), //off //the x2 y2 coordinates seem to be +1... check this..YES.
            Rect::new( 240, 64, 256, 80 ), //redlighted
            Rect::new( 240, 80, 256, 96 ), //destroyed
        ];

        let infl_life = 1000;

        match self.action_num {
            //initialize
            0 => {
                //set inflated life so we don't die when our life is destroyed
                let set_life = self.life;
                self.life = set_life.saturating_add(infl_life);

                //find other NPCs that share our event number (these will be deleted when this npc "dies")
                for npc in npc_list.iter_alive() {
                    if npc.event_num == self.event_num //matches NPC type
                    && npc.id != self.id {
                        self.child_ids.push(npc.id);
                    }
                }
                //start idleing
                self.action_num = 1;

            },
            //aninmate idle
            1 => {

                self.animate(6, 0, 1);

                //we've "died"
                if self.life < infl_life {

                    for id in &self.child_ids {
                        if let Some(npc) = npc_list.get_npc(*id as usize) {
                            let x = npc.x;
                            let y = npc.y;
                            let mut smoke = NPC::create(4, &state.npc_table);
                            smoke.x = x;
                            smoke.y = y;
                            smoke.cond.set_alive(true);
                            *npc = smoke;
                            
                        }
                    }

                    //make death sound
                    if let Some(table_entry) = state.npc_table.get_entry(self.npc_type) {
                        state.sound_manager.play_sfx(table_entry.death_sound);
                    }

                    //make smoke at NPC location
                    {
                        let x = self.x;
                        let y = self.y;
                        let mut smoke = NPC::create(4, &state.npc_table);
                        smoke.x = x;
                        smoke.y = y;
                        smoke.cond.set_alive(true);
                        let _ = npc_list.spawn(0x100, smoke)?;
                    }
                    //recover life, remove shootability
                    self.life = infl_life;
                    self.npc_flags.set_shootable(false);
                    self.anim_num = 2; //look "dead"



                    //full idle
                    self.action_num = 2;
                }
            }
            _ => {}
        }


        self.anim_rect = rc_list[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n384_moving_shield(
        &mut self,
        state: &mut SharedGameState,
    ) -> GameResult {

        match self.action_num {
            //choose starting direction
            0 => {
                self.action_num = 1;

                match self.direction {
                    Direction::Left => {
                        self.action_num = 1;
                    }
                    Direction::Right => {
                        self.action_num = 3;
                    }
                    _ => (),
                }
            }
            //move to the left
            1 => {
                self.vel_x = -0x400;


                if self.flags.hit_left_wall() {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.vel_x = 0;
                    self.direction = Direction::Right;
                }
            }
            //wait
            2 => {
                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 3;
                    self.anim_counter = 0;
                    self.anim_num = 1;
                }
            }
            //move to the right
            3 => {
                self.vel_x = 0x400;


                if self.flags.hit_right_wall() {
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.vel_x = 0;
                    self.direction = Direction::Left;
                }
            }
            //wait
            4 => {
                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 1;
                    self.anim_counter = 0;
                    self.anim_num = 1;
                }
            }
            _ => (),
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        //draw rects
        self.animate(1, 0, 3);
        self.anim_rect = state.constants.npc.n013_forcefield[self.anim_num as usize];

        Ok(())
    }

    
    pub(crate) fn tick_n385_8_tesla_shooter_ai(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        frame: &Frame,
        direction: Direction,
    ) -> GameResult {


        let rc_list = [
            //left
            Rect::new( 256, 0, 272, 16 ), //idle
            Rect::new( 272, 0, 288, 16 ), //charging
            Rect::new( 288, 0, 304, 16 ), //supercharging

            //up
            Rect::new( 256, 16, 272, 32 ), //idle
            Rect::new( 272, 16, 288, 32 ), //charging
            Rect::new( 288, 16, 304, 32 ), //supercharging

            //right
            Rect::new( 256, 32, 272, 48 ), //idle
            Rect::new( 272, 32, 288, 48 ), //charging
            Rect::new( 288, 32, 304, 48 ), //supercharging     
        
            //down
            Rect::new( 256, 48, 272, 64 ), //idle
            Rect::new( 272, 48, 288, 64 ), //charging
            Rect::new( 288, 48, 304, 64 ), //supercharging
        
        ];

        let dir_offset = match direction {
            Direction::Left => 0,
            Direction::Up => 3,
            Direction::Right => 6,
            _ => 9,
        };

        let player = self.get_closest_pseudo_player_mut(players, &npc_list);

        //zone of activity
        let vis_rect = Rect::new(0x28000, 0x1E000, 0x28000, 0x1E000);

        // //don't do anything of OOB
        // if self.x > player.x() + 0x28000
        // || self.x < player.x() - 0x28000
        // || self.y > player.y() + 0x1E000
        // || self.y < player.y() - 0x1E000

        if (!state.control_flags.control_enabled() && !state.control_flags.replay_mode())
        || !vis_rect.check_overlaps_point(self.x, self.y, player.x(), player.y()) {
            self.anim_rect = rc_list[dir_offset];
            return Ok(());
        }

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.anim_num = 0;
                    self.action_num = 1;


                    self.action_counter2 = self.rng.range(1..3) as u16; //bullets to fire
                    self.action_counter3 = 0; //current number fired

                    //delay handler
                    self.action_counter = self.rng.range(70..120) as u16;

                }


                if self.action_counter != 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_counter = 40; //time before firing
                    self.action_num = 2;
                    self.anim_num = 2;
                }
            }
            2 => {
                
                //delay between fires
                if self.action_counter > 0 {

                    self.action_counter -= 1;

                    //animate between frames 0 and 1
                    self.animate(1, 0, 1);
                    // self.anim_num += 1;
                    // if self.anim_num > 1 {
                    //     self.anim_num = 0;
                    // }
                }
                else {
                    
                    //reached our quota
                    self.action_counter3 += 1;
                    if self.action_counter3 > self.action_counter2 {
                        //return to start
                        self.action_num = 0;

                    } else {
                        //assign delay for next cycle
                        self.action_counter = 20;
                    }

                    //shoot a bullet
                    let angle = f64::atan2((self.y - player.y()) as f64, (self.x - player.x()) as f64)
                            + (self.rng.range(-6..6) as f64 * CDEG_RAD);
    
                    let mut npc = NPC::create(389, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    npc.vel_x = (angle.cos() * -1536.0) as i32;
                    npc.vel_y = (angle.sin() * -1536.0) as i32;

                    let _ = npc_list.spawn(0x100, npc);

                    if !player.cond().hidden() {
                        state.sound_manager.play_sfx(62);
                    }


                    //animate between frames 1 and 2
                    self.animate(1, 1, 2);
                    // self.anim_num += 1;
                    // if self.anim_num > 2 {
                    //     self.anim_num = 1;
                    // }


                }


            }
            _ => (),
        }



        self.anim_rect = rc_list[self.anim_num as usize + dir_offset];

        // if self.life <= 985 {
        //     self.npc_type = 154;
        //     self.action_num = 0;
        // }




        Ok(())
    }

    
    pub(crate) fn tick_n389_tesla_bullet(
        &mut self,
        state: &mut SharedGameState,
    ) -> GameResult {

        self.action_counter += 1;
        if self.flags.hit_anything() || self.action_counter > 300 {

            state.create_caret(
                self.x,
                self.y,
                CaretType::ProjectileDissipation,
                Direction::Left,
            );
            self.cond.set_alive(false);
        }

        let rc_list = [
            Rect::new( 256, 64, 272, 80 ),
            Rect::new( 272, 64, 288, 80 ),
            Rect::new( 288, 64, 304, 80 ),
            Rect::new( 304, 64, 320, 80 ),
        ];

        self.x += self.vel_x;
        self.y += self.vel_y;


        self.animate(1, 0, 3);

        self.anim_rect = rc_list[self.anim_num as usize];


        Ok(())
    }


    //lights
    pub(crate) fn tick_n390_raycast_light(&mut self) -> GameResult {

        //experience drops use flag_num:
        //use flag num to specify light width (in degrees, direction controls cardinal direction)
        
        //point down for now
        if self.action_num == 0 {
            self.direction = Direction::Bottom;
            self.action_num += 1;
        }


        let light_width = self.flag_num as i32 / 2;
        let direction = match self.direction {
            Direction::Bottom => 270,
            Direction::Left => 180,
            Direction::Up => 90,
            _ => 0,
        };

        self.light_options.light_type = NPCLightType::Cone;
        self.light_options.x = self.x;
        self.light_options.y = self.y;
        self.light_options.light_color = Color::from_rgb(255, 255, 180);
        self.light_options.light_power = 1.0;

        self.light_options.light_angle = (direction - light_width)..(direction + light_width);


        Ok(())
    }

}







