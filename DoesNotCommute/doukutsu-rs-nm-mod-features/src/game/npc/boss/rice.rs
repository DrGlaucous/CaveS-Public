use std::iter::{zip, IntoIterator};
use std::ops::{Range, RangeInclusive, RangeBounds};
use num_traits::abs;

use crate::common::{Direction, Rect};
use crate::components::flash::Flash;
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::boss::BossNPC;
use crate::game::npc::list::NPCList;
use crate::game::npc::NPC;
use crate::game::npc::Flag;
use crate::game::physics::PhysicalEntity;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::bullet::BulletManager;
use crate::util::rng::RNG;

/*

Boss behavior:

stage 1:

hanging body in shield, 
many small screens behind, lit with sinister face,
drops in curly boss NPCs only,

stage 2:
shield rises,
platforms appear,
ground starts moving,

TVs slide across top of screen, matching speed,
shooting the TV causes it to crash into the bottom of the shield,
lowering it and allowing the boss to be hit for a short time

for offensive,
shoots sideways lightning from shield,
drops in butes

stage 3:
pallette swap, red+black only,

drops harder enemies including modified missile balrog,
spawn floating waterway blocks

//this takes all 20 boss NPCs...
//10 NPCs make up the floor and rail
//3 NPCs make up the screen
//1 NPC makes up the grav gun
//4 NPCs make up the puppet
//2 NPCs make up the platform(s)


potential fight cycle:

boss sits shielded for a short moment, 
tv moves in and boss moves out, leers at player and shoots homing balls
gravity gun moves in and tries to pin the player
killing the TV causes it to move forward, bring the boss back onscreen, and crash into it,
lowering the shield and allowing the boss to be shot


*/

impl NPC {


    //forms part of a "ray" shot from the gravity gun, pulls the player in a certain direction (and towards the center of the NPC) >|<
    pub(crate) fn tick_n391_gravity(
        &mut self,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        boss: &mut BossNPC,
    ) -> GameResult {


        let rc_grav_particles_down = [
            Rect::new(192,64,208,80),
            Rect::new(192,80,208,96),
            Rect::new(192,96,208,112),
            Rect::new(192,112,208,128),
        ];

        let rc_grav_particles_up = [
            Rect::new(192,128,208,144),
            Rect::new(192,144,208,160),
            Rect::new(192,160,208,176),
            Rect::new(192,176,208,192),
        ];

        self.animate(5, 0, 3);


        if self.direction == Direction::Bottom {
            self.anim_rect = rc_grav_particles_down[self.anim_num as usize];
        } else {
            self.anim_rect = rc_grav_particles_up[self.anim_num as usize];
        }

        //from player mod.rs: 
        /*
        // wind / current forces
        if self.flags.force_left() {
            self.vel_x -= 0x88;
        }
        if self.flags.force_up() {
            self.vel_y -= 0x88;
        }
        if self.flags.force_right() {
            self.vel_x += 0x88;
        }
        if self.flags.force_down() {
            self.vel_y += 0x55;
        }
        */

        //check for NPCs/PCs inside NPC's hitbox
        for p in players {
            let result = Self::test_hit_phys_entity_non_solid(self, p);

            if result.any_flag() {
                //pull/push player

                if self.direction == Direction::Bottom {
                    p.vel_y += 0x55;
                } else {
                    p.vel_y -= 0x88;
                }

                //try to center player in the gravity
                if p.x > self.x {
                    p.vel_x -= 0x10;
                } else {
                    p.vel_x += 0x10;
                }
            }
        }
        for n in npc_list.iter_alive() {
            //ignore collision with ourselves
            if n.id == self.id {
                continue;
            }

            let result = Self::test_hit_phys_entity_non_solid(self, n);

            if result.any_flag() {
                //pull/push NPC

                if self.direction == Direction::Bottom {
                    n.vel_y += 0x55;
                } else {
                    n.vel_y -= 0x88;
                }

                //try to center player in the gravity
                if n.x > self.x {
                    n.vel_x -= 0x10;
                } else {
                    n.vel_x += 0x10;
                }

            }
        }


        //keep in line with boss parent, initial y is set on NPC creation
        self.x = boss.parts[self.parent_id as usize].x;

        Ok(())
    }


