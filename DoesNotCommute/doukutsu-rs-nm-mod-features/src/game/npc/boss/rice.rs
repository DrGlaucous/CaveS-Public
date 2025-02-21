use std::iter::{zip, IntoIterator};
use std::ops::{Range, RangeBounds, RangeInclusive, Shl};
use std::slice::from_raw_parts;
use cpal::platform;
use num_traits::abs;

use crate::common::{Direction, Rect};
use crate::components::flash::Flash;
use crate::framework::error::GameResult;
use crate::game::caret::{Caret, CaretType};
use crate::game::npc::boss::BossNPC;
use crate::game::npc::list::NPCList;
use crate::game::npc::{self, NPC};
use crate::game::npc::Flag;
use crate::game::physics::PhysicalEntity;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::bullet::BulletManager;
use crate::game::stage::Stage;
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

static X: i32 = 0;
static Y: i32 = 0;

static WIDTH: i32 = 0x200 * 16 * 30;
static HEIGHT: i32 = 0x200 * 16 * 16;

impl NPC {


    // shot by the monitor at the player, behaves simmilarly to those things jelly-things in the labyrinth
    pub(crate) fn tick_n391_homing_bead(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        boss: &mut BossNPC,
    ) -> GameResult {
        
        let rc_homing_bead = [
            Rect::new(192,64,208,80),
            Rect::new(192,80,208,96),
        ];

        //I have to do this dumb thing so that we can re-use p1 and p2 without "moving" them
        let [p1, p2] = players;


        match self.action_num {
            //wait with constant time before begining self-drive
            //be "launched" from the cannon
            0 | 1 => {
                if self.action_num == 0 {
                    //so hp can still be set in the NPC table
                    self.action_counter2 = self.life;
                    self.life = 8000;

                    self.action_num = 1;
                    self.action_counter = self.rng.range(20..60) as u16;
                }

                if self.action_counter < 1 {
                    self.action_num = 2; //begin taxicab homing
                } else {
                    self.action_counter -= 1;
                }
            }
            //new direction
            2 => {

                let player = self.get_closest_pseudo_player_mut([p1, p2], npc_list);

                //if closer on x dimension than y dimension
                if (player.x() - self.x).abs() > (player.y() - self.y).abs() {
                    self.vel_y = 0;
                    self.vel_x = if self.x - player.x() < 0 {0x200} else {-0x200};
                } else {
                    self.vel_y = if self.y - player.y() < 0 {0x200} else {-0x200};
                    self.vel_x = 0;
                }

                self.action_counter = self.rng.range(20..60) as u16;
                self.action_num = 1; //return to wait
            }

            //xplode
            4 => {
                self.cond.set_alive(false);

                //make explosion NPC
                let mut boom = NPC::create(392, &state.npc_table);
                boom.x = self.x;
                boom.y = self.y;
                boom.cond.set_alive(true);
                
                npc_list.spawn(0x100, boom)?;

            }
            _ => {}
        }

        //expire
        let pc = self.get_closest_player_mut([p1, p2]);
        self.action_counter3 += 1;
        if self.action_counter3 > 500
        || self.life < 8000 - self.action_counter2
        || Self::test_hit_phys_entity_non_solid(self, pc).hit_anything() {
            self.action_num = 4;
            return Ok(());
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        //animate + beep
        self.anim_counter += 1;
        if self.anim_counter > 5 {
            self.anim_counter = 0;
            self.anim_num += 1;

            if self.anim_num > 1 {
                self.anim_num = 0;
            } else {
                state.sound_manager.play_sfx(60);
            }
            
        }

        self.anim_rect = rc_homing_bead[self.anim_num as usize];

        Ok(())
    }



    pub(crate) fn tick_n392_xplosion(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {

        if self.action_num == 0 {
            self.action_num = 1;

            self.action_counter = 6;
        }

        if self.action_counter % 3 == 0 {
            let radius = 10;

            npc_list.create_death_smoke_up(
                self.x + self.rng.range(-radius..radius) * 0x200,
                self.y + self.rng.range(-radius..radius) * 0x200,
                16 as usize,
                2,
                state,
                &self.rng,
            );
        }

        if self.action_counter == 0 {
            self.cond.set_alive(false);

            state.sound_manager.play_sfx(44);
            //state.create_caret(self.x, self.y, CaretType::Explosion, Direction::Left);
        }
        self.action_counter = self.action_counter.saturating_sub(1);


        Ok(())
    }

    //todo: make a better name for this NPC when I can think again
    pub(crate) fn tick_n393_power_pellet(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {

        //things this NPC needs from the spawner:
        //x, y, vel_x, vel_y, action_counter2

        let rc_power_pellet = [
            Rect::new(192,96,208,112),
            Rect::new(192,112,208,128),
            Rect::new(192,128,208,144),
        ];

        self.action_counter += 1;
        //explode on timeout
        if self.action_counter > self.action_counter2 {
            self.cond.set_alive(false);

            let radius = 0x200 * 16;
            state.create_caret(self.x + self.rng.range(-radius..radius), self.y + self.rng.range(-radius..radius), CaretType::Explosion, Direction::Left);
        }


        self.animate(3, 0, 2);

        //apply velocity
        self.x += self.vel_x;
        self.y += self.vel_y;


        self.anim_rect = rc_power_pellet[self.anim_num as  usize];

        Ok(())
    }

    pub(crate) fn tick_n394_target(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {

        //things this NPC needs from the spawner:
        //action_counter2


        self.action_counter += 1;

        //timeout
        if self.action_counter > self.action_counter2 {
            self.cond.set_alive(false);
        }

        self.animate(0, 0, 1);

        self.anim_rect = state.constants.npc.n333_ballos_lightning[self.anim_num as usize];

        Ok(())
    }


    pub(crate) fn tick_n395_spike_wall_r(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {

        let rc_power_pellet = [
            Rect::new(240,280,272,296),
            Rect::new(240,296,272,312),
        ];


        let left_bounce_rc = Rect::new(
            0x200 * 16,
            0x200 * 8,
            0x200 * -8,
            0x200 * 8
        );


        self.action_counter += 1; //err: overflow

        self.animate(0, 0, 1);

        self.anim_rect = rc_power_pellet[0];

        Ok(())
    }

    /*
    
    cyborg, 2 attack ideas:
    spin arms like a manhack
    stick arms out in front like a spear

    bonus:
    shoot projectiles

    */
    

    pub(crate) fn tick_n396_ravil_cyborg(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        stage: &mut Stage,
        boss: &mut BossNPC,
    ) -> GameResult {

        let life_const = 10000;

        fn hover(npc: &mut NPC) {
            {
                if npc.x > npc.vel_x2 {
                    npc.vel_x -= 20;
                } else {
                    npc.vel_x += 20;
                }

                if npc.y > npc.vel_y2 {
                    npc.vel_y -= 20;
                } else {
                    npc.vel_y += 20;
                }
            }
        }

        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 => {
                //init
                self.life += life_const;
                self.action_num = 1; //goto idle

            }
            1 => {
                //slow down
                self.vel_x = 7 * self.vel_x / 8;
                self.vel_y = 7 * self.vel_y / 8;

                self.animate(20, 0, 1);
            }

            //start attack
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.damage = 0;
                    self.npc_flags.set_shootable(true);

                    //save current location
                    self.vel_x2 = self.x;
                    self.vel_y2 = self.y;

                    self.vel_x = self.rng.range(-0x400..0x400);
                    self.vel_y = self.rng.range(-0x400..0x400);
                }

                //slow down
                //self.vel_x = 7 * self.vel_x / 8;
                //self.vel_y = 7 * self.vel_y / 8;

                hover(self);



                self.animate(20, 0, 1);

                //wait for 80 ticks
                self.action_counter += 1;
                if self.action_counter > 80 {
                    self.action_num = 30;
                }

                //face player
                //let player = self.get_closest_player_ref(&players);

                self.direction = if player.x > self.x { Direction::Right } else { Direction::Left };

            }

            //choose next action (action choices are a regular sequence)
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.action_counter = 0;
                    self.anim_num = 6; //raise hands
                    self.vel_x = 0;
                    self.vel_y = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_counter2 += 1;
                    self.action_counter2 %= 3;

                    self.action_num = match self.action_counter2 {
                        0 => 32,
                        1 => 34,
                        2 => 36,
                        _ => self.action_num,
                    };
                }
                
                hover(self);

            }

            //joust charge action
            32 | 33 => {
                //let player = self.get_closest_player_ref(&players);

                if self.action_num == 32 {
                    self.action_num = 33;
                    self.action_counter = 0;
                    self.npc_flags.set_shootable(false);
                    self.target_x = if player.x >= self.x { player.x + 0x14000 } else { player.x - 0x14000 };
                    self.target_y = player.y;

                    let angle = f64::atan2((self.y - self.target_y) as f64, (self.x - self.target_x) as f64);

                    self.vel_x = (-1536.0 * angle.cos()) as i32;
                    self.vel_y = (-1536.0 * angle.sin()) as i32;

                    self.direction = if self.vel_x <= 0 { Direction::Left } else { Direction::Right };
                
                    state.sound_manager.play_sfx(29); //TP noise
                }

                self.action_counter += 1;
                //self.anim_num = if self.action_counter & 2 != 0 { 2 } else { 3 }; //choose between shoulder charge or invisible
                self.animate(1, 2, 3);

                //timeout, goto choice again
                if self.action_counter > 50 {
                    self.action_num = 20;
                }
            }
            //spin-blade attack
            34 | 35 => {
                //let player = self.get_closest_player_ref(&players);

                if self.action_num == 34 {
                    self.action_num = 35;
                    self.action_counter = 0;
                    self.damage = 4;
                    self.target_x = player.x;
                    self.target_y = player.y;

                    let angle = f64::atan2((self.y - self.target_y) as f64, (self.x - self.target_x) as f64);

                    self.vel_x = (-1536.0 * angle.cos()) as i32;
                    self.vel_y = (-1536.0 * angle.sin()) as i32;

                    let half_w = stage.map.width as i32 * state.tile_size.as_int() * 0x200 / 2;
                    let half_h = stage.map.height as i32 * state.tile_size.as_int() * 0x200 / 2;


                    self.direction = if self.vel_x <= 0 { Direction::Left } else { Direction::Right };
                }

                self.action_counter += 1;
                if self.action_counter > 20 && self.shock != 0 {
                    self.action_num = 40;
                } else if self.action_counter > 50 || (self.flags.hit_right_wall() || self.flags.hit_left_wall()) {
                    self.action_num = 20;
                }

                //animate twirl
                self.animate(1, 4, 5);

                //sound
                if self.action_counter % 5 == 1 {
                    state.sound_manager.play_sfx(109); //critter fly
                }
            }
            
            //charge + shoot
            36 | 37 => {
                //make target, flicker shot
                self.animate(1, 6, 7);

                if self.action_num == 36 {

                    self.action_num = 37;

                    self.target_x = player.x;
                    self.target_y = player.y;
    
                    let mut tgt = NPC::create(394, &state.npc_table);
                    tgt.cond.set_alive(true);
                    tgt.x = self.target_x;
                    tgt.y = self.target_y;
                    tgt.action_counter2 = 50; //alive for 50 ticks
                    let _ = npc_list.spawn(0x100, tgt);
    
                    state.sound_manager.play_sfx(103); //chg sound
                }

                //shoot
                self.action_counter += 1;
                if self.action_counter > 50 //after 50 ticks
                {
                    //we may potentially use another sound here (custom)
                    state.sound_manager.play_sfx(58);

                    if (self.action_counter as i32 - 50) % 5 == 0 //every 5 ticks from then on
                    {
                        //starting framerect offset for the shot so we get a "ripple" effect when shooting
                        let ct_offset = (self.action_counter as i32 - 50) % 3;
    
                        let mut shot = NPC::create(393, &state.npc_table);
                        shot.cond.set_alive(true);
                        shot.npc_flags.set_ignore_solidity(true);
                        shot.x = self.x;
                        shot.y = self.y;
                        shot.anim_num = ct_offset as u16;
                        shot.action_counter2 = 50;
    
                        //target player
                        let mut angle = f64::atan2((self.y - self.target_y) as f64, (self.x - self.target_x) as f64);
                        angle += self.rng.range(-100..100) as f64 * 0.001;
                        shot.vel_x = (angle.cos() * -2048.0 * 1.0) as i32;
                        shot.vel_y = (angle.sin() * -2048.0 * 1.0) as i32;
    
                        let _ = npc_list.spawn(0x100, shot);
    
                    }
                }

                if self.action_counter > 80 {//after 400 ticks, reset to target
                    self.action_num = 20;
                }

                hover(self);

            }

            //knock back and slow down
            40 | 41 => {
                if self.action_num == 40 {
                    self.action_num = 41;
                    self.action_counter = 0;
                    self.anim_num = 8;
                    self.damage = 0;

                    //get location relative to player


                    //let mut angle = f64::atan2((player.y - self.y) as f64, (player.x - self.x) as f64);
                    //angle += self.rng.range(-100..100) as f64 * 0.001;
                    //self.vel_x = (angle.cos() * -2048.0 * 3.0) as i32;
                    //self.vel_y *= -1; (angle.sin() * -2048.0 * 3.0) as i32;

                    self.vel_y *= -1;
                    self.vel_x *= -1;

                }

                //slow down
                self.vel_x = 11 * self.vel_x / 12;
                self.vel_y = 11 * self.vel_y / 12;

                self.action_counter += 1;
                if self.action_counter > 40 {
                    self.action_num = 20;
                    self.action_counter = 0;
                }
            }
            _ => (),
        }

        //death event
        if self.life < life_const {
            self.cond.set_alive(false);
            //explode
            npc_list.create_death_smoke(self.x, self.y, 0x200 * 16, 4, state, &self.rng);

            state.sound_manager.play_sfx(35); //sound: big boom
        }

        self.x += self.vel_x; //if self.shock > 0 { self.vel_x / 2 } else { self.vel_x };
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 10 };

