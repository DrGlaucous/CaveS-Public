use crate::common::Direction;
use crate::framework::error::GameResult;
use crate::game::npc::list::NPCList;
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::util::rng::RNG;

//all NPCs build off this class, defined by the functions placed in here.
//the game runs the corresponding functions based on /src/game/npc/mod.rs
impl NPC {


    //these functions will be used for refrence only
    pub fn tick_nXXX_quote_teleport_out(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 0;
                self.y -= 0x2000;
            }
            1 => {
                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.vel_y = -0x2ff;
                }
            }
            2 => {
                if self.vel_y > 0 {
                    self.hit_bounds.bottom = 0x2000;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_counter = 0;
                    self.action_num = 3;
                    self.anim_num = 0;
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 40 {
                    self.action_counter = 64;
                    self.action_num = 4;

                    state.sound_manager.play_sfx(29);
                }
            }
            4 => {
                self.anim_num = 0;
                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.cond.set_alive(false);
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };
        self.anim_rect = state.constants.npc.n111_quote_teleport_out[self.anim_num as usize + dir_offset];

        let offset = players[state.textscript_vm.executor_player.index()].get_texture_offset()
            + (state.get_skinsheet_offset() * state.tile_size.as_int() as u16 * 2);
        self.anim_rect.top += offset;
        self.anim_rect.bottom += offset;

        if self.action_num == 4 {
            self.anim_rect.bottom = self.anim_rect.top + self.action_counter / 4;

            if self.action_counter & 0x02 != 0 {
                self.anim_rect.left += 1;
            }
        }

        Ok(())
    }

    pub fn tick_nXXX_quote_teleport_in(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 0;
                self.anim_counter = 0;
                self.x += 0x2000;
                self.y += 0x1000;

                state.sound_manager.play_sfx(29);
            }
            1 => {
                self.action_counter += 1;
                if self.action_counter >= 64 {
                    self.action_num = 2;
                    self.action_counter = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 3;
                    self.anim_num = 1;
                    self.hit_bounds.bottom = 0x1000;
                }
            }
            3 => {
                if self.flags.hit_bottom_wall() {
                    self.action_counter = 0;
                    self.action_num = 4;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };
        self.anim_rect = state.constants.npc.n111_quote_teleport_out[self.anim_num as usize + dir_offset];

        let offset = players[state.textscript_vm.executor_player.index()].get_texture_offset()
            + (state.get_skinsheet_offset() * state.tile_size.as_int() as u16 * 2);
        self.anim_rect.top += offset;
        self.anim_rect.bottom += offset;

        if self.action_num == 1 {
            self.anim_rect.bottom = self.anim_rect.top + self.action_counter / 4;

            if self.action_counter & 0x02 != 0 {
                self.anim_rect.left += 1;
            }
        }

        Ok(())
    }

    pub(crate) fn tick_nXXX_quote(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 0;

                if self.tsc_direction > 10 {
                    let player = &players[state.textscript_vm.executor_player.index()];
                    self.x = player.x;
                    self.y = player.y;

                    self.direction =
                        Direction::from_int(self.tsc_direction.saturating_sub(10) as usize).unwrap_or(Direction::Left);
                } else {
                    self.direction = Direction::from_int(self.tsc_direction as usize).unwrap_or(Direction::Left);
                }
            }
            2 => {
                self.anim_num = 1;
            }
            10 => {
                self.action_num = 11;
                self.anim_num = 2;

                state.sound_manager.play_sfx(71);

                let mut npc = NPC::create(4, &state.npc_table);
                npc.cond.set_alive(true);
                npc.direction = Direction::Left;
                npc.x = self.x;
                npc.y = self.y;

                for _ in 0..4 {
                    npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                    npc.vel_y = self.rng.range(-0x600..0) as i32;

                    let _ = npc_list.spawn(0x100, npc.clone());
                }
            }
            11 => {
                self.anim_num = 2;
            }
            20 => {
                self.action_num = 21;
                self.action_counter = 63;

                state.sound_manager.play_sfx(29);
            }
            21 => {
                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.cond.set_alive(false);
                }
            }
            50 | 51 => {
                if self.action_num == 50 {
                    self.action_num = 51;
                    self.anim_num = 3;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 6 {
                        self.anim_num = 3;
                    }
                }

                self.x += self.direction.vector_x() * 0x200;
            }
            60 | 61 => {
                if self.action_num == 60 {
                    self.action_num = 61;
                    self.anim_num = 7;
                    self.target_x = self.x;
                    self.target_y = self.y;
                }

                self.target_y += 0x100;
                self.x = self.target_x + self.rng.range(-1..1) as i32 * 0x200;
                self.y = self.target_y + self.rng.range(-1..1) as i32 * 0x200;
            }
            70 | 71 => {
                if self.action_num == 70 {
                    self.action_num = 71;
                    self.action_counter = 0;
                    self.anim_num = 3;
                    self.anim_counter = 0;
                }

                self.x += (self.direction.vector_x() as i32 | 1) * 0x100;

                self.anim_counter += 1;
                if self.anim_counter > 8 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 6 {
                        self.anim_num = 3;
                    }
                }
            }
            80 => {
                self.anim_num = 8;
            }
            99 | 100 | 101 => {
                if self.action_num == 99 || self.action_num == 100 {
                    self.action_num = 101;
                    self.anim_num = 3;
                    self.anim_counter = 0;
                }

                self.vel_y += 0x40;
                self.clamp_fall_speed();

                if self.flags.hit_bottom_wall() {
                    self.vel_y = 0;
                    self.action_num = 102;
                }

                self.y += self.vel_y;
            }
            102 => {
                self.anim_counter += 1;
                if self.anim_counter > 8 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 6 {
                        self.anim_num = 3;
                    }
                }
            }
            // Curly Clone Grabbed Player (Switch)
            200 => {
                self.anim_num = 2;
                if let Some(parent) = self.get_parent_ref_mut(npc_list) {
                    self.x = parent.x;
                    self.vel_x = parent.vel_x;
                    self.y = parent.y;
                    self.vel_y = parent.vel_y;

                    if parent.action_counter3 == 0 {
                        self.cond.set_alive(false);
                    }
                }
            }
            _ => (),
        }

        //face left, offset is 0, otherise it is 10 (no explicit animation numbers for left and right, they are all lumped together)
        let dir_offset = if self.direction == Direction::Left { 0 } else { 10 };
        //set player rect to its correct animation number
        self.anim_rect = state.constants.npc.n150_quote[self.anim_num as usize + dir_offset];

        //teleport in, wipe rect bottom down
        if self.action_num == 21 {
            self.anim_rect.bottom = self.anim_rect.top + self.action_counter / 4;
        }

        //skin change based on mimiga mask for continuity
        let offset = players[state.textscript_vm.executor_player.index()].get_texture_offset()
            + (state.get_skinsheet_offset() * state.tile_size.as_int() as u16 * 2);
        self.anim_rect.top += offset;
        self.anim_rect.bottom += offset;

        Ok(()) //return
    }

    //looks exactly the same as the original quote AI, just here because 2 players
    pub(crate) fn tick_nXXX_second_quote(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        if !players[1].cond.alive() {
            self.cond.set_alive(false);
            return Ok(());
        }
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 0;

                if self.tsc_direction > 10 {
                    let player = &players[state.textscript_vm.executor_player.index() + 1 % 1];
                    self.x = player.x;
                    self.y = player.y;

                    self.direction =
                        Direction::from_int(self.tsc_direction.saturating_sub(10) as usize).unwrap_or(Direction::Left);
                } else {
                    self.direction = Direction::from_int(self.tsc_direction as usize).unwrap_or(Direction::Left);
                }
            }
            2 => {
                self.anim_num = 1;
            }
            10 => {
                self.action_num = 11;
                self.anim_num = 2;

                state.sound_manager.play_sfx(71);

                let mut npc = NPC::create(4, &state.npc_table);
                npc.cond.set_alive(true);
                npc.direction = Direction::Left;
                npc.x = self.x;
                npc.y = self.y;

                for _ in 0..4 {
                    npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                    npc.vel_y = self.rng.range(-0x600..0) as i32;

                    let _ = npc_list.spawn(0x100, npc.clone());
                }
            }
            11 => {
                self.anim_num = 2;
            }
            20 => {
                self.action_num = 21;
                self.action_counter = 63;

                state.sound_manager.play_sfx(29);
            }
            21 => {
                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.cond.set_alive(false);
                }
            }
            50 | 51 => {
                if self.action_num == 50 {
                    self.action_num = 51;
                    self.anim_num = 3;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 6 {
                        self.anim_num = 3;
                    }
                }

                self.x += self.direction.vector_x() * 0x200;
            }
            60 | 61 => {
                if self.action_num == 60 {
                    self.action_num = 61;
                    self.anim_num = 7;
                    self.target_x = self.x;
                    self.target_y = self.y;
                }

                self.target_y += 0x100;
                self.x = self.target_x + self.rng.range(-1..1) as i32 * 0x200;
                self.y = self.target_y + self.rng.range(-1..1) as i32 * 0x200;
            }
            70 | 71 => {
                if self.action_num == 70 {
                    self.action_num = 71;
                    self.action_counter = 0;
                    self.anim_num = 3;
                    self.anim_counter = 0;
                }

                self.x += (self.direction.vector_x() as i32 | 1) * 0x100;

                self.anim_counter += 1;
                if self.anim_counter > 8 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 6 {
                        self.anim_num = 3;
                    }
                }
            }
            80 => {
                self.anim_num = 8;
            }
            99 | 100 | 101 => {
                if self.action_num == 99 || self.action_num == 100 {
                    self.action_num = 101;
                    self.anim_num = 3;
                    self.anim_counter = 0;
                }

                self.vel_y += 0x40;
                self.clamp_fall_speed();

                if self.flags.hit_bottom_wall() {
                    self.vel_y = 0;
                    self.action_num = 102;
                }

                self.y += self.vel_y;
            }
            102 => {
                self.anim_counter += 1;
                if self.anim_counter > 8 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 6 {
                        self.anim_num = 3;
                    }
                }
            }
            200 => {
                self.anim_num = 9;
                if let Some(parent) = self.get_parent_ref_mut(npc_list) {
                    self.x = parent.x + parent.vel_x + 0xA00;
                    self.y = parent.y + parent.vel_y - 0x1C00;
                }
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 10 };
        self.anim_rect = state.constants.npc.n150_quote[self.anim_num as usize + dir_offset];

        if self.action_num == 21 {
            self.anim_rect.bottom = self.anim_rect.top + self.action_counter / 4;
        }

        let offset = players[state.textscript_vm.executor_player.index() + 1 % 1].get_texture_offset()
            + (state.get_skinsheet_offset() * state.tile_size.as_int() as u16 * 2);
        self.anim_rect.top += offset;
        self.anim_rect.bottom += offset;

        Ok(())
    }



    pub fn tick_mythcrew(
        &mut self, //self-refrence
        state: &mut SharedGameState, //global game state
        players: [&mut Player; 2], //where the players are (only 2?)
        //npc_list: &NPCList,
        playertype: u16, //what skin to wear (adam, jamie, kari, grant, tory)
    ) -> GameResult
    {

        //default: turn to face nearest player
        //A: talk
        //B: face static direction


        //test flipping
        let mut now = 0;
        match  std::time::SystemTime::now().duration_since( std::time::SystemTime::UNIX_EPOCH)
        {
            Ok(n) => 
            {
                now = n.as_millis();
            }
            Err(_) => panic!("SystemTime before UNIX EPOCH!")
        }
        self.angle = core::f32::consts::PI * (((now % 100000) as f32)/ 1000.0); 

        //anchor on NPC center
        self.anchor_x = ((self.anim_rect.right - self.anim_rect.left) / 2) as f32;
        self.anchor_y = ((self.anim_rect.bottom - self.anim_rect.top) / 2) as f32;

        //end test


        //switch statement
        match self.action_num
        {
            //turn to face nearest player
            0 =>
            {
                self.anim_counter = 0;
                self.anim_num = 0;
                let player = self.get_closest_player_ref(&players);
                if player.x < self.x {self.direction = Direction::Left} else {self.direction = Direction::Right}
            }
            //talk
            1 =>
            {
                //switch animations
                //if self.rng.range(0..1) == 0
                if true//self.anim_counter.wrapping_add(1) > 5
                {
                    self.anim_counter = 0;
                    self.anim_num = self.rng.range(0..2) as u16; //cycle between taling animations
                }
            }
            //face static direction
            _ =>
            {
                self.anim_counter = 0;
                self.anim_num = 0;
            }


        }

        //velocities
        self.vel_x = 0;
        self.vel_y += 0x20;

        //limit gravity
        if self.vel_y > 0x512 {self.vel_y = 0x512}
        else if self.vel_y < -0x512 {self.vel_y = -0x512}


        //switch direction
        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n371_adam[self.anim_num as usize + dir_offset];

        //shift down based on which skin we're wearing
        let pt_offset = (self.anim_rect.bottom - self.anim_rect.top) * playertype;
        self.anim_rect.top += pt_offset;
        self.anim_rect.bottom += pt_offset;


        return Ok(())

    }


    pub fn tick_n371_thru_n375(
        &mut self, //self-refrence
        state: &mut SharedGameState, //global game state
        players: [&mut Player; 2], //where the players are (only 2?)
        //npc_list: &NPCList,
    ) -> GameResult
    {
        match self.npc_type
        {
            //adam
            371 =>{return self.tick_mythcrew(state, players, 0)}
            //jamie
            372 =>{return self.tick_mythcrew(state, players, 1)}
            //kari
            373 =>{return self.tick_mythcrew(state, players, 2)}
            //grant
            374 =>{return self.tick_mythcrew(state, players, 3)}
            //tory (also default because rust requires it)
            375 | _ =>{return self.tick_mythcrew(state, players, 4)}
        }
    }




}