    // shot by the monitor at the player, behaves simmilarly to those things jelly-things in the labyrinth
    pub(crate) fn tick_n392_homing_bead(
        &mut self,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        boss: &mut BossNPC,
    ) -> GameResult {

        
        let rc_homing_bead = [
            Rect::new(192,192,208,208),
            Rect::new(192,208,208,224),
        ];


        Ok(())
    }


    //better, more generic version of the player-on-npc non-solid code
    pub(crate) fn test_hit_phys_entity_non_solid(npc_1: &dyn PhysicalEntity, npc_2: &dyn PhysicalEntity) -> Flag {

        let mut flags = Flag(0);
        
        let npc_1_coords = (npc_1.x(), npc_1.y());
        let npc_2_coords = (npc_2.x(), npc_2.y());
        let npc_1_hit_bounds = npc_1.hit_bounds();
        let npc_2_hit_bounds = npc_2.hit_bounds();


        if npc_1_coords.0 + (npc_1_hit_bounds.right as i32) > npc_2_coords.0 - (npc_2_hit_bounds.left as i32)
            && npc_1_coords.0 - (npc_1_hit_bounds.left as i32) < npc_2_coords.0 + (npc_2_hit_bounds.right as i32)
            && npc_1_coords.1 + (npc_1_hit_bounds.bottom as i32) > npc_2_coords.1 - (npc_2_hit_bounds.top as i32)
            && npc_1_coords.1 - (npc_1_hit_bounds.top as i32) < npc_2_coords.1 + (npc_2_hit_bounds.bottom as i32)
        {
            //just set some flag
            flags.set_hit_left_wall(true);
        }

        flags
    }


}

impl BossNPC {


    //either the floor or the ceiling, use tgt_x/y to set the bounds it will loop on (XM and YM are applied to the PC)
    fn tick_b11_rice_rail(&mut self, i: usize) {
        
        let rc_rail = Rect::new(0,0,272,16);
        let rc_floor = Rect::new(0,16,272,64);

        let npc = &mut self.parts[i];

        //set loop lock
        if npc.action_num == 0 {
            npc.action_num += 1;

            //since the NPC's origin has to be in the middle, we already start "part-way" through the loopback, so we offset this to here
            npc.target_x = npc.x;// - npc.hit_bounds.left as i32;
            npc.target_y = npc.y;// - npc.hit_bounds.top as i32;
        }

        //set rect based on animation number
        npc.anim_rect = if npc.anim_num == 0 { rc_rail } else { rc_floor };


        npc.x += npc.vel_x;
        npc.y += npc.vel_y;

        //we're using the hit rect, not the anim rect right now
        //let half_width = (npc.anim_rect.width() / 2) * 0x200;

        let hitbox_width = (npc.hit_bounds.left + npc.hit_bounds.right) as i32;
        let hitbox_height = (npc.hit_bounds.top + npc.hit_bounds.bottom) as i32;

        //loopback, snap back to start if we moved beyond rect distance
        if (npc.x + npc.hit_bounds.right as i32) < npc.target_x {
            npc.x += hitbox_width;
        } else if (npc.x - npc.hit_bounds.left as i32) > npc.target_x {
            npc.x -= hitbox_width;
        }

        if (npc.y + npc.hit_bounds.bottom as i32) < npc.target_y {
            npc.y += hitbox_height;
        } else if (npc.y - npc.hit_bounds.top as i32) > npc.target_y {
            npc.y -= hitbox_height;
        }


    }

