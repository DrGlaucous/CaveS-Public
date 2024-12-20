use std::cmp::Ordering;

use num_traits::{abs, clamp};

use crate::common::{Direction, Rect};
use crate::framework::error::GameResult;
use crate::game::npc::list::NPCList;
use crate::game::npc::{NPCLayer, NPC};
use crate::game::player::{Player, TargetPlayer};
use crate::game::shared_game_state::SharedGameState;
use crate::game::stage::Stage;
use crate::game::weapon::bullet::BulletManager;
use crate::util::rng::RNG;


impl NPC {

    pub(crate) fn tick_n079_mahin(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {        
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 2;
                self.anim_counter = 0;
            }
            2 => {
                self.anim_num = 0;
                if self.rng.range(0..120) == 10 {
                    self.action_num = 3;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                let player = self.get_closest_player_mut(players);
                if (self.x - (0x4000) < player.x)
                    && (self.x + (0x4000) > player.x)
                    && (self.y - (0x4000) < player.y)
                    && (self.y + (0x2000) > player.y)
                {
                    self.face_player(player);
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 2;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;

        self.clamp_fall_speed();

        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n079_mahin[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n371_fatrunner(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        stage: &mut Stage,
    ) -> GameResult {

        //0xCFF
        let mut max_fall_speed = 0xEFF;


        match self.action_num {
            //lockout invisible
            0 => {
                self.anim_rect = Rect::new(0,0,0,0);
            }

            //player control (1 is init, 2 is control 3 is "key")
            1 | 2 | 3 => {

                //set up
                if self.action_num == 1 {

                    //find index of fall height NPC
                    for npc in npc_list.iter_alive() {
                        if npc.event_num == self.event_num + 1 {
                            self.parent_id = npc.id;
                            break;
                        }
                    }
                    self.action_num = 2;
                }

                //run normal action
                {
                    let player_index = 0;

                    //requres key state to be false
                    let move_left = players[player_index].controller.move_left() && self.action_num != 3;
                    let move_right = players[player_index].controller.move_right() && self.action_num != 3;
                    let move_jump = players[player_index].controller.trigger_jump() || players[player_index].controller.trigger_up() && self.action_num != 3;
            
                    let brick_crush = self.target_y > 2580; //2801 is the speed the moment of collision (falling from 3 bricks, its 2592)
            
                    let hit_l = self.flags.hit_left_wall();
                    let hit_r = self.flags.hit_right_wall();
            
            
                    //walking
                    if move_left != move_right {
                        if move_left {
                            self.vel_x = -0x300;
                            self.direction = Direction::Left;
                        } else {
                            self.vel_x = 0x300;
                            self.direction = Direction::Right;
                        }
            
                        //also stand (but keep velocity so we keep hitting the wall)
                        if hit_l || hit_r {
                            if self.anim_num > 2 {self.anim_num = 3;}
                            self.animate(5, 0, 2);
                        } else {
                            if self.anim_num < 3 || self.anim_num > 10 {self.anim_num = 3;}
                            self.animate(3, 3, 10);
                        }
            
                    } else {
                        //standing
                        self.vel_x = 0;
            
                        if self.anim_num > 2 {self.anim_num = 3;}
                        self.animate(5, 0, 2);
                    }
            
                    //jumping + jump rects
                    if self.flags.hit_bottom_wall() {
            
                        //quake + smoke if we touched down
                        if self.vel_y2 == 0 {
                            state.quake_counter = 10;
            
                            //smoke +  destroy snack
                            if brick_crush {
                                //big thud
                                state.sound_manager.play_sfx(112);
                                let mut npc = NPC::create(4, &state.npc_table);
                                npc.cond.set_alive(true);
                                npc.direction = Direction::Left;
            
                                for _ in 0..3 {
                                    npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                                    npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                                    npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                                    npc.vel_y = self.rng.range(-0x600..0) as i32;
            
                                    let _ = npc_list.spawn(0, npc.clone());
                                }
            
                                //check if the tile(s) below the character are breakable (will at most be 2)
                                let x_check = [
                                    self.x - self.hit_bounds.left as i32,
                                    self.x + self.hit_bounds.right as i32,
                                ];
                                //we'll be looking at the coordinate right below the PC's feet
                                let y_c = self.y + self.hit_bounds.bottom as i32 + 0x200 * state.tile_size.as_int() / 2;
            
                                for x_c in x_check {
            
                                    let tile_size = state.tile_size.as_int() * 0x200;
                                    let x = (x_c / tile_size) as usize;
                                    let y = (y_c / tile_size) as usize;
            
                                    let attrib = stage.map.get_attribute(x, y);
                                    let tile_type = stage.tile_at(x, y);
            
                                    match attrib {
                                        //snack block, sub 1 (equivalent of SMP)
                                        0x43 => {
                                            state.sound_manager.play_sfx(12); //break block sfx
                                            stage.change_tile(x, y, tile_type.wrapping_sub(1));
                                            self.vel_y = -0x7FF; //bounce
            
            
                                            //more smoke
                                            {
                                                npc.x = x as i32 * 0x2000;
                                                npc.y = y as i32 * 0x2000;
                
                                                for _ in 0..3 {
                                                    let _ = npc_list.spawn(0, npc.clone());
                                                }
                                            }
            
                                            //brick shards
                                            {
                                                let mut npc = NPC::create(372, &state.npc_table);
                                                npc.x = x as i32 * 0x2000;
                                                npc.y = y as i32 * 0x2000;
                                                npc.cond.set_alive(true);
                                                npc.action_counter = tile_type;
            
                                                npc.direction = Direction::Left;
                                                let _ = npc_list.spawn(0, npc.clone());
                                                npc.direction = Direction::Up;
                                                let _ = npc_list.spawn(0, npc.clone());
                                                npc.direction = Direction::Right;
                                                let _ = npc_list.spawn(0, npc.clone());
                                                npc.direction = Direction::Bottom;
                                                let _ = npc_list.spawn(0, npc);
            
                                            }
            
                                        }
                                        _ => {}
                                    }
            
                                }
            
            
            
                            } else {
                                //low thud
                                state.sound_manager.play_sfx(111);
                            }
                        }
            
                        self.vel_y2 = 1; //using this to see if we touched down from a jump (1 == on floor, 0 == in air)
            
                        if move_jump {
                            state.sound_manager.play_sfx(80); //jump noise
                            self.vel_y = -0xAFF;
                        }
                    } else if self.vel_y < 0 {
                        self.vel_y2 = 0;
            
                        //jump rect
                        self.anim_num = 11;
                    } else {
                        self.vel_y2 = 0;
            
                        //fall rect
                        self.anim_num = 12;
                    }
            
                }

                //check for spikes
                if self.flags.hit_by_spike() {
                    self.action_num = 10;
                    state.sound_manager.play_sfx(81); //die noise
                    self.vel_y = -0x7FF; //bounce
                }

                //check for lower map limit (set by null npc with action number + 1)
                if let Some(n) = npc_list.get_npc(self.parent_id as usize) {
                    if self.y > n.y {
                        self.action_num = 10;
                        state.sound_manager.play_sfx(81); //die noise
                    }
                }

            }

            //spike hit sequence
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;

                    self.layer = NPCLayer::Foreground; //fall in front of the tiles
                    self.npc_flags.set_ignore_solidity(true);

                    self.anchor_x = 8.0;//16.0;
                    self.anchor_y = 24.0;//24.0;

                    self.target_x = self.rng.range(-100..100); //random spin
                }

                max_fall_speed = 0x1400;

                
                self.angle += self.target_x as f32 / 100.0;

                //"die" and run event
                if self.y > stage.map.height as i32 * stage.map.tile_size.as_int() * 0x200 {
                    state.textscript_vm.start_script(self.event_num);
                    self.action_num = 30;//goto idle
                };



            }
            //fall to death sequence
            20 | 21 => {

            }

            //walk in direction
            30 => {

                if self.direction == Direction::Left {
                    self.vel_x = -0x300;
                } else {
                    self.vel_x = 0x300;
                }
    
                if self.anim_num < 3 || self.anim_num > 10 {self.anim_num = 3;}
                self.animate(3, 3, 10);
            }
            //stand in place
            31 => {
                //standing
                self.vel_x = 0;
    
                if self.anim_num > 2 {self.anim_num = 3;}
                self.animate(5, 0, 2);
            }
            //back turned
            32 => {
                self.vel_x = 0;
                self.anim_num = 13;
            }

            _ => {}
        }

        //apply grvity + rects ONLY if we aren't locked out
        if self.action_num != 0 {
            self.vel_y += 0x90;
            if self.vel_y > max_fall_speed {
                self.vel_y = max_fall_speed;
            }
    
            self.target_y = self.vel_y; //keep track of last velocity (for brick_crush)
    
            self.x += self.vel_x;
            self.y += self.vel_y;
    
            let rl_offset = if self.direction == Direction::Right {14} else {0};
    
            self.anim_rect = state.constants.npc.n371_fatrunner[self.anim_num as usize + rl_offset];
        }

        Ok(())
    }




    pub(crate) fn tick_n372_brick_shard(
        &mut self,
        stage: &mut Stage,
    ) -> GameResult {

        //when spawning, set self.action_counter to the ID of the tile to be micked and the direction to the corresponding corner

        match self.action_num {
            //initialize
            0 => {
                self.layer = NPCLayer::Foreground;
                self.action_num = 1;
                self.vel_x = self.rng.range(-0x400..0x400);
                self.vel_y = self.rng.range(-0x600..0);
                //random spin speed from -pi to pi
                self.target_x = self.rng.range(-100..100);

                self.anchor_x = 4.0;
                self.anchor_y = 4.0;
        
                //portion of the brick to draw
                //Left |Up
                //Right|Bottom

                let rc = Rect::new(
                    (self.action_counter % 16) * 16,
                    (self.action_counter / 16) * 16,
                    (self.action_counter % 16) * 16 + 16,
                    (self.action_counter / 16) * 16 + 16,
                );

                self.anim_rect = match self.direction {
                    Direction::Left => {
                        Rect::new(rc.left, rc.top, rc.right - 8, rc.bottom - 8)
                    },
                    Direction::Up => {
                        Rect::new(rc.left + 8, rc.top, rc.right, rc.bottom - 8)
                    },
                    Direction::Right => {
                        Rect::new(rc.left, rc.top + 8, rc.right - 8, rc.bottom)
                    },
                    //bottom
                    _ => {
                        Rect::new(rc.left + 8, rc.top + 8, rc.right, rc.bottom)
                    },
                };
        
            }
            //do nothing
            _ => {}
        }
        

        if self.y > stage.map.height as i32 * stage.map.tile_size.as_int() * 0x200 {
            self.cond.set_alive(false);
        };

        //apply gravity
        self.vel_y += 0x90;
        if self.vel_y > 0xCFF {
            self.vel_y = 0xCFF;
        }

        self.angle += self.target_x as f32 / 100.0;
        self.x += self.vel_x;
        self.y += self.vel_y;

        Ok(())
    }



    pub(crate) fn tick_n373_soap_shuttle(
        &mut self,
        state: &mut SharedGameState,
        stage: &mut Stage,
    ) -> GameResult {


        let frame_travel_count = 70; //number of frames it takes to travel from one end of the screen to the other (independent of screen size)

        //separate init (so it is guaranteed to run even on even if we set the action number to something different on the first tick)
        //this is the "centerpoint" on the movement semicircle
        if self.action_counter3 == 0 {
            self.target_x = self.x;
            self.target_y = self.y;
            self.layer = NPCLayer::Foreground;

            self.action_counter3 = 1;
        }


        match self.action_num {
            
            //init
            // 0 => {
            //     //goto idle
            //     self.anim_num = 6;
            //     self.action_num = 1;
            // }

            //drop-off sequence (start in top left corner of screen, arc to center, then go to top right) when at "centerpoint", run self event
            //"centerpoint" is where self event+1 is
            10 | 11 => {

                //init, snap to top left corner of frame
                if self.action_num == 10 {

                    self.action_num = 11;
                    self.action_counter = 0;

                    //the maximum cartesian distance the NPC is allowed to travel from the center of the frame
                    let rect = state.get_drawn_edge_rect(stage);
                    let max_x = rect.width() / 2.0 * 0x200 as f32;
                    let max_y = rect.height() / 2.0 * 0x200 as f32;


                    //position self in the top left corner of the centerpoint (0,0 is center here)
                    let start_x = -max_x - (self.display_bounds.left + self.display_bounds.right) as f32;
                    let start_y = -max_y - (self.display_bounds.top + self.display_bounds.bottom) as f32;

                    //set target to the top right corner of the centerpoint
                    let end_x = max_x + (self.display_bounds.left + self.display_bounds.right) as f32;
                    let mid_x = (start_x + end_x) / 2.0;

                    //no "+b" because our "center" is at 0,0
                    // y = ax^2
                    //  y/(x^2)=A
                    //stow 'A' multiplier
                    self.variable_1 = start_y / (start_x * start_x);

                    //store step size
                    self.vel_x = ((end_x - start_x) as f32 / frame_travel_count as f32).ceil() as i32;

                    //store start offset
                    self.vel_x2 = start_x as i32;
                    //self.vel_y2 = start_y as i32;

                    //<ANP0300:0010:0000
                }

                let x_ct = self.vel_x2 + self.vel_x * self.action_counter as i32;
                
                self.x =  self.target_x + x_ct;

                self.y =  self.target_y + (self.variable_1 * x_ct as f32 * x_ct as f32) as i32;

                self.action_counter += 1;




                //hit midpoint, run event
                if self.action_counter == frame_travel_count / 2 {
                    state.textscript_vm.start_script(self.event_num);
                }

                //reached end of arc, goto idle
                if self.action_counter > frame_travel_count {
                    self.action_num = 1;
                }

                self.animate(1, 3, 4);

                if self.anim_num == 3 && self.anim_counter == 0 {
                    state.sound_manager.play_sfx(82); //flap sound
                }

            }

            //idle
            0 | 1 | _ => {
                self.x = self.target_x;
                self.y = self.target_y;
                self.anim_num = 6;
            }
        }


        self.anim_rect = state.constants.npc.n373_soap_shuttle[self.anim_num as usize];

        Ok(())
    }



    pub(crate) fn tick_n373_soap_shuttle_circle(
        &mut self,
        state: &mut SharedGameState,
        stage: &mut Stage,
    ) -> GameResult {


        let frame_travel_count = 70; //number of frames it takes to travel from one end of the screen to the other (independent of screen size)

        //separate init (so it is guaranteed to run even on even if we set the action number to something different on the first tick)
        //this is the "centerpoint" on the movement semicircle
        if self.action_counter3 == 0 {
            self.target_x = self.x;
            self.target_y = self.y;
            self.layer = NPCLayer::Foreground;

            self.action_counter3 = 1;
        }


        match self.action_num {
            
            //init
            // 0 => {
            //     //goto idle
            //     self.anim_num = 6;
            //     self.action_num = 1;
            // }

            //drop-off sequence (start in top left corner of screen, arc to center, then go to top right) when at "centerpoint", run self event
            //"centerpoint" is where self event+1 is
            10 | 11 => {

                //init, snap to top left corner of frame
                if self.action_num == 10 {

                    self.action_num = 11;
                    self.action_counter = 0;

                    //the maximum cartesian distance the NPC is allowed to travel from the center of the frame
                    let rect = state.get_drawn_edge_rect(stage);
                    let max_x = rect.width() / 2.0 * 0x200 as f32;
                    let max_y = rect.height() / 2.0 * 0x200 as f32;


                    //position self in the top left corner of the centerpoint (0,0 is center here)
                    let start_x = -max_x - (self.display_bounds.left + self.display_bounds.right) as f32;
                    let start_y = -max_y - (self.display_bounds.top + self.display_bounds.bottom) as f32;

                    //set target to the top right corner of the centerpoint
                    let end_x = max_x + (self.display_bounds.left + self.display_bounds.right) as f32;
                    let mid_x = (start_x + end_x) / 2.0;

                    //find center of circle given these points
                    let center_t = {
                        //Calculate midpoints of p1-p2 and p2-p3
                        let mid1 = (
                            (start_x + mid_x) / 2.0,
                            (start_y + 0.0) / 2.0,
                        );

                        let mid2 = (
                            (mid_x + end_x) / 2.0,
                            (0.0 + start_y) / 2.0,
                        );
                    
                        //Calculate slopes of p1-p2 and p2-p3
                        let slope1 = (0.0 - start_y) / (mid_x - start_x);
                        let slope2 = (start_y - 0.0) / (end_x - mid_x);
                    
                        //Perpendicular slopes
                        let perp_slope1 = -1.0 / slope1;
                        let perp_slope2 = -1.0 / slope2;
                    
                    
                        // alculate intersection of the perpendicular bisectors
                        let x_center = (perp_slope1 * mid1.0 - perp_slope2 * mid2.0 + mid2.1 - mid1.1)
                            / (perp_slope1 - perp_slope2);
                        let y_center = perp_slope1 * (x_center - mid1.0) + mid1.1;
                    
                        (x_center, y_center)
                    };

                    //find circle radius
                    let radius = ((start_x - center_t.0).powi(2) + (start_y - center_t.1).powi(2)).powf(0.5);

                    let start_rad = (start_y - center_t.1).atan2(start_x - center_t.0);
                    let end_rad = (start_y - center_t.1).atan2(end_x - center_t.0);

                    //starting/current radian location
                    self.vel_x2 = (start_rad * 1000.0) as i32;

                    let radian_step = (end_rad - start_rad) / frame_travel_count as f32;
                    self.vel_y2 = (radian_step * 1000.0) as i32; //size to step vel_x2 by.

                    //re-use to store radius
                    self.vel_y = radius as i32;

                }

                self.action_counter += 1;

                //get current raidan given action counter state
                let curr_radain = (self.vel_x2 + self.action_counter as i32 * self.vel_y2) as f32 / 1000.0;

                self.x = (curr_radain.cos() * self.vel_y as f32) as i32 + self.target_x;
                self.y = (curr_radain.sin() * self.vel_y as f32) as i32 + self.target_y - self.vel_y;


                //hit midpoint, run event
                if self.action_counter == frame_travel_count / 2 {
                    state.textscript_vm.start_script(self.event_num);
                }

                //reached end of arc, goto idle
                if self.action_counter > frame_travel_count {
                    self.action_num = 1;
                }


                self.animate(1, 3, 4);

                if self.anim_num == 3 && self.anim_counter == 0 {
                    state.sound_manager.play_sfx(82); //flap sound
                }

            }

            //idle
            0 | 1 | _ => {
                self.x = self.target_x;
                self.y = self.target_y;
                self.anim_num = 6;
            }
        }


        self.anim_rect = state.constants.npc.n373_soap_shuttle[self.anim_num as usize];

        Ok(())
    }




    pub(crate) fn tick_n374_npc_collider(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {

        match self.action_num {
            //find NPC to collide with (using our flag number for refrence)
            0 => {
                self.action_num = 1;
                for npc in npc_list.iter_alive() {

                    //if npc.npc_type == 371 {
                    if npc.event_num == self.flag_num {
                        self.parent_id = npc.id;
                        break;
                    }
                }
            }
            //check for collisions
            1 => {
                if let Some(npc) = npc_list.get_npc(self.parent_id as usize) {
                    
                    let npc_1_coords = (self.x, self.y);
                    let npc_2_coords = (npc.x, npc.y);
                    let npc_1_hit_bounds = self.hit_bounds;
                    let npc_2_hit_bounds = npc.hit_bounds;
            
            
                    if npc_1_coords.0 + (npc_1_hit_bounds.right as i32) > npc_2_coords.0 - (npc_2_hit_bounds.left as i32)
                        && npc_1_coords.0 - (npc_1_hit_bounds.left as i32) < npc_2_coords.0 + (npc_2_hit_bounds.right as i32)
                        && npc_1_coords.1 + (npc_1_hit_bounds.bottom as i32) > npc_2_coords.1 - (npc_2_hit_bounds.top as i32)
                        && npc_1_coords.1 - (npc_1_hit_bounds.top as i32) < npc_2_coords.1 + (npc_2_hit_bounds.bottom as i32)
                    {
                        //removed universal-ness with this condition, but it's not like I'll be using it for anything else in this mod
                        //ensure the NPC isn't falling "dead"
                        if npc.action_num != 10 && npc.action_num != 11 {
                            state.textscript_vm.start_script(self.event_num);
                        }
                    }
                }

            }
            _ => {

            }
        }


        Ok(())
    }




    pub(crate) fn tick_n375_store_door(
        &mut self,
        state: &mut SharedGameState,
    ) -> GameResult {

        let door_rect = Rect::new(112, 80, 144, 112);
        
        //let display_rect = Rect::new(16 * 0x200, 24 * 0x200, 16 * 0x200, 8 * 0x200);

        //separate init (so it is guaranteed to run even on even if we set the action number to something different on the first tick)
        //save starting location
        if self.action_counter3 == 0 {
            self.target_x = self.x;
            self.target_y = self.y;

            self.action_counter3 = 1;
        }

        let door_height = 0x200 * 16 * 2;

        match self.action_num {

            //idle open
            0 => {
                self.anim_rect = Rect::new(0, 0, 0, 0);
            }
            //idle closed
            1 => {
                self.anim_rect = door_rect;
            }
            //open
            10 | 11 => {

                if self.action_num == 10 {
                    self.vel_y = -0x200;
                    self.vel_y2 = self.target_y;
                    self.action_num = 11;
                }

                self.vel_y2 += self.vel_y;

                let dist = ((self.target_y - self.vel_y2) / 0x200).abs() as u16;

                self.anim_rect = Rect {
                    left: door_rect.left,
                    top: door_rect.top + dist,
                    right: door_rect.right,
                    bottom: door_rect.bottom,
                };

                if self.target_y >= self.vel_y2 + door_height {
                    self.action_num = 0;
                }

            }
            //close
            20 | 21 => {

                if self.action_num == 20 {
                    self.vel_y = 0x200;
                    self.vel_y2 = self.target_y - door_height;
                    self.action_num = 21;
                }

                self.vel_y2 += self.vel_y;

                let dist = ((self.target_y - self.vel_y2) / 0x200).abs() as u16;

                self.anim_rect = Rect {
                    left: door_rect.left,
                    top: door_rect.top + dist,
                    right: door_rect.right,
                    bottom: door_rect.bottom,
                };

                if self.target_y <= self.vel_y2 {
                    self.action_num = 1;
                }

            }

            _ => {}
        }

        Ok(())
    }








}







