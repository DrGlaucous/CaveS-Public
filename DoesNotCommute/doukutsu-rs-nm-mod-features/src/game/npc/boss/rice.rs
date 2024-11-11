use std::iter::{zip, IntoIterator};
use std::ops::{Range, RangeInclusive, RangeBounds};

use crate::common::{Direction, Rect};
use crate::components::flash::Flash;
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::boss::BossNPC;
use crate::game::npc::list::NPCList;
use crate::game::npc::NPC;
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
            npc.target_x = npc.x;
            npc.target_y = npc.y;
        }

        //set rect based on animation number
        npc.anim_rect = if npc.anim_num == 0 { rc_rail } else { rc_floor };


        npc.x += npc.vel_x;
        npc.y += npc.vel_y;

        //we're using the hit rect, not the anim rect right now
        //let half_width = (npc.anim_rect.width() / 2) * 0x200;

        //loopback, snap back to start if we moved beyond rect distance
        if (npc.x + npc.hit_bounds.right as i32) < npc.target_x {
            npc.x += npc.hit_bounds.width() as i32;
        } else if (npc.x - npc.hit_bounds.left as i32) > npc.target_x {
            npc.x -= npc.hit_bounds.width() as i32;
        }

        if (npc.y + npc.hit_bounds.bottom as i32) < npc.target_y {
            npc.y += npc.hit_bounds.height() as i32;
        } else if (npc.y - npc.hit_bounds.top as i32) > npc.target_y {
            npc.y -= npc.hit_bounds.height() as i32;
        }


    }

    //the screen that displays the boss' face, can animate, and takes personal damage (not main boss damage)
    fn tick_b11_rice_tv_screen(&mut self, i: usize) {

        let rc_tv_face = [
            Rect::new(272,0,264,64),
            Rect::new(360,0,352,64),
            Rect::new(448,0,536,64),

            Rect::new(272,64,264,128),
            Rect::new(360,64,352,128),
            Rect::new(448,64,536,128),

            Rect::new(272,128,264,192),
            Rect::new(360,128,352,192),
            Rect::new(448,128,536,192),

            Rect::new(272,192,264,256),
            Rect::new(360,192,352,256),
            Rect::new(448,192,536,256),

            Rect::new(272,256,264,320), //shock
        ];

        //have to do this here to appease the borrow checker
        //parent is the rail slider, this ID is set on NPC creation
        let p_id = self.parts[i].parent_id as usize;
        let parent_npc_coords = (self.parts[p_id].x, self.parts[p_id].y);

        let npc = &mut self.parts[i];

        //offset relative to parent
        npc.x = parent_npc_coords.0;
        npc.y = parent_npc_coords.1 + 0x200 * 8 * 8; //offset down

        //todo: face animations

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
        npc.y = parent_npc_coords.1;

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

    fn tick_b11_rice_tv(&mut self, i: usize) {

        let rc_tv_mount = [
            Rect::new(208,160,272,200), //unlit
            Rect::new(208,200,272,240), //lit
            Rect::new(208,240,272,280), //shock
        ];

        let npc = &mut self.parts[i];

        //move in,
        //leer at player, 

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

    }

    fn tick_b11_rice_platform(&mut self, i: usize) {

        let rc_platform = [
            Rect::new(0,256,144,272), //off
            Rect::new(0,272,144,288), //lit
        ];

        let npc = &mut self.parts[i];

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

    fn set_rail_speed(&mut self, speed: i32) {

        for i in 11..=16 {
            self.parts[i].vel_x = speed;
        }
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


        let rc_grav_particles_down = [
            Rect::new(192,64,208,80),
            Rect::new(192,80,208,96),
            Rect::new(192,96,208,112),
        ];

        let rc_grav_particles_up = [
            Rect::new(192,112,208,128),
            Rect::new(192,128,208,144),
            Rect::new(192,144,208,160),
        ];


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
                        
                        //origin on rightmost edge
                        npc.display_bounds = Rect::new(
                            0x200 * 8 * 17,
                            0x200 * 8 * 0,
                            0x200 * 8 * 0,
                            0x200 * 8 * 2,
                        );

                        //hit bounds is bugged: it doesn't use rect::left, only rect::right, and assumes the origin is in the center.
                        //this is not a problem with the boss, but we have to work around it if we don't want to mess with the other collision code.
                        npc.hit_bounds = Rect::new(
                            0,
                            0,
                            0x200 * 16 * 4,
                            0x200 * 16 * 1,
                        );

                        // npc.hit_bounds = Rect::new(
                        //     0x200 * 16 * 1,
                        //     0x200 * 8 * 0,
                        //     0x200 * 16 * 1,
                        //     16 * 0x200 * 1,
                        // );

                    }


                    self.parts[1].x = 0x200 * 8 * 16 + x;
                    self.parts[1].y = 0x200 * 16 * 6 + y;
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

                        //origin in top left corner
                        npc.display_bounds = Rect::new(
                            16 * 0x200 * 0,
                            16 * 0x200 * 0,
                            16 * 0x200 * 17,
                            16 * 0x200 * 1,
                        );
                        npc.hit_bounds = Rect::new(
                            16 * 0x200 * 0,
                            16 * 0x200 * 0,
                            16 * 0x200 * 17,
                            16 * 0x200 * 1,
                        );

                        npc.y = y + (2 * 16 + 8) * 0x200;
                        npc.x = x + x_loc;

                    }

                    //do floor
                    for (npc, x_loc) in zip( &mut self.parts[14..=16], rail_loc_list) {
                        npc.cond.set_alive(true);
                        npc.npc_flags.set_invulnerable(true);
                        npc.npc_flags.set_solid_hard(true);
                        npc.npc_flags.set_ignore_solidity(true);


                        //origin in top left corner
                        npc.display_bounds = Rect::new(
                            16 * 0x200 * 0,
                            16 * 0x200 * 0,
                            16 * 0x200 * 17,
                            16 * 0x200 * 3,
                        );
                        npc.hit_bounds = Rect::new(
                            16 * 0x200 * 0,
                            16 * 0x200 * 0,
                            16 * 0x200 * 17,
                            16 * 0x200 * 3,
                        );
                        
                        npc.anim_num = 1;

                        npc.y = y + (12 * 16 + 8) * 0x200;
                        npc.x = x + x_loc

                    }





                }

                self.set_rail_speed(-0x200 * 4);


            }
            _ => {
            }
        }


        //run sub-parts
        {

            //run platforms
            for i in 1..=2 {
                self.tick_b11_rice_platform(i);
            }

            //run moving rails
            for i in 11..=16 {
                self.tick_b11_rice_rail(i);
            }
        }


    }







}