    //the screen that displays the boss' face, can animate, and takes personal damage (not main boss damage)
    fn tick_b11_rice_tv_screen(&mut self, i: usize) {

        let rc_tv_face = [
            Rect::new(272,0,360,64), //0
            Rect::new(360,0,448,64),
            Rect::new(448,0,536,64),

            Rect::new(272,64,360,128), //3
            Rect::new(360,64,448,128),
            Rect::new(448,64,536,128),

            Rect::new(272,128,360,192), //6
            Rect::new(360,128,448,192),
            Rect::new(448,128,536,192),

            Rect::new(272,192,360,256), //9
            Rect::new(360,192,448,256),
            Rect::new(448,192,536,256),

            Rect::new(272,256,264,320), //shock (12)
        ];

        //have to do this here to appease the borrow checker
        //parent is the rail slider, this ID is set on NPC creation
        let p_id = self.parts[i].parent_id as usize;
        let parent_npc_coords = (self.parts[p_id].x, self.parts[p_id].y);

        let npc = &mut self.parts[i];

        //offset relative to parent
        npc.x = parent_npc_coords.0;
        npc.y = parent_npc_coords.1 + 0x200 * (8 * 7); //offset down

        //todo: face animations

        let mut anim_offset = 0;

        //determine face offset
        match npc.action_num {
            0 => {

            }
            10 => {
                anim_offset = 2;
            }
            20 => {
                anim_offset = 4;
            }
            30 => {
                anim_offset = 6;
            }
            40 => {
                anim_offset = 8;
            }
            50 => {
                anim_offset = 10;
            }

            
            _ => {}
        }

        //animate
        npc.anim_counter += 1;
        if npc.anim_counter > 12 {
            npc.anim_counter = 0;

            npc.anim_num += 1;
            if npc.anim_num > anim_offset + 1 
            || npc.anim_num < anim_offset{
                npc.anim_num = anim_offset;
            }
        }
        if npc.shock % 2 == 1 {
            npc.anim_num = 12;
        }
        npc.anim_rect = rc_tv_face[npc.anim_num as usize];

    }

    //overlayed on top of the screen to add digital static
    fn tick_b11_rice_tv_noise(&mut self, i: usize) {

        let rc_tv_noise = [
            Rect::new(536,0,624,64),
            Rect::new(536,64,624,128),
        ];

        //have to do this here to appease the borrow checker
        //parent is the main TV screen, set on NPC creation
        let p_id = self.parts[i].parent_id as usize;
        let parent_npc_coords = (self.parts[p_id].x, self.parts[p_id].y);

        let npc = &mut self.parts[i];

        //offset relative to parent
        npc.x = parent_npc_coords.0;
        npc.y = parent_npc_coords.1 + 0x200 * (8 * 7);

        //change static
        if npc.rng.range(0..10) == 5 {
            if npc.anim_num == 0 {
                npc.anim_num = 1;
            } else {
                npc.anim_num = 0;
            }
        }

        npc.anim_rect = rc_tv_noise[npc.anim_num as usize];


    }

    //ticks the tv shuttle, tv screen, and tv noise
    fn tick_b11_rice_tv_shuttle(&mut self, shuttle: usize, t_static: usize, screen: usize) {

        let rc_tv_shuttle = [
            Rect::new(208,160,272,200), //unlit
            Rect::new(208,200,272,240), //lit
            Rect::new(208,240,272,280), //shock
        ];

        let npc = &mut self.parts[shuttle];

        //location to the left offscreen
        let x_min = -(0x200 * 64);

        //location to the right offscreen
        let x_max = (0x200 * (16 * 32 + 64));

        match npc.action_num {

            //idle offscreen
            0 => {
                npc.x = x_min;
                npc.target_x = x_min;
            }

            //idle
            1 => {}


            //slide in
            10 | 11 => {

                if npc.action_num == 10 {
                    //start offscreen
                    npc.x = x_min;
                    npc.target_x = 0x200 * 16 * 16; //roughly halfway

                    //start moving at a decent speed
                    npc.vel_x = 0x400;
                    npc.action_num = 11;
                }

            }

            _ => {}
        }


        //move the shuttle back and forth around the target
        if npc.action_num == 10
        || npc.action_num == 11 {

            if npc.target_x < npc.x {
                npc.vel_x -= 0x05;
            }
            if npc.target_x > npc.x {
                npc.vel_x += 0x05;
            }

            let clamp_speed = (abs(npc.target_x - npc.x) / 0x50).clamp(0x100, 0x800);
            npc.vel_x = npc.vel_x.clamp(-clamp_speed, clamp_speed);
            npc.vel_y = npc.vel_y.clamp(-clamp_speed, clamp_speed);

        } else {
            //don't slide around
            npc.vel_x = 0;
        }

        npc.x += npc.vel_x;

        //move in,
        //leer at player,


        //animate parts
        npc.anim_counter += 1;
        if npc.anim_counter > 4 {
            npc.anim_counter = 0;

            npc.anim_num += 1;
            if npc.anim_num > 1 {
                npc.anim_num = 0;
            }
        }
        if npc.shock % 2 == 1 {
            npc.anim_num = 2;
        }
        npc.anim_rect = rc_tv_shuttle[npc.anim_num as usize];


        //tick sub-parts
        self.tick_b11_rice_tv_noise(t_static);
        self.tick_b11_rice_tv_screen(screen);


    }

