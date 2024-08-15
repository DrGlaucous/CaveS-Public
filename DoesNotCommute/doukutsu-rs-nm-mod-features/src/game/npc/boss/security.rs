use std::iter::{zip, IntoIterator};
use std::ops::{Range, RangeInclusive, RangeBounds};

use crate::common::{CDEG_RAD, Direction, Rect};
use crate::framework::error::GameResult;
use crate::components::nikumaru::NikumaruCounter;
use crate::game::caret::CaretType;
use crate::game::npc::boss::BossNPC;
use crate::game::npc::list::NPCList;
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::game::stage::Stage;
use crate::game::weapon::Shooter;
use crate::util::rng::RNG;
use crate::game::TimingMode;

impl NPC {

    //modifications: occasionally shoots clocks and breaks on death

    fn tick_b10_tesla_shooter(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        direction: Direction,
    ) -> GameResult {


        let rc_list = [
            //left
            Rect::new( 256, 0, 272, 16 ), //idle
            Rect::new( 272, 0, 288, 16 ), //charging
            Rect::new( 288, 0, 304, 16 ), //supercharging
            Rect::new( 304, 0, 320, 16 ), //dead

            //up
            Rect::new( 256, 16, 272, 32 ), //idle
            Rect::new( 272, 16, 288, 32 ), //charging
            Rect::new( 288, 16, 304, 32 ), //supercharging
            Rect::new( 304, 16, 320, 32 ), //dead

            //right
            Rect::new( 256, 32, 272, 48 ), //idle
            Rect::new( 272, 32, 288, 48 ), //charging
            Rect::new( 288, 32, 304, 48 ), //supercharging 
            Rect::new( 304, 32, 320, 48 ), //dead    
        
            //down
            Rect::new( 256, 48, 272, 64 ), //idle
            Rect::new( 272, 48, 288, 64 ), //charging
            Rect::new( 288, 48, 304, 64 ), //supercharging
            Rect::new( 304, 48, 320, 64 ), //dead
        
        ];

        let dir_offset = match direction {
            Direction::Left => 0,
            Direction::Up => 4,
            Direction::Right => 8,
            _ => 12,
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
        let infl_life = 1000;

        match self.action_num {
            0 | 1 | 2 => {

                //init life
                if self.action_num == 0 {
                    //set inflated life so we don't die when our life is destroyed
                    let set_life = self.life;
                    self.life = set_life.saturating_add(infl_life);
                    self.action_num = 1; 
                }

                //cycle start
                if self.action_num == 1 {
                    self.anim_num = 0;
                    self.action_num = 2;


                    self.action_counter2 = self.rng.range(1..3) as u16; //bullets to fire

                    if self.action_counter2 != 1 {
                        let mammoth = 9;
                        let apples = mammoth + 2;
                    }

                    self.action_counter3 = 0; //current number fired

                    //delay handler
                    self.action_counter = self.rng.range(70..120) as u16;

                }


                if self.action_counter != 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_counter = 40; //time before firing
                    self.action_num = 3;
                    self.anim_num = 2;
                }
            }
            3 => {
                
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
                        self.action_num = 1;

                    } else {
                        //assign delay for next cycle
                        self.action_counter = 20;
                    }

                    //shoot a bullet
                    let angle = f64::atan2((self.y - player.y()) as f64, (self.x - player.x()) as f64)
                            + (self.rng.range(-6..6) as f64 * CDEG_RAD);
    
                    //probability that the projectile will be a clock
                    let (mut npc, speed) = if self.rng.range(0..4) == 0 {
                        let mut npc = NPC::create(375, &state.npc_table);

                        npc.event_num = self.rng.range(3..8) as u16;
                        npc.action_counter2 = 1; //set to despawn
                        npc.action_counter3 = NikumaruCounter::seconds_to_ticks(8, TimingMode::_50Hz) as u16; //despawn time is 8 seconds (50 TPS)
                        (npc, -1536.0/2.0)

                    } else {
                        (NPC::create(389, &state.npc_table), -1536.0)
                    };

                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    npc.vel_x = (angle.cos() * speed) as i32;
                    npc.vel_y = (angle.sin() * speed) as i32;

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

        //set "destroyed" (we don't use this for now, )
        if self.life < infl_life  && self.action_num <= 3 {
            self.npc_flags.set_shootable(false);
            self.anim_num = 3;
            self.action_num = 4; //idle
        }

        self.anim_rect = rc_list[self.anim_num as usize + dir_offset];

        Ok(())
    }



    
}


impl BossNPC {
    

