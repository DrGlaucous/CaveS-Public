use crate::common::Direction;
use crate::game::caret::CaretType;
//use crate::game::player::{Player, TargetPlayer};
use crate::game::weapon::{Shooter, TargetShooter};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::BulletManager;

impl Weapon {
    pub(crate) fn tick_spur(
        &mut self,
        player: &mut dyn Shooter,
        player_id: TargetShooter,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        const BULLETS: [u16; 6] = [37, 38, 39, 40, 41, 42];

        let mut shoot = false;
        let btype;

        if player.shoot() {
            self.add_xp(if player.equip().has_turbocharge() { 3 } else { 2 }, player, state);
            self.counter1 += 1;

            if self.counter1 & 2 != 0 {
                match self.level {
                    WeaponLevel::Level1 => {
                        state.sound_manager.play_sfx(59);
                    }
                    WeaponLevel::Level2 => {
                        state.sound_manager.play_sfx(60);
                    }
                    WeaponLevel::Level3 => {
                        if let (_, _, false) = self.get_max_exp(&state.constants) {
                            state.sound_manager.play_sfx(61);
                        }
                    }
                    WeaponLevel::None => unreachable!(),
                }
            }
        } else if self.counter1 > 0 {
            shoot = true;
            self.counter1 = 0;
        }

        if let (_, _, true) = self.get_max_exp(&state.constants) {
            if self.counter2 == 0 {
                self.counter2 = 1;
                state.sound_manager.play_sfx(65);
            }
        } else {
            self.counter2 = 0;
        }

        let level = self.level;
        if !player.shoot() {
            self.reset_xp();
        }

        match level {
            WeaponLevel::Level1 => {
                btype = 6;
                shoot = false;
            }
            WeaponLevel::Level2 => btype = 37,
            WeaponLevel::Level3 => {
                if self.counter2 == 1 {
                    btype = 39;
                } else {
                    btype = 38;
                }
            }
            WeaponLevel::None => unreachable!(),
        }

        if bullet_manager.count_bullets_multi(&BULLETS, player_id) > 0 || !(player.trigger_shoot() || shoot)
        {
            return;
        }

        if !self.consume_ammo(1) {
            state.sound_manager.play_sfx(37);
        } else {
            match player.direction() {
                Direction::Left if player.up() => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (13 * 0x200),
                        player.gun_offset_y() + (4 * 0x200),
                        btype,
                        player_id,
                        Direction::Up,
                        &state.constants,
                    );
                    state.create_caret(
                        player.gun_offset_x() + (13 * 0x200), 
                        player.gun_offset_y() + (6 * 0x200), 
                        CaretType::Shoot, Direction::Left);
                }
                Direction::Right if player.up() => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (10 * 0x200),
                        player.gun_offset_y() + (8 * 0x200),
                        btype,
                        player_id,
                        Direction::Up,
                        &state.constants,
                    );
                    state.create_caret(
                        player.gun_offset_x() + (10 * 0x200), 
                        player.gun_offset_y() + (6 * 0x200), 
                        CaretType::Shoot, Direction::Left);
                }
                Direction::Left if player.down() => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (14 * 0x200),
                        player.gun_offset_y() + (9 * 0x200),
                        btype,
                        player_id,
                        Direction::Bottom,
                        &state.constants,
                    );
                    state.create_caret(
                        player.gun_offset_x() + (14 * 0x200), 
                        player.gun_offset_y() + (11 * 0x200), 
                        CaretType::Shoot, Direction::Left);
                }
                Direction::Right if player.down() => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (9 * 0x200),
                        player.gun_offset_y() + (9 * 0x200),
                        btype,
                        player_id,
                        Direction::Bottom,
                        &state.constants,
                    );
                    state.create_caret(
                        player.gun_offset_x() + (9 * 0x200), 
                        player.gun_offset_y() + (11 * 0x200), 
                        CaretType::Shoot, Direction::Left);
                }
                Direction::Left => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (9 * 0x200),
                        player.gun_offset_y() + (11 * 0x200),
                        btype,
                        player_id,
                        Direction::Left,
                        &state.constants,
                    );
                    state.create_caret(
                        player.gun_offset_x() + (7 * 0x200), 
                        player.gun_offset_y() + (11 * 0x200), 
                        CaretType::Shoot, Direction::Left);
                }
                Direction::Right => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (14 * 0x200),
                        player.gun_offset_y() + (11 * 0x200),
                        btype,
                        player_id,
                        Direction::Right,
                        &state.constants,
                    );
                    state.create_caret(
                        player.gun_offset_x() + (16 * 0x200), 
                        player.gun_offset_y() + (11 * 0x200), 
                        CaretType::Shoot, Direction::Right);
                }
                _ => {}
            }

            let sound = match btype {
                6 => 49,
                37 => 62,
                38 => 63,
                39 => 64,
                _ => 0,
            };

            state.sound_manager.play_sfx(sound);
        }
    }
}