    fn tick_b11_rice_grav_gun(&mut self, i: usize) {

        let rc_grav_gun = [
            Rect::new(208,64,272,96), //unlit
            Rect::new(208,96,272,128), //lit
            Rect::new(208,128,272,160), //bottom
        ];

        let npc = &mut self.parts[i];

        //move in,
        //home towards player,
        //shoot gravity,
        //repeat,
        //if life == 0, move forward and expose boss

        match npc.action_num {

            _ => {}
        }

    }

    fn tick_b11_rice_platform(&mut self, i: usize) {

        let rc_platform = [
            Rect::new(0,256,144,272), //off
            Rect::new(0,272,144,288), //lit
        ];

        let npc = &mut self.parts[i];

        let display_width = (npc.display_bounds.left + npc.display_bounds.right) as i32;

        let retracted_x = npc.target_x - display_width * 2; //move extra far back
        let expanded_x = npc.target_x;
        let partway_x =  expanded_x - display_width / 2;

        match npc.action_num {

            //initialize, save starting position as refrence and retract, assume starting point is "full-out"
            0 => {
                npc.target_x = npc.x;
                npc.target_y = npc.y;

                npc.x -= display_width; //are in full-retract position\

                npc.action_num = 1; //idle
            }

            //move to position
            10 | 20 | 30 => {

                //select the correct location to go to
                let pos = match npc.action_num {
                    10 => retracted_x,
                    20 => partway_x,
                    _ => expanded_x,
                };

                //guard against negative velocities
                let absvel = abs(npc.vel_x2);

                if pos > npc.x {
                    npc.x += absvel;
                    npc.vel_x = absvel; //used to move the PC on top of the NPC
                } else {
                    npc.x -= absvel;
                    npc.vel_x = -absvel;
                }

                //close enough, snap to location
                if abs(npc.x - pos) < absvel {
                    npc.x = pos;
                    npc.vel_x = 0;
                    npc.action_num = 1; //idle
                }

            }

            //idle
            1 | _ => {}
        }


        //animate
        npc.anim_counter += 1;
        if npc.anim_counter > 4 {
            npc.anim_counter = 0;

            npc.anim_num += 1;
            if npc.anim_num > 1 {
                npc.anim_num = 0;
            }
        }
        npc.anim_rect = rc_platform[npc.anim_num as usize];

    }

    //helper functions

    //sets the movement speed of the floor and roof
    fn set_rail_speed(&mut self, speed: i32) {

        for i in 11..=16 {
            self.parts[i].vel_x = speed;
        }
    }