    fn tick_b10_security_screen (
        &mut self,
        i: usize,
    ) {
        let rc_list = [
            Rect::new(192, 96, 224, 112), // !
            Rect::new(224, 96, 256, 112), // shield down
            Rect::new(192, 112, 224, 128), // firing
            Rect::new(224, 112, 256, 128), // x
            Rect::new(192, 128, 224, 144), // blank
            Rect::new(224, 128, 256, 144), // lines
        ];

        let npc = &mut self.parts[i];
        
        if npc.direction == Direction::Left {
            npc.animate(6, 0, 1);
        } else {
            npc.anim_num = 1;
        }


        npc.anim_rect = if npc.anim_num != 0 {
            //graphic
            rc_list[npc.action_num.clamp(0, 5) as usize]
        } else {
            //blank
            rc_list[4]
        }



    }


    fn tick_b10_security_shield (
        &mut self,
        i: usize,
    ) {
        let rc_list = [
            Rect::new(128, 0, 144, 128),
            Rect::new(144, 0, 160, 128),
            Rect::new(160, 0, 176, 128),
            Rect::new(176, 0, 192, 128),
        ];

        let npc = &mut self.parts[i];

        if npc.direction == Direction::Left {
            npc.animate(2, 0, 3);
            npc.anim_rect = rc_list[npc.anim_num as usize];
            //npc.npc_flags.set_invulnerable(true);
        } else {
            npc.anim_rect = Rect::new(0,0,0,0);
            //npc.npc_flags.set_invulnerable(false);
        }

    }


    fn tick_b10_weakpoint (
        &mut self,
        i: usize,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) {

        let rc_list = [
            //flows
            Rect::new(192, 0, 208, 48),
            Rect::new(208, 0, 224, 48),
            Rect::new(224, 0, 240, 48),
            Rect::new(240, 0, 256, 48),

            //shock
            Rect::new(192, 48, 208, 96),
            //burnt
            Rect::new(208, 48, 224, 96),
            //off
            Rect::new(224, 48, 240, 96),
            

        ];

        let npc = &mut self.parts[i];

        let player = npc.get_closest_pseudo_player_mut(players, npc_list);

        match npc.action_num {
            //idle
            0 => {
                npc.animate(2, 0, 3);
            }
            //shoot (does not stop unless acted on)
            1 => {
                npc.animate(2, 0, 3);

                if npc.action_counter > 0 {
                    npc.action_counter -= 1;
                }
                else {
                    
                    //assign delay for next cycle
                    npc.action_counter = 7;

                    //shoot a bullet
                    let angle = f64::atan2((npc.y - player.y()) as f64, (npc.x - player.x()) as f64)
                            + (npc.rng.range(-6..6) as f64 * CDEG_RAD);
    
                    //probability that the projectile will be a clock
                    let mut bullet = if npc.rng.range(0..4) == 0 {
                        let mut bullet = NPC::create(375, &state.npc_table);

                        bullet.event_num = bullet.rng.range(5..8) as u16;
                        bullet.action_counter2 = 1; //set to despawn
                        bullet.action_counter3 = NikumaruCounter::seconds_to_ticks(8, TimingMode::_50Hz) as u16; //despawn time is 8 seconds (50 TPS)
                        bullet

                    } else {
                        NPC::create(389, &state.npc_table)
                    };

                    bullet.cond.set_alive(true);
                    bullet.x = npc.x;
                    bullet.y = npc.y;
                    bullet.vel_x = (angle.cos() * -1536.0) as i32;
                    bullet.vel_y = (angle.sin() * -1536.0) as i32;

                    let _ = npc_list.spawn(0x100, bullet);

                    if !player.cond().hidden() {
                        state.sound_manager.play_sfx(64);
                    }

                }
            }
            //smoke and die
            2 => {
                //make smoke
                let mut smoke = NPC::create(4, &state.npc_table);
                smoke.x = npc.x;
                smoke.y = npc.y;
                smoke.cond.set_alive(true);
                let _ = npc_list.spawn(0x100, smoke);

                //remove shootability
                npc.direction = Direction::Left;
                //idle "dead"
                npc.anim_num = 5;
                npc.action_num = 500;

            }
            //idle
            _ => {}
        }

        if npc.direction == Direction::Left {
            npc.npc_flags.set_shootable(false);
        } else {
            npc.npc_flags.set_shootable(true);
        }

        if npc.shock > 0 {
            npc.anim_rect = rc_list[4];
        } else {
            npc.anim_rect = rc_list[npc.anim_num as usize];
        }

    }



