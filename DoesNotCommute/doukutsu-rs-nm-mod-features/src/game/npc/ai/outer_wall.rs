use num_traits::abs;

use crate::common::Direction;
use crate::framework::error::GameResult;
use crate::game::npc::list::NPCList;
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::util::rng::RNG;

impl NPC {
    pub(crate) fn tick_n212_sky_dragon(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.y -= 0x800;
                }
                self.animate(30, 0, 1);
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 2;
                    self.anim_counter = 0;
                    self.target_y = self.y - 0x2000;
                    self.target_x = self.x - 0xc00;
                    self.vel_y = 0;
                    self.npc_flags.set_ignore_solidity(true);

                    // Co-op
                    if players[1].cond.alive() {
                        let mut npc = NPC::create(370, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.parent_id = self.id;
                        npc.x = self.x;
                        npc.y = self.y;
                        npc.action_num = 200;
                        npc.direction = Direction::Right;

                        let _ = npc_list.spawn(0xAA, npc);
                    }
                }

                self.vel_x += if self.x >= self.target_x { -8 } else { 8 };
                self.vel_y += if self.y >= self.target_y { -8 } else { 8 };

                self.x += self.vel_x;
                self.y += self.vel_y;

                self.animate(5, 2, 3);
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.npc_flags.set_ignore_solidity(true);
                }
                self.vel_y += if self.y >= self.target_y { -0x10 } else { 0x10 };

                self.vel_x += 0x20;
                self.vel_x = self.vel_x.clamp(-0x600, 0x600);

                self.x += self.vel_x;
                self.y += self.vel_y;

                self.animate(2, 2, 3);
            }
            30 => {
                self.action_num = 31;

                let mut npc = NPC::create(297, &state.npc_table);
                npc.cond.set_alive(true);
                npc.parent_id = self.id;
                let _ = npc_list.spawn(0x100, npc);
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n212_sky_dragon[self.anim_num as usize];

        if players[0].equip.has_mimiga_mask() && self.anim_num > 1 {
            self.anim_rect.top += 40;
            self.anim_rect.bottom += 40;
        }

        // Switch uses the extra space on the sprite sheet for 2P's Curly
        if state.constants.is_switch {
            if self.anim_num <= 1 {
                self.anim_rect.top += 8;
                self.display_bounds.top = 0x2000;
            } else {
                self.display_bounds.top = 0x3000;
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n213_night_spirit(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        // Nicalis
        if state.constants.is_cs_plus && self.tsc_direction != 0 {
            self.direction = Direction::Right;
        }

        let player = self.get_closest_player_ref(&players);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.anim_num = 0;
                    self.target_x = self.x;
                    self.target_y = self.y;
                }
                if player.y > self.y - 0x1000 && player.y < self.y + 0x1000 {
                    self.y += if self.direction != Direction::Left { 0x1E000 } else { -0x1E000 };
                    self.action_num = 10;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.vel_y = 0;
                    self.npc_flags.set_shootable(true);
                }
            }
            10 => {
                self.animate(2, 1, 3);

                self.action_counter += 1;
                if self.action_counter > 200 {
                    self.action_num = 20;
                    self.action_counter = 0;
                    self.anim_num = 4;
                }
            }
            20 => {
                self.animate(2, 4, 6);

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 30;
                    self.action_counter = 0;
                    self.anim_num = 7;
                }
            }
            30 => {
                self.animate(2, 7, 9);

                self.action_counter += 1;
                if self.action_counter % 5 == 1 {
                    let mut npc = NPC::create(214, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;

                    npc.vel_y = self.rng.range(-0x200..0x200);
                    npc.vel_x = self.rng.range(2..12) * 0x80;
                    let _ = npc_list.spawn(0x100, npc);

                    state.sound_manager.play_sfx(21);
                }

                if self.action_counter > 50 {
                    self.action_num = 10;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            40 => {
                self.vel_y += if self.y >= self.target_y { -0x40 } else { 0x40 };
                self.vel_y = self.vel_y.clamp(-0x400, 0x400);
                self.y += if self.shock > 0 { self.vel_y / 2 } else { self.vel_y };

                self.animate(2, 4, 6);

                if player.y < self.target_y + 0x1E000 && player.y > self.target_y - 0x1E000 {
                    self.action_num = 20;
                    self.action_counter = 0;
                    self.anim_num = 4;
                }
            }
            _ => (),
        }

        if self.action_num > 9 && self.action_num <= 30 {
            self.vel_y += if self.y >= player.y { -0x19 } else { 0x19 };

            self.vel_y = self.vel_y.clamp(-0x400, 0x400);

            if self.flags.hit_top_wall() {
                self.vel_y = 0x200;
            }
            if self.flags.hit_bottom_wall() {
                self.vel_y = -0x200;
            }
            self.y += if self.shock > 0 { self.vel_y / 2 } else { self.vel_y };

            if player.y > self.target_y + 0x1E000 || player.y < self.target_y - 0x1E000 {
                self.action_num = 40;
            }
        }

        self.anim_rect = state.constants.npc.n213_night_spirit[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n214_night_spirit_projectile(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {


        if self.action_num == 0 {
            self.action_num = 1;
            self.npc_flags.set_ignore_solidity(true);
        }

        let mut anim_offset = 0;
        if self.action_num == 1 {

            self.animate(2, 0, 2);

            match self.direction {
                Direction::Left =>{
                    self.vel_x -= 0x19;
                    if self.vel_x < 0 {
                        self.npc_flags.set_ignore_solidity(false);
                    }
                }
                Direction::Up =>{
                    self.vel_y -= 0x19;
                    if self.vel_y < 0 {
                        self.npc_flags.set_ignore_solidity(false);
                    }
                    anim_offset = 3;
                }
                Direction::Right =>{
                    self.vel_x += 0x19;
                    if self.vel_x > 0 {
                        self.npc_flags.set_ignore_solidity(false);
                    }
                    anim_offset = 6;
                }
                Direction::Bottom =>{
                    self.vel_y += 0x19;
                    if self.vel_y > 0 {
                        self.npc_flags.set_ignore_solidity(false);
                    }
                    anim_offset = 9;
                }
                _ => {}
            }
            self.x += self.vel_x;
            self.y += self.vel_y;


            if self.flags.hit_anything() {
                npc_list.create_death_smoke(self.x, self.y, self.display_bounds.right as usize, 4, state, &self.rng);
                state.sound_manager.play_sfx(28);
                self.cond.set_alive(false);
            }
        }

        //for testing purposes only
        if self.action_num == 2 {
            self.x += self.vel_x;
            self.y += self.vel_y;

            if self.flags.hit_anything() {
                npc_list.create_death_smoke(self.x, self.y, self.display_bounds.right as usize, 4, state, &self.rng);
                state.sound_manager.play_sfx(28);
                self.cond.set_alive(false);
            }

        }

        self.anim_rect = state.constants.npc.n214_night_spirit_projectile[self.anim_num as usize + anim_offset];

        Ok(())
    }

    pub(crate) fn tick_n215_sandcroc_outer_wall(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.target_y = self.y;
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_ignore_solidity(false);
                    self.npc_flags.set_invulnerable(false);
                    self.npc_flags.set_solid_soft(false);
                }

                let player = self.get_closest_player_mut(players);
                if abs(self.x - player.x) < 0x1800 && player.y > self.y && player.y < self.y + 0x1000 {
                    self.action_num = 15;
                    self.action_counter = 0;
                }
            }
            15 => {
                self.action_counter += 1;
                if self.action_counter > 10 {
                    state.sound_manager.play_sfx(102);
                    self.action_num = 20;
                }
            }
            20 => {
                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_num += 1;
                    self.anim_counter = 0;
                }

                match self.anim_num {
                    3 => self.damage = 15,
                    4 => {
                        self.action_num = 30;
                        self.action_counter = 0;
                        self.npc_flags.set_shootable(true);
                    }
                    _ => (),
                }
            }
            30 => {
                self.damage = 0;
                self.npc_flags.set_solid_soft(true);

                self.action_counter += 1;
                if self.shock > 0 {
                    self.action_num = 40;
                    self.action_counter = 0;
                }
            }
            40 => {
                self.npc_flags.set_ignore_solidity(true);
                self.y += 0x200;
                self.action_counter += 1;
                if self.action_counter == 32 {
                    self.action_num = 50;
                    self.action_counter = 0;
                    self.npc_flags.set_solid_soft(false);
                    self.npc_flags.set_shootable(false);
                }
            }
            50 => {
                if self.action_counter > 99 {
                    self.y = self.target_y;
                    self.action_num = 0;
                    self.anim_num = 0;
                } else {
                    self.action_counter += 1;
                }
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n215_sandcroc_outer_wall[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n347_hoppy(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {

        //lock out if keyed and not hidden + camera liveness hack
        if !state.control_flags.control_enabled() && !state.control_flags.replay_mode() {

            self.anim_rect = state.constants.npc.n347_hoppy[self.anim_num as usize];
            if self.direction != Direction::Left {
                let height = self.anim_rect.height();
                self.anim_rect.top += height;
                self.anim_rect.bottom += height;
            }
            return Ok(());
        }


        let player = self.get_closest_pseudo_player_mut(players, &npc_list);

        let (
            slide_dy,
            hop_dx,
            relative_x,
            relative_y,
        ) = if self.direction == Direction::Left {
            (
                &mut self.y,
                &mut self.x,
                player.x(),
                player.y()
            )
        } else {
            (
                &mut self.x,
                &mut self.y,
                player.y(),
                player.x()
            )
        };

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                }

                self.anim_num = 0;
                //let player = self.get_closest_player_ref(&players);
                if relative_y < *slide_dy + 0x10000 && relative_y > *slide_dy - 0x10000 {
                    self.action_num = 10;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            10 => {
                self.action_counter += 1;
                if self.action_counter == 4 {
                    self.anim_num = 2;
                }

                if self.action_counter > 12 {
                    self.action_num = 12;
                    self.anim_num = 3;
                    self.vel_x = 0x700;
                    state.sound_manager.play_sfx(6);
                }
            }
            12 => {

                if relative_y < *slide_dy {
                    self.vel_y = -0xAA;
                } else {
                    self.vel_y = 0xAA;
                }

                if (self.direction == Direction::Left && self.flags.hit_left_wall())
                || (self.direction != Direction::Left && self.flags.hit_top_wall())
                {
                    self.action_num = 13;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    self.vel_x = 0;
                    self.vel_y = 0;
                } else {
                    self.vel_x -= 0x2A;
                    if self.vel_x < -0x5FF {
                        self.vel_x = -0x5FF;
                    }
                    *hop_dx += self.vel_x;
                    *slide_dy += self.vel_y;
                }
            }
            13 => {
                self.action_counter += 1;
                if self.action_counter == 2 {
                    self.anim_num = 1;
                }

                if self.action_counter == 6 {
                    self.anim_num = 0;
                }

                if self.action_counter > 16 {
                    self.action_num = 1;
                }
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n347_hoppy[self.anim_num as usize];

        if self.direction != Direction::Left {
            let height = self.anim_rect.height();
            self.anim_rect.top += height;
            self.anim_rect.bottom += height;
        }

        Ok(())
    }
}