    //fixes the hitbox origin bug and works on anything that is a physical entity
    //we aren't using this right now because I want to try to keep this boss self-contained.
    //(and using this collision code requires modification to other bits of code so that the game doesn't override our collision flags)
    fn test_hit_npc_solid_hard_modified(pc: &mut dyn PhysicalEntity, npc: &dyn PhysicalEntity) -> Flag {
        let mut flags = Flag(0);

        //pull in often-re-used variables
        let npc_raw_hit_bounds = npc.hit_bounds();
        let npc_coords = (npc.x(), npc.y());

        let pc_raw_hit_bounds = pc.hit_bounds();
        let pc_coords = (pc.x(), pc.y());

        //get hitbox centers
        let npc_center_offset = (
            (npc_raw_hit_bounds.left as i32 + npc_raw_hit_bounds.right as i32) / 2,
            (npc_raw_hit_bounds.top as i32 + npc_raw_hit_bounds.bottom as i32) as i32 / 2,
        );
        let pc_center_offset = (
            (pc_raw_hit_bounds.left as i32 + pc_raw_hit_bounds.right as i32) / 2,
            (pc_raw_hit_bounds.top as i32 + pc_raw_hit_bounds.bottom as i32) as i32 / 2,
        );

        let mut pp_w = pc_raw_hit_bounds.width();

        pp_w += 1;


        //apply centers to coordinats to get hitbox centers relative to the worldspace
        let (npc_x, npc_y) = (
            npc_coords.0 + npc_raw_hit_bounds.right as i32 - npc_center_offset.0,
            npc_coords.1 + npc_raw_hit_bounds.bottom as i32 - npc_center_offset.1,
        );

        let (mut pc_x, mut pc_y) = (
            pc_coords.0 + pc_raw_hit_bounds.right as i32 - pc_center_offset.0,
            pc_coords.1 + pc_raw_hit_bounds.bottom as i32 - pc_center_offset.1,
        );

        //normalized hit bounds (equidistant on all sides)
        let npc_hit_bounds = Rect::new(npc_center_offset.0, npc_center_offset.1, npc_center_offset.0, npc_center_offset.1);
        let pc_hit_bounds = Rect::new(pc_center_offset.0, pc_center_offset.1, pc_center_offset.0, pc_center_offset.1);

        


        //distance between entities
        let fx1 = abs(pc_x - npc_x) as f32;
        let fy1 = abs(pc_y - npc_y) as f32;

        //size of half of the collision NPC's hitbox
        let fx2 = npc_hit_bounds.right as f32;
        let fy2 = npc_hit_bounds.top as f32;

        //if distance here is zero, set it to be one (to save against div/0 errors)
        let fx1 = if fx1 == 0.0 { 1.0 } else { fx1 };
        let fx2 = if fx2 == 0.0 { 1.0 } else { fx2 };

        //if we're coming at the NPC from the side (slope of location is less than the slope to the corner of the NPC's hitbox)
        if fy1 / fx1 <= fy2 / fx2 {

            //if the top of the PC is above the bottom
            //and the bottom of the PC is below the top (works for both ceiling and floor)
            if (pc_y - pc_hit_bounds.top as i32) < (npc_y + npc_hit_bounds.bottom as i32)
                && (pc_y + pc_hit_bounds.bottom as i32) > (npc_y - npc_hit_bounds.top as i32)
            {
                //if the left of the PC is inside the left wall of the NPC
                //and the left side of the PC is to the right of the NPC
                if (pc_x - pc_hit_bounds.right as i32) < (npc_x + npc_hit_bounds.right as i32)
                    && (pc_x - pc_hit_bounds.right as i32) > npc_x
                {
                    //if the PC's horizontal velocity is slower than the NPC, snap it to the NPC's velocity (push it forward)
                    if pc.vel_x() < npc.vel_x() {
                        pc.set_vel_x(npc.vel_x());
                    }

                    //snap to the surface of the NPC
                    pc_x = npc_x + npc_hit_bounds.right as i32 + pc_hit_bounds.right as i32;
                    flags.set_hit_left_wall(true);
                }

                //if the right of the PC is inside the right wall of the NPC
                //and the right side of the PC is to the left of the NPC
                if (pc_x + pc_hit_bounds.right as i32) > (npc_x - npc_hit_bounds.right as i32)
                    && (pc_x + pc_hit_bounds.right as i32) < npc_x
                {
                    if pc.vel_x() > npc.vel_x() {
                        pc.set_vel_x(npc.vel_x());
                    }

                    pc_x = npc_x - npc_hit_bounds.right as i32 - pc_hit_bounds.right as i32;
                    flags.set_hit_right_wall(true);
                }
            }
        }
        //roof / floor collisions
        else if (pc_x - pc_hit_bounds.right as i32) < (npc_x + npc_hit_bounds.right as i32)
            && (pc_x + pc_hit_bounds.right as i32) > (npc_x - npc_hit_bounds.right as i32)
        {
            if (pc_y - pc_hit_bounds.top as i32) < (npc_y + npc_hit_bounds.bottom as i32)
                && (pc_y - pc_hit_bounds.top as i32) > npc_y
            {
                if pc.vel_y() >= npc.vel_y() {
                    if pc.vel_y() < 0 {
                        pc.set_vel_y(0);
                    }
                } else {
                    pc_y = npc_y + npc_hit_bounds.bottom as i32 + pc_hit_bounds.top as i32 + 0x200;
                    pc.set_vel_y(npc.vel_y());
                }

                flags.set_hit_top_wall(true);
            }

            if (pc_y + pc_hit_bounds.bottom as i32) > (npc_y - npc_hit_bounds.top as i32)
                && (pc_y + pc_hit_bounds.bottom as i32) < (npc_y + 0x600)
            {

                //head bump noise
                // if pc.vel_y() - npc.vel_y() > 0x400 {
                //     state.sound_manager.play_sfx(23);
                // }


                //if in ironhead mode, don't transfer momentum over
                // if pc.control_mode == ControlMode::IronHead {
                //     pc.y() = npc.y() - npc_hit_bounds.top as i32 - pc_hit_bounds.bottom as i32 + 0x200;
                //     flags.set_hit_bottom_wall(true);
                // } else                
                // if npc.npc_flags().bouncy() {
                //     //bounce off the bottom wall
                //     pc.vel_y() = npc.vel_y() - 0x200;
                //     flags.set_hit_bottom_wall(true);
                // }
                // //if the PC has first landed, set momentum to match the NPC and clip the PC out
                // else 
                
                if !pc.flags().hit_bottom_wall() && pc.vel_y() > npc.vel_y() {
                    pc_x = pc_x + npc.vel_x();
                    pc_y = npc_y - npc_hit_bounds.top as i32 - pc_hit_bounds.bottom as i32 + 0x200;
                    pc.set_vel_y(npc.vel_y());

                    flags.set_hit_bottom_wall(true);
                }
            }
        }

        //apply pc_x and pc_y to actual PC coordinates
        //add back the center radius, then subtrackt the actual hitbox offset (opposite of what was done earlier)
        pc.set_x(pc_x + pc_center_offset.0 - pc_hit_bounds.right as i32);
        pc.set_y(pc_y + pc_center_offset.1 - pc_hit_bounds.bottom as i32);



        flags
    }