    fn hide_shield<R: RangeBounds<usize> + IntoIterator<Item = usize>> (
        &mut self,
        npcs: R,
    ) {
        for n in npcs {
            self.parts[n].direction = Direction::Right;
        }
    }
    fn show_shield<R: RangeBounds<usize> + IntoIterator<Item = usize>> (
        &mut self,
        npcs: R,
    ) {
        for n in npcs {
            self.parts[n].direction = Direction::Left;
        }
    }


    //make generators/tguns shootable and active, depending on the range fed into the function
    fn revive_b10_generators_or_tguns<R: RangeBounds<usize> + IntoIterator<Item = usize>> (
        &mut self,
        range: R,
        state: &mut SharedGameState,
    ) {

        for i in range {
            let cnpc = &mut self.parts[i];

            cnpc.npc_flags.set_shootable(true);

            //reset life
            if let Some(table_entry) = state.npc_table.get_entry(cnpc.npc_type) {
                let infl_life = 1000; //todo: make this dynamic with NPC
                cnpc.life = infl_life + table_entry.life;
            }


            cnpc.action_num = 1;


            // if cnpc.npc_type == 383 {
            //     cnpc.action_num = 1; //active action for generators
            // } else {
            //     cnpc.action_num = 0; //active action for tguns
            // }
        }
    }


    //make genrators unshootable and idle
    fn set_b10_generators_idle<R: RangeBounds<usize> + IntoIterator<Item = usize>> (
        &mut self,
        range: R,
    ) {

        for i in range {
            let cnpc = &mut self.parts[i];
            cnpc.npc_flags.set_shootable(false);
            cnpc.anim_num = 0; //idle rect
            cnpc.action_num = 2; //idle action
        }

    }

    //make genrators unshootable and dead
    fn set_b10_generators_dead<R: RangeBounds<usize> + IntoIterator<Item = usize>> (
        &mut self,
        range: R,
    ) {

        for i in range {
            let cnpc = &mut self.parts[i];
            cnpc.npc_flags.set_shootable(false);
            cnpc.anim_num = 2; //dead rect
            cnpc.action_num = 2; //idle action
        }

    }


    //make tguns unshootable and idle
    fn set_b10_tguns_idle<R: RangeBounds<usize> + IntoIterator<Item = usize>> (
        &mut self,
        range: R,
    ) {

        for i in range {
            let cnpc = &mut self.parts[i];
            cnpc.npc_flags.set_shootable(false);
            cnpc.anim_num = 0; //idle rect
            cnpc.action_num = 4; //idle action
        }

    }

    //make tguns unshootable and dead
    fn set_b10_tguns_dead<R: RangeBounds<usize> + IntoIterator<Item = usize>> (
        &mut self,
        range: R,
    ) {

        for i in range {
            let cnpc = &mut self.parts[i];
            cnpc.npc_flags.set_shootable(false);
            cnpc.anim_num = 3; //dead rect
            cnpc.action_num = 4; //idle action
        }

    }