        self.anim_rect = state.constants.npc.n396_ravil_cyborg[self.anim_num as usize + dir_offset];

        Ok(())
    }


    


    //note: potentially unused
    pub(crate) fn tick_nXXX_rice_lightning(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {

        match self.action_num {
            0 | 1 => {
                //place target on player's Y
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.target_x = self.x;
                    self.target_y = self.y;
                    self.y = self.get_closest_player_mut(players).y;
                    state.sound_manager.play_sfx(103);
                }
                self.action_counter += 1;
                self.anim_num = 1 - (self.action_counter & 2) / 2;
                if self.direction == Direction::Left && self.action_counter == 20 {
                    let mut npc = NPC::create(146, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.target_x;
                    npc.y = self.target_y;
                    let _ = npc_list.spawn(0x100, npc);
                }

                if self.action_counter > 40 {
                    self.cond.set_alive(false);
                }
            }

            _ => (),
        }

        self.anim_rect = state.constants.npc.n333_ballos_lightning[self.anim_num as usize];

        Ok(())
    }


    //forms part of a "ray" shot from the gravity gun, pulls the player in a certain direction (and towards the center of the NPC) >|<
    pub(crate) fn tick_nxxx_gravity(
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


    //check for player within this bounding box
    fn test_hit_phys_entity_non_solid_bb(npc_1: &dyn PhysicalEntity, bounding_box: Option<Rect<u32>>, npc_2: &dyn PhysicalEntity) -> Flag {

        let mut flags = Flag(0);
        
        let npc_1_coords = (npc_1.x(), npc_1.y());
        let npc_2_coords = (npc_2.x(), npc_2.y());

        let npc_1_hit_bounds = if let Some(bb) = bounding_box {
            bb.clone()
        } else {
            npc_1.hit_bounds().clone()
        };
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
            //npc.target_y = npc.y;// - npc.hit_bounds.top as i32;
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

        //feature not needed
        // if (npc.y + npc.hit_bounds.bottom as i32) < npc.target_y {
        //     npc.y += hitbox_height;
        // } else if (npc.y - npc.hit_bounds.top as i32) > npc.target_y {
        //     npc.y -= hitbox_height;
        // }


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

            Rect::new(272,256,360,320), //shock (12)
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
            Rect::new(0,0,0,0), //hidden
        ];

        //have to do this here to appease the borrow checker
        //parent is the main TV screen, set on NPC creation
        let p_id = self.parts[i].parent_id as usize;
        let parent_npc_coords = (self.parts[p_id].x, self.parts[p_id].y);

        let npc = &mut self.parts[i];

        //offset relative to parent
        npc.x = parent_npc_coords.0;
        npc.y = parent_npc_coords.1 + 0x200 * (8 * 7);

        if npc.action_num == 0 {
            //nothing
            npc.anim_num = 2;
        } else {
            //change static
            if npc.rng.range(0..10) == 5 {
                if npc.anim_num == 0 {
                    npc.anim_num = 1;
                } else {
                    npc.anim_num = 0;
                }
            }   
        }

        npc.anim_rect = rc_tv_noise[npc.anim_num as usize];


    }

    //ticks the tv shuttle, tv screen, and tv noise
    fn tick_b11_rice_tv_shuttle(
        &mut self,
        npc_list: &NPCList,
        state: &mut SharedGameState,
        shuttle: usize,
        t_static: usize,
        screen: usize,
    ) {

        let tv_life = 20;
        let dummy_life = 8000; //some random number

        let rc_tv_shuttle = [
            Rect::new(208,160,272,200), //unlit
            Rect::new(208,200,272,240), //lit
            Rect::new(208,240,272,280), //shock
        ];

        let (npc, npc_screen, npc_static) = unsafe {
            let ptr = self.parts.as_mut_ptr();
            let a = ptr.add(shuttle).as_mut().unwrap();
            let b = ptr.add(screen).as_mut().unwrap();
            let c = ptr.add(t_static).as_mut().unwrap();
            (a, b, c)
        };

        
        //let npc = &mut self.parts[shuttle];

        //location to the left offscreen
        let x_min = X -(0x200 * 64);

        //location to the right offscreen
        let x_max = X + WIDTH + (0x200 * 64);

        match npc.action_num {

            //idle offscreen
            0 => {
                npc.x = x_min;
                npc.target_x = x_min;
            }

            //idle
            1 => {}


            //slide in, prep for offensive
            10 | 11 => {

                if npc.action_num == 10 {
                    //start offscreen
                    npc.x = x_min;
                    npc.target_x = 0x200 * 16 * 19; //roughly halfway

                    //start moving at a decent speed
                    npc.vel_x = 0x400;
                    npc.action_num = 11;

                    //snap TV screen to shuttle
                    //npc_screen.

                    //make npc duo shootable
                    npc_screen.npc_flags.set_shootable(true);
                    npc_screen.cond.set_alive(true);
                    npc_screen.life = dummy_life;

                    npc.npc_flags.set_shootable(true);
                    npc.cond.set_alive(true);
                    npc.life = dummy_life;
                }

                //passed target, start shooting at player
                if npc.x  > npc.target_x {
                    npc.action_num = 12;
                }
            
            }

            //shoot at PC
            12 => {
                
                npc.action_counter += 1;
                if npc.action_counter > 40 {
                    npc.action_counter2 += 1;
                    npc.action_counter = 0;

                    let mut ball = NPC::create(391, &state.npc_table);
                    ball.x = npc.x;
                    ball.y = npc.y;
                    ball.cond.set_alive(true);
                    state.sound_manager.play_sfx(47);
                    
                    //alternate between launch sides
                    if npc.action_counter2 % 2 == 0 {
                        //shoot from L
                        ball.x += -0x200 * 16;
                        ball.y += 0x200 * 16;

                        ball.vel_x = -0x200;
                        ball.vel_y = 0x200;

                    } else {
                        //shoot from R
                        ball.x += 0x200 * 16;
                        ball.y += 0x200 * 16;

                        ball.vel_x = 0x200;
                        ball.vel_y = 0x200;
                    }

                    let _ = npc_list.spawn(0x100, ball);
                }

            }

            //"die", speed forward, and crash into the puppet (TODO state 21: drop TV) 
            20 | 21=> {

                //let cutoff_width = X + WIDTH + npc_screen.hit_bounds.left as i32 + 0x200 * 16;
                if npc.x < x_max {
                    npc.vel_x += 0x16;
                } else {
                    npc.action_num = 0;

                    //life reset happens when the action number is changed to 20.
                    //npc_screen.life = dummy_life;
                    //npc.life = dummy_life;
                }

                npc.action_counter += 1;
                if npc.action_counter > 10 {

                    let radius = 40;

                    state.create_caret(
                        npc.x + npc.rng.range(-radius..radius) * 0x200,
                        npc.y + npc.rng.range(-radius..radius) * 0x200,
                        CaretType::Explosion,
                        Direction::Left
                    );
                    
                    npc_list.create_death_smoke_up(
                        npc.x + npc.rng.range(-radius..radius) * 0x200,
                        npc.y + npc.rng.range(-radius..radius) * 0x200,
                        16 as usize,
                        2,
                        state,
                        &npc.rng,
                    );

                    state.sound_manager.play_sfx(44);
                    npc.action_counter = 0;

                }
            }

            _ => {}
        }


        //move the shuttle back and forth around the target
        if npc.action_num == 10
        || npc.action_num == 11
        || npc.action_num == 12 {

            if npc.target_x < npc.x {
                npc.vel_x -= 0x05;
            }
            if npc.target_x > npc.x {
                npc.vel_x += 0x05;
            }

            let clamp_speed = (abs(npc.target_x - npc.x) / 0x50).clamp(0x100, 0x800);
            npc.vel_x = npc.vel_x.clamp(-clamp_speed, clamp_speed);
            npc.vel_y = npc.vel_y.clamp(-clamp_speed, clamp_speed);


            //force all shock values to be the same (not anymore: each part is separate now)
            npc_static.shock = npc_screen.shock;
            //npc.shock = npc_screen.shock;


        } else {
            //don't slide around
            if npc.action_num != 20 {
                npc.vel_x = 0;
            }
        }

        //detect "death" and start death sequence
        let curr_dmg = (dummy_life - npc_screen.life) + (dummy_life - npc.life);
        if curr_dmg >= tv_life
        && npc.action_num != 20
        && npc.action_num != 21 {
            npc_screen.npc_flags.set_shootable(false);
            npc.npc_flags.set_shootable(false);
            npc.action_num = 20;

            //reset life
            npc_screen.life = dummy_life;
            npc.life = dummy_life;
        }

        //half life left, show static
        if curr_dmg >= tv_life / 2 {
            npc_static.action_num = 1; //show static
        } else {
            npc_static.action_num = 0; //no static
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

    fn tick_b11_rice_grav_gun(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
        gun: usize,
        beam: usize,
        players: [&mut Player; 2],
    ) {

        let rc_grav_gun = [
            Rect::new(208,64,272,96), //unlit
            Rect::new(208,96,272,128), //lit
            Rect::new(208,128,272,160), //shock
        ];

        let (npc, npc_beam) = unsafe {
            let ptr = self.parts.as_mut_ptr();
            let a = ptr.add(gun).as_mut().unwrap();
            let b = ptr.add(beam).as_mut().unwrap();
            (a, b)
        };


        //move in,
        //home towards player,
        //shoot gravity,
        //repeat,
        //if life == 0, move forward and expose boss


        //location to the left offscreen
        let x_min = X -(0x200 * 64);
        //location to the right offscreen
        let x_max = X + WIDTH + (0x200 * 64);
        //speed of deceleration
        let decel_constant = 20;

        let sim_life = 20;
        let dummy_life = 8000; //some random number

        let [p1, p2] = players;
 
        let rc_player_at_top = Some(Rect::new(
            0x200 * 2 * 8,
            0x200 * 0,
            0x200 * 2 * 8,
            0x200 * 2 * 8,
        ));


        let target = npc.get_closest_player_mut([p1, p2]);

        match npc.action_num {

            //idle offscreen
            0 => {
                npc.npc_flags.set_shootable(false);
                npc.x = x_min;
                npc.target_x = x_min;

                //beam is off
                npc_beam.action_num = 0;

            }

            //idle
            1 => {
                npc.life = dummy_life;
            }

            //slide towards player, prep for offensive (lights off)
            10 | 11 => {

                //init
                if npc.action_num == 10 {
                    npc.action_num = 11;
                    //npc.target_x = target.x;

                    npc.action_counter = 0;
                    npc.npc_flags.set_shootable(true);
                    npc.cond.set_alive(true);
                    npc.life = dummy_life;

                    //beam is off
                    npc_beam.action_num = 0;

                    //get sign of current player location
                    npc.target_x = (npc.x - target.x).signum();
                }

                //unit is off
                npc.anim_num = 0;

                //accelerate toward player
                if npc.x < target.x {
                    npc.vel_x += 0x10;
                } else {
                    npc.vel_x -= 0x10;
                }
                npc.vel_x = npc.vel_x.clamp(-0x200, 0x200);


                //if we cross over the player, increment counter (like monster X)
                let curr_pc_sign = (npc.x - target.x).signum();
                if curr_pc_sign != npc.target_x
                && curr_pc_sign != 0 {
                    npc.action_counter += 1;

                    //cache player position
                    npc.target_x = curr_pc_sign;


                }

                //we crossed over X times, goto next action (keep following the player)
                if npc.action_counter > 4 {
                    npc.action_num = if npc.rng.range(0..20) < 10 { 20 } else { 30 }
                }

                npc.x += npc.vel_x;

            }

            //decelerate, shoot grav-well in-place, times out or reacts to player touch
            20 => {
                
                if npc.vel_x > decel_constant {
                    npc.vel_x -= decel_constant;
                } else if npc.vel_x < -decel_constant {
                    npc.vel_x += decel_constant;
                } else {
                    //goto shoot
                    npc.vel_x = 0;
                    npc.action_num = 21;
                    npc.action_counter = 0;

                    //beam is on
                    npc_beam.action_num = 1;
                }                
                
                npc.x += npc.vel_x;
            }

            //shoot gravity well
            21 => {
                
                //blink on
                npc.animate(5, 0, 1);

                npc.action_counter += 1;

                //shoot electricity if time expires or the player is sucked in
                let r = NPC::test_hit_phys_entity_non_solid_bb(npc, rc_player_at_top, target);
                if npc.action_counter > 200 || r.any_flag() {
                    //zap
                    npc.action_num = 40;
                }
            }
            
            //shoot gravity well (follow player, does not stop until the player touches the device)
            30 | 31 => {

                //blink on (faster)
                npc.animate(3, 0, 1);

                //init
                if npc.action_num == 30 {
                    npc.action_num = 31;

                    //beam is on
                    npc_beam.action_num = 1;
                }

                //accelerate toward player
                if npc.x < target.x {
                    npc.vel_x += 0x10;
                } else {
                    npc.vel_x -= 0x10;
                }
                npc.vel_x = npc.vel_x.clamp(-0x200, 0x200);


                //cache player position
                npc.target_x = target.x;

                //apply acceleration
                npc.x += npc.vel_x;


                //(touch player to go to next state)
                let r = NPC::test_hit_phys_entity_non_solid_bb(npc, rc_player_at_top, target);
                if r.any_flag() {
                    //zap
                    npc.action_num = 40;
                }

            }
            //decelerate
            40 => {

                if npc.vel_x > decel_constant {
                    npc.vel_x -= decel_constant;
                } else if npc.vel_x < -decel_constant {
                    npc.vel_x += decel_constant;
                } else {
                    //goto zap
                    npc.vel_x = 0;
                    npc.action_num = 41;
                    npc.action_counter = 0;

                    npc.anim_num = 1; //on
                }
                npc.x += npc.vel_x;
            }

            //zap lightning
            41 | 42 => {

                if npc.action_num == 41 {
                    let mut zap = NPC::create(333, &state.npc_table);
                    zap.cond.set_alive(true);
                    zap.x = npc.x;
                    zap.y = npc.y + 0x200 * 8 * 18;
                    let _ = npc_list.spawn(0x100, zap);

                    npc.action_num = 42;
                }

                npc.action_counter += 1;
                if npc.action_counter > 50 {
                    //return to follow
                    npc.action_num = 10;
                }

                npc.animate(0, 1, 2);

            }

            //die
            60 | 61 => {

                //init
                if npc.action_num == 60 {
                    npc.action_num = 61;
                    npc.action_counter = 0;

                }

                //accelerate until offscreen to the right
                if npc.x < x_max {
                    npc.vel_x += 0x16;
                } else {
                    npc.action_num = 0; //goto: idle offscreen
                }
                npc.x += npc.vel_x;


                //create smoke around NPC
                npc.action_counter += 1;
                if npc.action_counter > 10 {

                    let radius = 40;

                    state.create_caret(
                        npc.x + npc.rng.range(-radius..radius) * 0x200,
                        npc.y + npc.rng.range(-radius..radius) * 0x200,
                        CaretType::Explosion,
                        Direction::Left
                    );
                    
                    npc_list.create_death_smoke_up(
                        npc.x + npc.rng.range(-radius..radius) * 0x200,
                        npc.y + npc.rng.range(-radius..radius) * 0x200,
                        16 as usize,
                        2,
                        state,
                        &npc.rng,
                    );

                    state.sound_manager.play_sfx(44);
                    npc.action_counter = 0;

                }

            }



            _ => {}
        }

        //detect death
        if dummy_life - npc.life >= sim_life
        && npc.action_num != 60 {
            //goto action 60 ("die")
            npc.life = dummy_life;
            npc.npc_flags.set_shootable(false);
            npc_beam.anim_num = 0; //beam off
            npc.action_num = 60;
            

        }

        //for debugging current NPC state
        npc.action_counter3 = npc.action_counter;

        //hold beam below gun
        npc_beam.x = npc.x;
        npc_beam.y = npc.y + 16 * 0x200;
        npc_beam.vel_x = npc.vel_x;
        

        //factor in shock
        if npc.shock % 2 == 1 {
            npc.anim_num = 2;
        }

        npc.anim_rect = rc_grav_gun[npc.anim_num as usize];

        self.tick_b11_rice_grav_gun_beam(beam, [p1, p2]);


    }

    //sucks players toward the top of the beam
    fn tick_b11_rice_grav_gun_beam(
        &mut self,
        i: usize,
        players: [&mut Player; 2],
    ) {

        let rc_beam = [
            Rect::new(0,0,0,0), //unlit
            Rect::new(272,320,352,456), //lit 1
            Rect::new(352,320,432,456), //lit 2
            Rect::new(432,320,512,456), //lit 3
            Rect::new(512,320,592,456), //lit 4
        ];

        let npc = &mut self.parts[i];

        //the beam is shaped like a trapezoid
        let top_width = 8 * 0x200 / 2;
        let bottom_width = rc_beam[1].width() * 0x200 / 2;
        let height = rc_beam[1].height() as u32 * 0x200;

        //pre-calculate slope
        let slope = (height as f32) / (bottom_width - top_width) as f32; // "/"

        //let pc = npc.get_closest_player_mut(players);

        match npc.action_num {

            //idle off
            0 => {
                npc.anim_num = 0;
            }

            //blink on, "suck" player to top
            1 => {

                npc.animate(5, 1, 4);

                for pc in players {

                    if !pc.cond.alive() {
                        continue;
                    }
                    
                    //get hitbox width given player height
                    let curr_height = pc.y - npc.y;
                    let hitbox_width = curr_height as f32 / slope + top_width as f32;

                    let readable_width = hitbox_width / (0x200 as f32);
                    npc.action_counter3 = readable_width.abs() as u16;

                    //is within x range of trapezoid
                    if pc.x > npc.x - hitbox_width as i32
                    && pc.x < npc.x {
                        //PC is on the left of the NPC, push the PC right
                        pc.vel_x += 0x80;
                        pc.vel_y -= 0xC0;

                    } else if pc.x < npc.x + hitbox_width as i32
                    && pc.x > npc.x {
                        //pc is on the right of the NPC, push it left
                        pc.vel_x -= 0x80;
                        pc.vel_y -= 0xC0;

                    }
                }


            }

            //do nothing
            _ => {}
        }

        npc.anim_rect = rc_beam[npc.anim_num as usize];

    }

    fn tick_b11_rice_spike_plate(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
        spike_plate_id: usize,
        players: [&mut Player; 2],
    ) {

        let rc_spike_plate = [
            Rect::new(0,400,48,464), //retract
            Rect::new(48,400,96,464), //extend
            Rect::new(96,400,144,464), //retract (shock)
            Rect::new(144,400,192,464), //extend
        ];

        let npc = unsafe {
            let ptr = self.parts.as_mut_ptr();
            let a = ptr.add(spike_plate_id).as_mut().unwrap();
            a
        };


        //move in,
        //home towards player,
        //shoot gravity,
        //repeat,
        //if life == 0, move forward and expose boss


        //location to the left offscreen
        let x_min = X -(0x200 * 64);
        //location to the right offscreen
        let x_max = X + WIDTH + (0x200 * 64);
        //damage the spike does when the player touches it
        let spike_damage = 20;

        let sim_life = 20;
        let dummy_life = 8000; //some random number

        let [p1, p2] = players;


        let rc_player_at_top = Rect::new(
            0x200 * 3 * 8,
            0x200 * 2 * 8,
            0x200 * 3 * 8,
            0x200 * 8,
        );


        let target = npc.get_closest_player_mut([p1, p2]);

        match npc.action_num {

            //idle offscreen
            0 => {
                npc.life = dummy_life;
                npc.npc_flags.set_shootable(false);
                npc.x = x_min;
                npc.target_x = x_min;
            }

            //idle
            1 => {}

            //slide towards player
            10 | 11 => {

                //init
                if npc.action_num == 10 {
                    npc.action_num = 11;
                    //npc.target_x = target.x;

                    npc.action_counter = 0;
                    npc.npc_flags.set_shootable(true);
                    npc.cond.set_alive(true);
                    npc.life = dummy_life;

                    //get sign of current player location
                    npc.target_x = (npc.x - target.x).signum();
                }

                //unit is off
                npc.anim_num = 0;

                //accelerate toward player
                if npc.x < target.x {
                    npc.vel_x += 0x10;
                } else {
                    npc.vel_x -= 0x10;
                }
                npc.vel_x = npc.vel_x.clamp(-0x200, 0x200);


                //if we cross over the player, increment counter (like monster X)
                let curr_pc_sign = (npc.x - target.x).signum();
                if curr_pc_sign != npc.target_x
                && curr_pc_sign != 0 {
                    npc.action_counter += 1;

                    //cache player position
                    npc.target_x = curr_pc_sign;

                }

                npc.x += npc.vel_x;

            }


            //die
            60 | 61 => {

                //init
                if npc.action_num == 60 {
                    npc.action_num = 61;
                    npc.action_counter = 0;

                }

                //accelerate until offscreen to the right
                if npc.x < x_max {
                    npc.vel_x += 0x16;
                } else {
                    npc.action_num = 0; //goto: idle offscreen
                }
                npc.x += npc.vel_x;


                //create smoke around NPC
                npc.action_counter += 1;
                if npc.action_counter > 10 {

                    let radius = 40;

                    state.create_caret(
                        npc.x + npc.rng.range(-radius..radius) * 0x200,
                        npc.y + npc.rng.range(-radius..radius) * 0x200,
                        CaretType::Explosion,
                        Direction::Left
                    );
                    
                    npc_list.create_death_smoke_up(
                        npc.x + npc.rng.range(-radius..radius) * 0x200,
                        npc.y + npc.rng.range(-radius..radius) * 0x200,
                        16 as usize,
                        2,
                        state,
                        &npc.rng,
                    );

                    state.sound_manager.play_sfx(44);
                    npc.action_counter = 0;

                }

            }



            _ => {}
        }

        //spring player back
        let r = NPC::test_hit_phys_entity_non_solid_bb(npc, Some(rc_player_at_top), target);
        if r.any_flag() && npc.action_counter2 == 0 {
            target.vel_y -= 0x200 * 20; //todo: PC's vel_y is clamped vertically, limiting "springiness"
            target.vel_x += npc.vel_x;
            target.damage(spike_damage, state, npc_list);
            npc.action_counter2 = 20;
            state.sound_manager.play_sfx(47);
        }
        npc.action_counter2 = npc.action_counter2.saturating_sub(1);

        //detect death
        if dummy_life - npc.life >= sim_life
        && npc.action_num != 60 {
            //goto action 60 ("die")
            npc.life = dummy_life;
            npc.npc_flags.set_shootable(false);
            npc.action_num = 60;
        }

        //for debugging current NPC state
        npc.action_counter3 = npc.action_counter;


        npc.anim_num = if npc.action_counter2 != 0 {1} else {0};

        npc.anim_rect = rc_spike_plate[npc.anim_num as usize];


    }


    fn tick_b11_rice_platform(&mut self, i: usize, rc_no: usize) {

        let rc_platform_top = [
            Rect::new(0,208,72,304), //off
            Rect::new(72,208,144,304), //lit
        ];
        let rc_platform_bottom = [
            Rect::new(0,304,72,400), //off
            Rect::new(72,304,144,400), //lit
        ];

        let npc = &mut self.parts[i];

        //let display_width = (npc.display_bounds.left + npc.display_bounds.right) as i32;

        match npc.action_num {

            //initialize 
            0 => {
                npc.target_x = npc.x; //set target position to current position
                npc.action_num = 1; //freeze in place, idle
            }

            //move to position (same as TV shuttle)
            10 => {

                if npc.target_x < npc.x {
                    npc.vel_x -= 0x02;
                }
                if npc.target_x > npc.x {
                    npc.vel_x += 0x02;
                }
    
                let clamp_speed = (abs(npc.target_x - npc.x) / 0x50).clamp(0x50, 0x800);
                npc.vel_x = npc.vel_x.clamp(-clamp_speed, clamp_speed);

                npc.x += npc.vel_x;
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
        let rc = if rc_no == 0 {&rc_platform_top} else {&rc_platform_bottom};
        npc.anim_rect = rc[npc.anim_num as usize];


    }

    //animates the entire shield generator + puppet
    fn tick_b11_rice_shield_tube(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
        players: [&mut Player; 2],
        top: usize,
        bottom: usize,
        shield: usize,
        puppet: usize,
    ) {

        /*
        states, done using target_x from the top generator (its weird, I know.)
        
        0: idle with puppet slouch

        10, 11, tension puppet, then lower top

        20, 21, blast lower part and disable shield (then idle with shield off)


        */


        match self.parts[top].target_x {
            
            0 => {
                self.parts[top].action_num = 0; //idle offscreen top
                self.parts[bottom].action_num = 0; //idle offscreen right
                self.parts[shield].action_num = 0; //shield off
                self.parts[puppet].action_num = 0; //idle slouched
            }

            //move parts in, turn on shield when everything is in its place (init / regenerate)
            10 | 11 => {

                if self.parts[top].target_x == 10 {
                    self.parts[top].action_num = 10; //move down
                    self.parts[bottom].action_num = 10; //move in

                    //set up puppet
                    self.parts[puppet].action_num = 20; //puppet bob
                    self.parts[puppet].vel_y = 0x80; //set movement speed
                    self.parts[puppet].target_y = 0x200 * 8 * 8;
    
                    //idle, wait for shield
                    self.parts[top].target_x = 11;
                }

                //top AND bottom have reached their spot, turn on the shield
                if self.parts[top].action_num == 11
                && self.parts[bottom].action_num == 11 {
                    self.parts[shield].action_num = 20; //shield on

                    self.parts[top].target_x = 12; //idle
                }



            }
            //idle (shield has reached target)
            12 => {}


            //make vulnerable
            20 => {

                self.parts[shield].action_num = 0; //shield off
                self.parts[puppet].action_num = 40; //puppet bob faster

                //drop bottom arm
                self.parts[bottom].action_num = 20;

                //turn top arm off
                self.parts[top].action_num = 20;

                //idle
                self.parts[top].target_x = 21;

            }

            //note: these may potenitally be unused if we ANP the shield target directly from the main boss
            //shoot laser from bottom shield (does not stop, must be told to stop by ANPing the npc to 10)
            30 => {
                self.parts[shield].action_num = 30; //shoot shield
                self.parts[top].target_x = 31; //idle ON
            }
            //idle
            31 => {}
            
            _ => {}
        }

        self.tick_b11_rice_generator_top(top);
        self.tick_b11_rice_generator_bottom(bottom);
        self.tick_b11_rice_shield(state, npc_list, players, shield);
        self.tick_b11_rice_puppet(puppet);

    }


    //just the shield generator top
    fn tick_b11_rice_generator_top(&mut self, i: usize) {

        let rc_generator_top = [
            Rect::new(144,304,240,328), //off
            Rect::new(144,352,240,376), //lit
        ];

        let npc = &mut self.parts[i];


        //states
        /*
        sit above frame, idle
        
        spool up puppet, move down into frame

        idle, blink in low state


        */

        //where the generator will sit for the majority of the fight
        let active_position = Y + 0x200 * (16 * 4 + 8);

        match npc.action_num {

            //idle in retracted state
            0 => {
                npc.y = Y - 0x200 * 16 * 1;
            }

            //move down
            10 => {

                if npc.y < active_position{
                    npc.y += 0x200;
                } else {
                    npc.y = active_position;
                    npc.action_num = 11; //idle
                }

            }

            //idle, animate
            11 => {
                npc.animate(4, 0, 1);
            }

            //sit off
            20 => {
                npc.anim_num = 0;
            }



            _ => {}
        }

        npc.anim_rect = rc_generator_top[npc.anim_num as usize];


    }

    //just the shield generator bottom
    fn tick_b11_rice_generator_bottom(&mut self, i: usize) {

        let rc_generator_bottom = [
            Rect::new(144,328,240,352), //off
            Rect::new(144,376,240,400), //lit
        ];

        let parent_id = self.parts[i].parent_id;
        let parent_x = self.parts[parent_id as usize].x;
        let parent_y = self.parts[parent_id as usize].y;


        let npc = &mut self.parts[i];

        /*
            states:
            idle in retract position,

        */

        let y_offset = parent_y + 0x200 * 8 * 12;

        match npc.action_num {

            //initialize, idle in retract position
            0 => {
                npc.x = parent_x + 0x200 * 8 * 10;

                //ensure y is at correct level
                npc.y = y_offset;
            }
            //idle
            1 => {}

            //move in from right
            10 => {

                if npc.x > parent_x {
                    npc.x -= 0x200;
                } else {
                    npc.x = parent_x;
                    npc.action_num = 11; //idle
                }
                
                npc.y = y_offset;
            }
            11 => {
                //idle at position
                npc.x = parent_x;

                npc.y = y_offset;
            }


            //explode, fall off, and get carried away by platform (we need to run the personalized physics code in main boss function to append xm)
            20 | 21 => {

                if npc.action_num == 20 {
                    npc.action_num = 21;

                    //initial throw velocities
                    npc.vel_y = -0x200;
                    npc.vel_x = -0x400;
                }

                //append y momentum
                npc.vel_y += 0x40;
                npc.clamp_fall_speed();



                //halt the y momentum if we've made contact with the floor
                if npc.flags.hit_bottom_wall() {
                    npc.vel_y = 0;
                }

                npc.y += npc.vel_y;
                npc.x += npc.vel_x;

                npc.anim_num = 0; //off

                //if offscreen, return to idle state
                if npc.x + 0x200 * 4 * 16 < X
                || npc.y - 0x200 * 4 * 16 > Y + HEIGHT {
                    npc.action_num = 0;
                }
            }



            _ => {}
        }

        //if not in the destroyed state, animate
        if npc.action_num != 21 {
            npc.animate(4, 0, 1);
        }

        npc.anim_rect = rc_generator_bottom[npc.anim_num as usize];

    }

    //just the puppet (TODO: fix bobbing speed)
    fn tick_b11_rice_puppet(&mut self, i: usize) {

        let rc_puppet: [Rect<u16>; 4] = [
            Rect::new(0,64,48,208), //slouch
            Rect::new(48,64,96,208), //mid
            Rect::new(96,64,112,208), //hanging
            Rect::new(112,64,128,208), //hanging, shock
        ];

        //18 * 8

        //has different display rects depending on the sprite used, since the hanging sprite is 1 block wide, while the slouched ones are 3.
        let rc_disp_puppet = [
            Rect::new(
                0x200 * 24,
                0x200 * 8 * 17,
                0x200 * 24,
                0x200 * 8,
            ),
            Rect::new(
                0x200 * 8,
                0x200 * 8 * 17,
                0x200 * 8,
                0x200 * 8,
            ),
        ];

        //parent is top shield generator
        let parent_id = self.parts[i].parent_id;
        let parent_x = self.parts[parent_id as usize].x;
        let parent_y = self.parts[parent_id as usize].y;


        let npc = &mut self.parts[i];

        /*
        states:

        slouch,
        transition to taught,
        goto target_y linear

        prep + bob around target_y

        target_y is relative to parent

        */

        match npc.action_num {

            //idle slouched, location undefined
            0 => {
                npc.anim_num = 0;
                npc.vel_x = 0;
                npc.vel_y = 0;
                npc.action_counter = 0;
            }

            //tension lines, transisiton to floating
            1 => {
                npc.action_counter += 1;

                if npc.action_counter > 5 {
                    npc.action_counter = 0;
                    npc.anim_num += 1;

                    //reached end of transition animation, idle with tensioned strings
                    if npc.anim_num > 2 {
                        npc.anim_num = 2;
                        npc.action_num = 2;
                    }
                }
            }
            //idle tensioned
            2 => {
                npc.anim_num = 2;
                npc.vel_x = 0;
                npc.vel_y = 0;
                npc.action_counter = 0;
            }

            //goto position (requires vel_y be set)
            10 => {

                //snap to tensioned rect
                npc.anim_num = 2;

                //guard against negative velocities
                let absvel = abs(npc.vel_y);

                if npc.target_y + parent_y > npc.y {
                    npc.y += absvel;
                } else {
                    npc.y -= absvel;
                }

                //close enough, snap to location
                if abs(npc.y - (parent_y + npc.target_y)) < absvel {
                    npc.y = parent_y + npc.target_y;
                    npc.action_num = 2; //idle
                }
            }

            //bob
            20 | 21 => {
                if npc.action_num == 20 {
                    npc.vel_y = 0x20; //give some starting velocity
                    npc.action_num += 1;
                }

                if parent_y + npc.target_y < npc.y {
                    npc.vel_y -= 0x05;
                }
                if parent_y + npc.target_y > npc.y {
                    npc.vel_y += 0x05;
                }
                npc.vel_y = npc.vel_y.clamp(-0x80, 0x80);

                npc.anim_num = 2; //ensure correct rect (tensioned)
            }

            //bob faster
            40 | 41 => {
                if npc.action_num == 40 {
                    npc.vel_y = 0x20; //give some starting velocity
                    npc.action_num += 1;
                }

                if parent_y + npc.target_y < npc.y {
                    npc.vel_y -= 0x05;
                }
                if parent_y + npc.target_y > npc.y {
                    npc.vel_y += 0x05;
                }
                npc.vel_y = npc.vel_y.clamp(-0xA0, 0xA0);

                npc.anim_num = 2; //ensure correct rect (tensioned)
            }

            _ => {}
        }


        //ensure this NPC doesn't die
        npc.life = 10000;

        //factor in shock
        if npc.shock % 2 == 1 {
            npc.anim_num = 3;
        }



        //add velocities
        npc.x = parent_x;
        npc.y += npc.vel_y;


        npc.display_bounds = rc_disp_puppet[if npc.anim_num < 2 {0} else {1}];

        //get distance between puppet and parent y, and adjust the display + anim rects to crop off the extra string
        let dist_diff = npc.y - parent_y;

        npc.display_bounds.top = dist_diff as u32;

        let mut needed_rc = rc_puppet[npc.anim_num as usize];

        //how much we need to crop off the top of the rect
        
        let top_displ = needed_rc.height() as i32 - (dist_diff + npc.display_bounds.bottom as i32) / 0x200;
        needed_rc.top += top_displ as u16;

        npc.anim_rect = needed_rc;


    }

    //just the shield
    fn tick_b11_rice_shield(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
        players: [&mut Player; 2],
        i: usize,
    ) {

        let rc_shield = [
            Rect::new(0,0,0,0), //off
            Rect::new(128,64,160,144), //on
            Rect::new(160,64,192,144), //charge
        ];


        //parent is top shield generator
        let parent_id = self.parts[i].parent_id;
        let parent_x = self.parts[parent_id as usize].x;
        let parent_y = self.parts[parent_id as usize].y;


        let npc = &mut self.parts[i];

        let player = npc.get_closest_player_mut(players);

        match npc.action_num {

            //shield off
            0 => {
                npc.anim_num = 0;
                npc.npc_flags.set_bouncy(false);
                npc.npc_flags.set_invulnerable(false);
            }
            //shield on
            10 => {
                npc.anim_num = 1;

                npc.npc_flags.set_bouncy(true);
                npc.npc_flags.set_invulnerable(true);
            }
            //shield faltering
            20 => {
                npc.animate(2, 1, 2);

                npc.npc_flags.set_bouncy(true);
                npc.npc_flags.set_invulnerable(true);
            }

            //charge + shoot
            30 | 31 => {
                npc.animate(2, 1, 2);

                npc.npc_flags.set_bouncy(true);
                npc.npc_flags.set_invulnerable(true);

                //place target on player
                if npc.action_num == 30 {
                    npc.action_num = 31;
                    npc.action_counter = 0;

                    npc.target_x = player.x;
                    npc.target_y = player.y;

                    let mut tgt = NPC::create(394, &state.npc_table);
                    tgt.cond.set_alive(true);
                    tgt.x = npc.target_x;
                    tgt.y = npc.target_y;
                    tgt.action_counter2 = 50; //alive for 50 ticks
                    let _ = npc_list.spawn(0x100, tgt);

                    state.sound_manager.play_sfx(103); //chg sound
                }

                //shoot
                npc.action_counter += 1;
                if npc.action_counter > 50 //after 50 ticks
                {
                    //we may potentially use another sound here (custom)
                    state.sound_manager.play_sfx(58);

                    if (npc.action_counter as i32 - 50) % 2 == 0 //every 3 ticks from then on
                    {
                        //starting framerect offset for the shot so we get a "ripple" effect when shooting
                        let ct_offset = (npc.action_counter as i32 - 50) % 3;
    
                        let mut shot = NPC::create(393, &state.npc_table);
                        shot.cond.set_alive(true);
                        shot.npc_flags.set_ignore_solidity(true);
                        shot.x = npc.x;
                        shot.y = npc.y;
                        shot.anim_num = ct_offset as u16;
                        shot.action_counter2 = 200;
    
                        //target player
                        let angle = f64::atan2((npc.y - npc.target_y) as f64, (npc.x - npc.target_x) as f64);
    
                        shot.vel_x = (angle.cos() * -2048.0 * 3.0) as i32;
                        shot.vel_y = (angle.sin() * -2048.0 * 3.0) as i32;
    
                        let _ = npc_list.spawn(0x100, shot);
    
                    }
                }

                
            }




            _ => {}
        }

        //offset the shield
        npc.x = parent_x;
        npc.y = parent_y + 0x200 * 8 * 6;

        npc.anim_rect = rc_shield[npc.anim_num as usize];


    }

    //helper functions

    //sets the movement speed of the floor and roof (TODO: make this take an index list)
    fn set_rail_speed(&mut self, speed: i32) {

        for i in 12..=17 {
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

        //what was this, some sort of test, maybe?
        //let mut pp_w = pc_raw_hit_bounds.width();
        //pp_w += 1;


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


        //force these states for now
        //state.settings.noclip = true; 
        state.settings.god_mode = true;
        state.settings.infinite_booster = true;


        let rc_lightning = [
            Rect::new(0,288,240,304),
            Rect::new(0,304,240,320),
            Rect::new(0,320,240,336),
            Rect::new(0,336,240,352),
        ];

        //note: boss statics are at the top of the file

        //main boss life
        let boss_life = 200;
        let boss_prt2_life = 300;

        //NPC id list for easy draw order switching
        let main_id = 0; //main controller
        let platform_top_id = 1; //upper platform, hangs down
        let platform_bottom_id = 18; //lower platform, sits up
        let puppet_parts_id = (
            3, //platform top
            4, //platform bottom
            5, //shield
            6 //puppet
        );
        let grav_gun_id = (
            7, //gun
            8 //gravity
        );
        let tv_parts_id = (
            9, //static
            10, //screen
            11 //shuttle
        );
        let rails_id = (
            12, //top rails (interchangable)
            13,
            14
        );
        let floor_id = (
            15, //bottom rails (interchangable)
            16,
            17
        );
        let spike_plate_id = 19;



        //manage actions
        match self.parts[main_id].action_num {
            //init sub-parts
            0 => {

                //parts are interated backwards: 0 is drawn on top

                //"global" hurt sounds
                self.hurt_sound[main_id] = 52;


                //event controller
                {
                    let npc = &mut self.parts[main_id];
                    npc.action_counter = 0;
                    npc.action_num = 1; //idle mode
                    npc.npc_flags.set_event_when_killed(true);
                    npc.event_num = 1000;
                    npc.life = boss_life;
                    npc.cond.set_alive(true);

                    npc.x = X;
                    npc.y = Y;
                    

                    //no hurt voice: use
                    //if let Some(table_entry) = state.npc_table.get_entry(npc.npc_type) {
                    //    state.sound_manager.play_sfx(table_entry.hurt_sound);
                    //}
                    //state.sound_manager.play_sfx(self.boss.hurt_sound[idx]);
                }

                //user platforms
                {

                    let npc = &mut self.parts[platform_top_id];

                    npc.cond.set_alive(true);
                    npc.npc_flags.set_invulnerable(true);
                    npc.npc_flags.set_solid_hard(true);
                    npc.npc_flags.set_ignore_solidity(true);
                    
                    //origin in center of solid part of the platform
                    npc.display_bounds = Rect::new(
                        0x200 * 36,
                        0x200 * 8 * 11,
                        0x200 * 36,
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

                    //bottom

                    let npc = &mut self.parts[platform_bottom_id];

                    npc.cond.set_alive(true);
                    npc.npc_flags.set_invulnerable(true);
                    npc.npc_flags.set_solid_hard(true);
                    npc.npc_flags.set_ignore_solidity(true);
                    
                    //origin in center of solid part of the platform
                    npc.display_bounds = Rect::new(
                        0x200 * 36,
                        0x200 * 8 * 1,
                        0x200 * 36,
                        0x200 * 8 * 11,
                    );

                    npc.hit_bounds = Rect::new(
                        0x200 * 8 * 4,
                        0x200 * 8 * 1,
                        0x200 * 8 * 4,
                        0x200 * 8 * 1,
                    );



                    //starting locations of the platforms
                    self.parts[platform_top_id].x = 0x200 * 8 * -4 + X;
                    self.parts[platform_top_id].y = 0x200 * 8 * 15 + Y;

                    self.parts[platform_bottom_id].x = 0x200 * 8 * -4 + X;
                    self.parts[platform_bottom_id].y = 0x200 * 8 * 22 + Y;




                    //have the platforms initialize themselves immediately
                    self.tick_b11_rice_platform(platform_top_id, 0);
                    self.tick_b11_rice_platform(platform_bottom_id, 1);
                    // for i in 1..=2 {
                    //     self.tick_b11_rice_platform(i);
                    // }

                }

                //puppet parts
                {
                    //platforms, shield, and puppet

                    //platform top
                    let npc = &mut self.parts[puppet_parts_id.0];
                    npc.action_counter = 0;
                    npc.cond.set_alive(true);

                    npc.hit_bounds = Rect::new(
                        0x200 * 16,
                        0x200 * 8,
                        0x200 * 16,
                        0x200 * 8,
                    );

                    npc.display_bounds = Rect::new(
                        0x200 * 8 * 3,
                        0x200 * 8 * 2,
                        0x200 * 8 * 5,
                        0x200 * 8 * 1,
                    );

                    //on right side of screen
                    npc.x = X + WIDTH - 0x200 * (16 * 4);

                    //platform bottom
                    let npc = &mut self.parts[puppet_parts_id.1];
                    npc.action_counter = 0;
                    npc.parent_id = puppet_parts_id.0 as u16; //parent is top
                    npc.cond.set_alive(true);

                    npc.hit_bounds = Rect::new(
                        0x200 * 16,
                        0x200 * 8,
                        0x200 * 16,
                        0x200 * 8,
                    );

                    npc.display_bounds = Rect::new(
                        0x200 * 8 * 3,
                        0x200 * 8 * 1,
                        0x200 * 8 * 5,
                        0x200 * 8 * 2,
                    );


                    //shield
                    let npc = &mut self.parts[puppet_parts_id.2];
                    npc.parent_id = puppet_parts_id.0 as u16; //parent is top
                    npc.cond.set_alive(true);

                    npc.display_bounds = Rect::new(
                        0x200 * 16,
                        0x200 * 8 * 5,
                        0x200 * 16,
                        0x200 * 8 * 5,
                    );

                    npc.hit_bounds = Rect::new(
                        0x200 * 16,
                        0x200 * 8 * 5,
                        0x200 * 16,
                        0x200 * 8 * 5,
                    );


                    //puppet
                    let npc = &mut self.parts[puppet_parts_id.3];
                    npc.parent_id = puppet_parts_id.0 as u16; //parent is top
                    npc.cond.set_alive(true);

                    //this is the only NPC that actually counts toward the boss HP
                    npc.cond.set_damage_boss(true);
                    npc.npc_flags.set_shootable(true);

                    npc.hit_bounds = Rect::new(
                        0x200 * 8,
                        0x200 * 8,
                        0x200 * 8,
                        0x200 * 8,
                    );



                }

                //gravity gun
                {
                    //top
                    let npc = &mut self.parts[grav_gun_id.0];
                    npc.cond.set_alive(true);
                    npc.npc_flags.set_solid_hard(true);
                    npc.npc_flags.set_shootable(true);

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


                    npc.y = Y + (3 * 16) * 0x200;
                    npc.x = X;

                    //ray
                    let npc = &mut self.parts[grav_gun_id.1];
                    npc.cond.set_alive(true);
                    npc.display_bounds = Rect::new(
                        0x200 * 40,
                        0x200 * 0,
                        0x200 * 40,
                        0x200 * 8 * 17,
                    );
                    npc.parent_id = grav_gun_id.0 as u16;

                }

                //tv parts
                {
                    //static
                    let npc = &mut self.parts[tv_parts_id.0];

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
                    npc.parent_id = tv_parts_id.2 as u16; //shuttle

                    //screen
                    let npc = &mut self.parts[tv_parts_id.1];
                    
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
                    npc.parent_id = tv_parts_id.2 as u16; //shuttle

                    //shuttle

                    let npc = &mut self.parts[tv_parts_id.2];

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

                    npc.y = Y + (3 * 16) * 0x200;
                    npc.x = X; //this is offset back immediately anyway: starting point doesn't matter here

                    self.parts[tv_parts_id.2].action_num = 0;
                    self.tick_b11_rice_tv_shuttle(npc_list, state, tv_parts_id.2, tv_parts_id.0, tv_parts_id.1);



                }

                //rails + floor
                {

                    let rail_loc_list = [
                        16 * 0x200 * (-1),
                        16 * 0x200 * (17 - 1),
                        16 * 0x200 * ((17 * 2) -1),
                    ];

                    //start with top rail
                    for (id, x_loc) in zip( [rails_id.0, rails_id.1, rails_id.2], rail_loc_list) {
                        let npc = &mut self.parts[id];
                        
                        npc.cond.set_alive(true);
                        //npc.npc_flags.set_invulnerable(true);
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

                        npc.y = Y + (2 * 16 + 16) * 0x200;
                        npc.x = X + x_loc;// + (0x200 * 8 * 17);

                    }

                    //do floor
                    for (id, x_loc) in zip( [floor_id.0, floor_id.1, floor_id.2], rail_loc_list) {
                        let npc = &mut self.parts[id];

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

                        npc.y = Y + HEIGHT - (32) * 0x200;
                        npc.x = X + x_loc;// + (0x200 * 8 * 17);

                    }


                }

                //mashy spike plate (vs. aristotle)
                {
                    let npc = &mut self.parts[spike_plate_id];

                    //top
                    npc.cond.set_alive(true);
                    npc.npc_flags.set_solid_hard(true);
                    npc.npc_flags.set_shootable(true);

                    npc.hit_bounds = Rect::new(
                        0x200 * 3 * 8,
                        0x200 * 8,
                        0x200 * 3 * 8,
                        0x200 * 8,
                    );

                    npc.display_bounds = Rect::new(
                        0x200 * 3 * 8,
                        0x200 * 3 * 8,
                        0x200 * 3 * 8,
                        0x200 * 5 * 8,
                    );


                    npc.y = Y + HEIGHT - (2 * 16) * 0x200;
                    npc.x = X;
                }

                //test starting ANPs

                //slide in (tv positions pre-baked)
                self.parts[tv_parts_id.2].action_num = 10;

                self.parts[platform_top_id].action_num = 10; //begin moving
                self.parts[platform_top_id].target_x = 0x200 * 16 * 13; //move to here

                self.parts[platform_bottom_id].action_num = 10; //begin moving
                self.parts[platform_bottom_id].target_x = 0x200 * 16 * 16; //move to here

                self.set_rail_speed(-0x200 * 4);

            }


            //state: check for TV action 20 and if it's past the puppet.
            //if so, break puppet floor and disable shield

            //idle with screen, check for TV destruction
            10 => {

                //tv shuttle in "dead" states and moved past the platform
                //(or same with ggun)
                if (self.parts[tv_parts_id.2].action_num / 10 == 2
                && self.parts[tv_parts_id.2].x > self.parts[puppet_parts_id.0].x)
                || (self.parts[grav_gun_id.0].action_num / 10 == 6
                && self.parts[grav_gun_id.0].x > self.parts[puppet_parts_id.0].x) {

                    //to next action (disable shield and idle offline)
                    self.parts[main_id].action_num = 20;
                }


            }
            //wait x ammount of time (or until a certain ammount of health is depleted) before restoring shield
            20 | 21 => {

                if self.parts[main_id].action_num == 20 {
                    //disable puppet shield (with target_x)
                    self.parts[puppet_parts_id.0].target_x = 20;

                    //prep for offline idle
                    self.parts[main_id].action_counter = 0;
                    self.parts[main_id].action_num = 21;

                    //keep track of starting HP
                    self.parts[main_id].action_counter2 = self.parts[main_id].life;
                }


                self.parts[main_id].action_counter += 1;

                //did 20 damage or waited longer than 100 ticks
                if self.parts[main_id].action_counter > 200
                //todo: fix underflow error
                || self.parts[main_id].life <= self.parts[main_id].action_counter2.saturating_sub(30) {
                    //goto shield restore
                    self.parts[main_id].action_num = 30;
                }

            }

            //restore shield, shoot several bursts at the player
            30 | 31 => {

                
                if self.parts[main_id].action_num == 30 {
                    //restore puppet shield (ANP with target_x)
                    self.parts[puppet_parts_id.0].target_x = 10;

                    self.parts[main_id].action_counter = 0;
                    self.parts[main_id].action_num = 31;

                    //get a random shot count
                    self.parts[main_id].action_counter2 = self.parts[main_id].rng.range(4..6) as u16;
                }

                //bottom arm has finished sliding in, begin target+shooting sequence
                if self.parts[puppet_parts_id.0].target_x == 12 {

                    self.parts[main_id].action_counter += 1;

                    let initial_time = 50; //time to wait before starting any attacks
                    let shot_time = 50 + 30; //50 ticks to "charge", 30 ticks' worth of firing

                    //timeout and return to other actions
                    if self.parts[main_id].action_counter >= initial_time + shot_time * self.parts[main_id].action_counter2 {
                        //reset shield
                        self.parts[puppet_parts_id.0].target_x = 10;
                        self.parts[main_id].action_num = 50; //do return to idle
                    } else if self.parts[main_id].action_counter > initial_time {
                        //wait initial time, then shoot every "shot time"

                        if (self.parts[main_id].action_counter - initial_time) % shot_time == 0 {
                            self.parts[puppet_parts_id.2].action_num = 30; //shoot shield
                        }
                    }


                }


            }

            //return to idle
            50 => {

                //slide TV in if it died
                if self.parts[tv_parts_id.2].action_num == 0 {
                    self.parts[tv_parts_id.2].action_num = 10;
                }
                
                //slide in ggun if it died
                if self.parts[grav_gun_id.0].action_num == 0 {
                    self.parts[grav_gun_id.0].action_num = 10;
                }


                //goto "wait for TV destroy"
                self.parts[main_id].action_num = 10;

            }


            //begin part 2 phase, drop the floor
            100 | 101 => {

                if self.parts[main_id].action_num == 100 {
                    self.parts[main_id].action_counter = 0;
                    self.parts[main_id].action_num = 101;

                    //slide spike plate in
                    self.parts[spike_plate_id].action_num = 10;
                }


                //begin slide down

                //slide down
                let y_mult = self.parts[main_id].action_counter as i32 * 0x100;
                self.parts[main_id].action_counter += 1;
                for id in [floor_id.0, floor_id.1, floor_id.2] {

                    let npc = &mut self.parts[id];
                    npc.y = Y + HEIGHT - (32) * 0x200 + y_mult;
                }

                //lower platform is not lowered anymore
                //self.parts[platform_bottom_id].y = 0x200 * 8 * 22 + Y + y_mult;


                state.quake_counter = 10;


                if self.parts[main_id].action_counter > 200 {
                    self.parts[main_id].action_num = 102; //current state: idle
                }
                
            }

            //debug: ANP tube to move in
            200 => {
                self.parts[puppet_parts_id.0].target_x = 10; //begin move-in (use 3's target_x to govern the states of the entire tube)

                //slide TV in
                self.parts[tv_parts_id.2].action_num = 10;

                //slide ggun in 
                self.parts[grav_gun_id.0].action_num = 10;

                self.parts[main_id].action_num = 10; //idle
            }
            //tell TV to shoot
            201 => {
                self.parts[tv_parts_id.2].action_num = 12; //shoot mode
                self.parts[main_id].action_num = 10; //idle
            }


            _ => {
            }
        }

        //un-alived condition
        if self.parts[main_id].life == 0 {
            self.parts[main_id].life = boss_life;
            self.parts[main_id].cond.set_alive(true);
        }


        //run sub-parts
        {
            //I have to do this dumb thing so that we can re-use p1 and p2 without "moving" them
            let [p1, p2] = players;

            //run platforms
            // for i in [platform_top_id, platform_bottom_id] {
            //     self.tick_b11_rice_platform(i);
            // }
            
            self.tick_b11_rice_platform(platform_top_id, 0);
            self.tick_b11_rice_platform(platform_bottom_id, 1);

            //run tv
            self.tick_b11_rice_tv_shuttle(npc_list, state, tv_parts_id.2, tv_parts_id.0, tv_parts_id.1);

            //run ggun
            self.tick_b11_rice_grav_gun(state, npc_list, grav_gun_id.0, grav_gun_id.1, [p1, p2]);

            //run mashy plate
            self.tick_b11_rice_spike_plate(state, npc_list, spike_plate_id, [p1, p2]);

            //run moving rails
            for i in [rails_id.0, rails_id.1, rails_id.2] {
                self.tick_b11_rice_rail(i);
            }
            //run floor (+ collisions)
            for i in [floor_id.0, floor_id.1, floor_id.2] {
                self.tick_b11_rice_rail(i);
                self.run_all_collisions_on_npc([p1, p2], npc_list, i);
            }

            //run tube
            self.tick_b11_rice_shield_tube(state, npc_list, [p1, p2], puppet_parts_id.0, puppet_parts_id.1, puppet_parts_id.2, puppet_parts_id.3);


        }


    }




}