    //does collision checking agains the player and all active NPCs on the boss part "i"
    fn run_all_collisions_on_npc(&mut self,
        _players: [&mut Player; 2],
        npc_list: &NPCList,
        i: usize
    ) {

        //the best bet would to be converting the players and NPCs into physicalEntity iterators and chaining them together, but this works OK too.

        
        //problem: NPC flags are likely reset as soon as the "real" collision code is run
        //displacement is still affected though, so some is better than none
        for npc in npc_list.iter_alive() {
            if !npc.npc_flags.ignore_solidity() {
                npc.flags = Self::test_hit_npc_solid_hard_modified(npc, &mut self.parts[i]);
            }
        }

        //revert to default method for players because of the flag reset problem
        // for player in players {
        //     if player.cond.alive() {
        //         player.flags = Self::test_hit_npc_solid_hard_modified(player, &mut self.parts[i]);
        //     }
        // }

    }


    //manager, lives at 0,0 and directs the rest of the NPCs
    pub(crate) fn tick_b11_rice (
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        bullet_manager: &BulletManager,
        flash: &mut Flash,
    ) {




        let rc_shield = [
            Rect::new(0,0,0,0), //off
            Rect::new(128,64,160,144), //on
            Rect::new(160,64,192,144), //charge
        ];

        let rc_puppet = [
            Rect::new(0,64,48,208), //slouch
            Rect::new(48,64,96,208), //mid
            Rect::new(96,64,112,208), //hanging
            Rect::new(112,64,128,208), //hanging, shock
        ];

        let rc_generator_top = [
            Rect::new(0,208,80,232), //off
            Rect::new(80,208,160,232), //lit
        ];

        let rc_generator_bottom = [
            Rect::new(0,232,80,256), //off
            Rect::new(80,232,160,256), //lit
        ];

        let rc_lightning = [
            Rect::new(0,288,240,304),
            Rect::new(0,304,240,320),
            Rect::new(0,320,240,336),
            Rect::new(0,336,240,352),
        ];


        //all boss offsets are relative to this (top left corner of boss)
        let (x,y) = (
            0 * 0x200,
            0 * 0x200,
        );


        //manage actions
        match self.parts[0].action_num {
            //init sub-parts
            0 => {

                //parts are interated backwards: 0 is drawn on top

                //"global" hurt sounds
                self.hurt_sound[0] = 52;


                //event controller 0
                {
                    let npc = &mut self.parts[0];
                    npc.action_counter = 0;
                    npc.action_num = 1; //idle mode
                    npc.npc_flags.set_event_when_killed(true);
                    npc.event_num = 1000;
                    npc.life = 300;
                    npc.cond.set_alive(true);
                    

                    //no hurt voice: use
                    //if let Some(table_entry) = state.npc_table.get_entry(npc.npc_type) {
                    //    state.sound_manager.play_sfx(table_entry.hurt_sound);
                    //}
                    //state.sound_manager.play_sfx(self.boss.hurt_sound[idx]);
                }

                //user platforms (1,2)
                {
                    for npc in &mut self.parts[1..=2] {
                        
                        npc.cond.set_alive(true);
                        npc.npc_flags.set_invulnerable(true);
                        npc.npc_flags.set_solid_hard(true);
                        npc.npc_flags.set_ignore_solidity(true);
                        
                        //origin in center of solid part of the platform
                        npc.display_bounds = Rect::new(
                            0x200 * 8 * 13,
                            0x200 * 8 * 1,
                            0x200 * 8 * 4,
                            0x200 * 8 * 1,
                        );

                        //hit bounds is bugged: it doesn't use rect.left, only rect.right, and assumes the origin is in the center.
                        //this is not a problem with the boss, but we have to work around it if we don't want to mess with the other collision code.
                        npc.hit_bounds = Rect::new(
                            0x200 * 8 * 4,
                            0x200 * 8 * 1,
                            0x200 * 8 * 4,
                            0x200 * 8 * 1,
                        );

                    }

                    //starting locations of the platforms
                    self.parts[1].x = 0x200 * 8 * 13 + x;
                    self.parts[1].y = 0x200 * 8 * 20 + y;

                    self.parts[2].x = 0x200 * 8 * 13 + x;
                    self.parts[2].y = 0x200 * 8 * 10 + y;

                    //have the platforms initialize themselves immediately
                    for i in 1..=2 {
                        self.tick_b11_rice_platform(i);
                    }

                    //test: start with platform out
                    self.parts[1].vel_x2 = 0x200 * 400;
                    self.parts[1].action_num = 30;
                    self.tick_b11_rice_platform(1);


                    //speed of platform movement is determined with vel_x2 (using vel_x directly results in the "conveyor effect")
                    self.parts[1].vel_x2 = 0x200 / 2;
                    self.parts[1].action_num = 20; //goto part in

                



                }

                //puppet parts (3,4,5,6)
                {
                    //platforms, shield, and puppet
                }

                //gravity gun (7)
                {

                }

                //tv parts (8,9,10)
                {
                    //static
                    let npc = &mut self.parts[8];

                    //static overlay can't be shot
                    npc.hit_bounds = Rect::new(
                        0x200 * (5 * 8 + 4),
                        0x200 * 8 * 4,
                        0x200 * (5 * 8 + 4),
                        0x200 * 8 * 4,
                    );
                    npc.display_bounds = Rect::new(
                        0x200 * (5 * 8 + 4),
                        0x200 * 8 * 4,
                        0x200 * (5 * 8 + 4),
                        0x200 * 8 * 4,
                    );

                    npc.cond.set_alive(true);
                    npc.parent_id = 10; //shuttle

                    //screen
                    let npc = &mut self.parts[9];
                    
                    npc.hit_bounds = Rect::new(
                        0x200 * (5 * 8 + 4),
                        0x200 * 8 * 4,
                        0x200 * (5 * 8 + 4),
                        0x200 * 8 * 4,
                    );

                    npc.display_bounds = Rect::new(
                        0x200 * (5 * 8 + 4),
                        0x200 * 8 * 4,
                        0x200 * (5 * 8 + 4),
                        0x200 * 8 * 4,
                    );

                    npc.cond.set_alive(true);
                    npc.parent_id = 10; //shuttle

                    //shuttle

                    let npc = &mut self.parts[10];

                    //just the shuttle body
                    npc.hit_bounds = Rect::new(
                        0x200 * 3 * 8,
                        0x200 * 8,
                        0x200 * 3 * 8,
                        0x200 * 8,
                    );

                    npc.display_bounds = Rect::new(
                        0x200 * 4 * 8,
                        0x200 * 16,
                        0x200 * 4 * 8,
                        0x200 * 24,
                    );

                    npc.npc_flags.set_solid_hard(true);
                    npc.cond.set_alive(true);

                    npc.y = y + (3 * 16) * 0x200;
                    npc.x = x; //this is offset back immediately anyway: starting point doesn't matter here

                    self.parts[10].action_num = 0;
                    self.tick_b11_rice_tv_shuttle(10, 8, 9);

                    //slide in
                    self.parts[10].action_num = 10;


                }

                //rails + floor (11,12,13,  14,15,16)
                {

                    let rail_loc_list = [
                        16 * 0x200 * (-1),
                        16 * 0x200 * (17 - 1),
                        16 * 0x200 * ((17 * 2) -1),
                    ];

                    //start with top rail
                    for (npc, x_loc) in zip( &mut self.parts[11..=13], rail_loc_list) {
                        npc.cond.set_alive(true);
                        npc.npc_flags.set_invulnerable(true);
                        npc.npc_flags.set_ignore_solidity(true);

                        //origin was in top left corner
                        //origin is now in center

                        npc.display_bounds = Rect::new(
                            0x200 * 8 * 17,
                            0x200 * 8 * 1,
                            0x200 * 8 * 17,
                            0x200 * 8 * 1,
                        );
                        npc.hit_bounds = Rect::new(
                            0x200 * 8 * 17,
                            0x200 * 8 * 1,
                            0x200 * 8 * 17,
                            0x200 * 8 * 1,
                        );

                        npc.y = y + (2 * 16 + 16) * 0x200;
                        npc.x = x + x_loc;// + (0x200 * 8 * 17);

                    }

                    //do floor
                    for (npc, x_loc) in zip( &mut self.parts[14..=16], rail_loc_list) {
                        npc.cond.set_alive(true);
                        npc.npc_flags.set_invulnerable(true);
                        npc.npc_flags.set_solid_hard(true);
                        npc.npc_flags.set_ignore_solidity(true);

                        //origin was in top left corner
                        //origin is now in center

                        npc.display_bounds = Rect::new(
                            0x200 * 8 * 17,
                            0x200 * 8 * 3,
                            0x200 * 8 * 17,
                            0x200 * 8 * 3,
                        );
                        npc.hit_bounds = Rect::new(
                            0x200 * 8 * 17,
                            0x200 * 8 * 3,
                            0x200 * 8 * 17,
                            0x200 * 8 * 3,
                        );
                        
                        npc.anim_num = 1;

                        npc.y = y + (14 * 16 + 32) * 0x200;
                        npc.x = x + x_loc;// + (0x200 * 8 * 17);

                    }





                }

                self.set_rail_speed(-0x200 * 4);


            }
            _ => {
            }
        }


        //run sub-parts
        {
            //I have to do this dumb thing so that we can re-use p1 and p2 without "moving" them
            let [p1, p2] = players;

            //run platforms
            for i in 1..=2 {
                self.tick_b11_rice_platform(i);
            }

            //run tv
            self.tick_b11_rice_tv_shuttle(10, 8, 9);

            
            //run moving rails
            for i in 11..=13 {
                self.tick_b11_rice_rail(i);
            }
            //run floor (+ collisions)
            for i in 14..=16 {
                self.tick_b11_rice_rail(i);
                self.run_all_collisions_on_npc([p1, p2], npc_list, i);
            }


        }


    }







}