    //main update section
    pub(crate) fn tick_b10_security(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        stage: &mut Stage,
    ) {

        let (x,y) = (
            (99 * 16 + 8) * 0x200,
            (23 * 16 + 8) * 0x200,
        );

        match self.parts[0].action_num {
            //init all sub-parts
            0 => {

                //parts are interated backwards: 0 is drawn on top

                //"global" hurt sounds
                self.hurt_sound[0] = 52;

                //event controller 0
                {
                    let npc = &mut self.parts[0];
                    npc.action_counter = 0;
                    npc.action_num = 6; //idle mode
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

                //screen 1
                {
                    let npc = &mut self.parts[1];
                    
                    npc.cond.set_alive(true);
                    npc.direction = Direction::Right;
                    npc.action_num = 5;
                    npc.display_bounds = Rect::new(
                        16 * 0x200,
                        8 * 0x200,
                        16 * 0x200,
                        8 * 0x200,
                    );

                    npc.x = x;
                    npc.y = y + (3 * 16 + 8) * 0x200;

                }
                //shield (2-5), 4 total
                {

                    let shield_loc_list = [
                        -(1 * 16 + 8 )* 0x200,// -(1 * 16 + 8 )* 0x200,
                        -(0 * 16 + 8 )* 0x200,// -(0 * 16 * 0x200 + 8),
                        (0 * 16 + 8 )* 0x200,// (0 * 16 * 0x200 + 8),
                        (1 * 16 + 8 )* 0x200,// (1 * 16 * 0x200 + 8),
                    ];


                    for (npc, x_loc) in zip( &mut self.parts[2..=5], shield_loc_list) {
                        npc.cond.set_alive(true);
                        npc.npc_flags.set_invulnerable(false);
                        npc.direction = Direction::Left;
                        npc.display_bounds = Rect::new(
                            8 * 0x200,
                            64 * 0x200,
                            8 * 0x200,
                            64 * 0x200,
                        );

                        npc.hit_bounds = Rect::new(
                            8 * 0x200,
                            64 * 0x200,
                            8 * 0x200,
                            64 * 0x200,
                        );

                        npc.x = x + x_loc;
                        npc.y = y;// - (2 * 16 * 0x200);

                    }

                }
                //center column weakpoint 6
                {
                    let npc = &mut self.parts[6];
                    npc.cond.set_alive(true);
                    npc.cond.set_damage_boss(true);
                    npc.direction = Direction::Left;
                    npc.life = 1000; //substanial so it doesn't actually "die"

                    npc.display_bounds = Rect::new(
                        8 * 0x200,
                        24 * 0x200,
                        8 * 0x200,
                        24 * 0x200,
                    );
                    npc.hit_bounds = Rect::new(
                        8 * 0x200,
                        24 * 0x200,
                        8 * 0x200,
                        24 * 0x200,
                    );

                    npc.x = x;
                    npc.y = y;



                }
                //tguns 7-10, x4
                {
                    let tgun_loc_list = [
                        (-10 * 16 * 0x200),
                        (10 * 16 * 0x200),
                        (-5 * 16 * 0x200),
                        (5 * 16 * 0x200),
                    ];
                    for (npc, x_loc) in zip( &mut self.parts[7..=10], tgun_loc_list) {
                        let mut tgun_npc = NPC::create(385, &state.npc_table);
                        tgun_npc.x = x + x_loc;
                        tgun_npc.y = y - (4 * 16 + 8) * 0x200;
                        tgun_npc.cond.set_alive(true);

                        tgun_npc.init_rng(x_loc);
                        *npc = tgun_npc;

                    }
                }
                //generators 11-14, x4
                {

                    //note for all spawned generators, set starting act_no to 1 so it doesn't try to gather its own children

                    let generator_loc_list = [
                        (-12 * 16 * 0x200),
                        (12 * 16 * 0x200),
                        (-7 * 16 * 0x200),
                        (7 * 16 * 0x200),
                    ];
                    for (npc, x_loc) in zip( &mut self.parts[11..=14], generator_loc_list) {
                        let mut gen_npc = NPC::create(383, &state.npc_table);
                        gen_npc.action_num = 1;
                        gen_npc.x = x + x_loc;
                        gen_npc.y = y - (4 * 16 + 8) * 0x200;
                        gen_npc.cond.set_alive(true);

                        *npc = gen_npc;

                    }

                    self.revive_b10_generators_or_tguns(11..=14, state);

                }

                //stalk: 15
                {
                    let npc = &mut self.parts[15];
                    npc.action_counter = 0;
                    npc.cond.set_alive(true);

                    npc.x = x;
                    npc.y = y;

                    npc.anim_rect = Rect::new(
                        0,
                        0,
                        128,
                        160,
                    );
                    npc.display_bounds = Rect::new(
                        64 * 0x200,
                        80 * 0x200,
                        64 * 0x200,
                        80 * 0x200,
                    );
                }

                //idle
                self.parts[0].action_num = 1;

            }

            //idle with lines
            1 => {
                
                //idle-ize generators
                self.set_b10_generators_idle(11..=14);
                //idle-ize Tguns
                self.set_b10_tguns_idle(7..=10);

                //idle-ize weakpoint
                self.parts[6].action_num = 0;
                self.parts[6].direction = Direction::Left;

                //no blink, show lines
                self.parts[1].action_num = 5;
                self.parts[1].direction = Direction::Right;

                self.hide_shield(2..=5);

                //goto idle
                self.parts[0].action_num = 500;

            }

            //idle destroyed
            2 => {

                //idle-ize generators
                self.set_b10_generators_dead(11..=14);
                //idle-ize Tguns
                self.set_b10_tguns_dead(7..=10);

                //destroy weakpoint
                self.parts[6].action_num = 2;

                //blink x
                self.parts[1].action_num = 3;
                self.parts[1].direction = Direction::Left;

                //goto idle
                self.parts[0].action_num = 500;
            }
            
            //idle destroyed (smoke)
            3 | 4 => {
                if self.parts[0].action_num == 3 {
                    //idle-ize generators
                    self.set_b10_generators_dead(11..=14);
                    //idle-ize Tguns
                    self.set_b10_tguns_dead(7..=10);

                    //blink x
                    self.parts[1].action_num = 3;
                    self.parts[1].direction = Direction::Left;

                    //goto idle
                    self.parts[0].action_num = 4;
                }

                let npc = &mut self.parts[0];
                npc.action_counter += 1;
                
                if npc.action_counter > 10 {
                    let mut smoke = NPC::create(4, &state.npc_table);
                    smoke.x = npc.rng.range((x - 20 * 0x200)..(x + 20 * 0x200));
                    smoke.y = npc.rng.range((y - 60 * 0x200)..(y + 60 * 0x200));
                    smoke.cond.set_alive(true);
                    let _ = npc_list.spawn(0x100, smoke);

                    npc.action_counter = 0;
                }


            }

            //idle pre-boss
            5 => {
                //prep generators
                self.revive_b10_generators_or_tguns(11..=14, state);
                //idle-ize Tguns
                self.set_b10_tguns_idle(7..=10);

                //idle-ize weakpoint
                self.parts[6].action_num = 0;
                self.parts[6].direction = Direction::Left;


                //blink '!'
                self.parts[1].action_num = 0;
                self.parts[1].direction = Direction::Left;

                self.show_shield(2..=5);

                //goto idle
                self.parts[0].action_num = 500;
            }

            //start/run boss
            10 | 11 | 12 => {

                //re-set life + start fight
                if self.parts[0].action_num == 10 {
                    //set life
                    self.parts[0].life = 300;
                    self.parts[0].action_num = 11;
                }
                //start fight (without resetting life)
                if self.parts[0].action_num == 11 {

                    //blink '!'
                    self.parts[1].action_num = 0;
                    self.parts[1].direction = Direction::Left;

                    //idle-ize weakpoint
                    self.parts[6].action_num = 0;
                    self.parts[6].direction = Direction::Left;

                    //prep generators + tguns
                    self.revive_b10_generators_or_tguns(7..=14, state);

                    self.show_shield(2..=5);

                    //begin running
                    self.parts[0].action_num = 12;

                    //play start sound
                    state.sound_manager.play_sfx(65); //spur MAX

                }

                //check if all generators are dead
                let mut all_dead = true;
                for i in 11..=14 {
                    let npc = &mut self.parts[i];

                    //generator isn't in "idle" mode
                    if npc.action_num <= 1 {
                        all_dead = false;
                        break;
                    } 
                }

                //goto drop shields action
                if all_dead {
                    self.parts[0].action_num = 13;
                }



            }
            //drop shields
            13 | 14 => {

                if self.parts[0].action_num == 13 {

                    //set state
                    self.hide_shield(2..=5);
                    self.set_b10_tguns_dead(7..=10);
                    
                    //mark current life (taking away too much life will end the cycle early)
                    self.parts[0].action_counter2 = self.parts[0].life;
                    
                    //make weakpoint shootable
                    self.parts[6].action_num = 0;
                    self.parts[6].direction = Direction::Right;

                    //blink 'shield down'
                    self.parts[1].action_num = 1;
                    self.parts[1].direction = Direction::Left;


                    //reset counter
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_num = 14;
                }

                self.parts[0].action_counter += 1;

                //max time: 1600

                //cycle through "disabled" phases
                if self.parts[0].action_counter > 300
                || self.parts[0].action_counter2 - self.parts[0].life > 100 {
                    //restart (all init work done in action 11)
                    self.parts[0].action_num = 11
                } else if self.parts[0].action_counter == 150 {
                    //begin firing
                    self.parts[6].action_num = 1;

                } else if self.parts[0].action_counter == 100 {
                    //set screen to "firing"
                    self.parts[1].action_num = 2;

                }


            }

            _ => {}
        }

        //run all sub-parts
        {
            //I have to do this dumb thing so that we can re-use p1 and p2 without "moving" them
            let [p1, p2] = players;
            
            self.tick_b10_security_screen(1);
            for i in 2..=5 {
                self.tick_b10_security_shield(i);
            }

            self.tick_b10_weakpoint(
                6,
                state,
                [p1, p2],
                npc_list,
            );



            for i in 7..=10 {
                let _ = self.parts[i].tick_b10_tesla_shooter(state, [p1, p2], npc_list, Direction::Bottom);
            }
            for i in 11..=14 {
                let _ = self.parts[i].tick_n383_shield_generator(state, npc_list);
            }
        }

    }
}


