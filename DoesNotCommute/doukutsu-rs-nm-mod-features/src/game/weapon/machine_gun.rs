use crate::common::Direction;
use crate::game::caret::CaretType;
//use crate::game::player::{Player, TargetPlayer};
use crate::game::weapon::{Shooter, TargetShooter};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::BulletManager;

impl Weapon {
    pub(crate) fn tick_machine_gun(
        &mut self,
        player: &mut dyn Shooter,
        player_id: TargetShooter,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        const BULLETS: [u16; 3] = [10, 11, 12];

        if !player.shoot() {
            self.counter1 = 6;
            self.counter2 += 1;

            if (player.equip().has_turbocharge() && self.counter2 > 1) || self.counter2 > 4 {
                self.counter2 = 0;
                self.refill_ammo(1);
            }
            return;
        }

        if bullet_manager.count_bullets_multi(&BULLETS, player_id) > 4 {
            return;
        }

        // self.counter2 : recharge time counter
        self.counter1 += 1; // autofire counter

        if self.counter1 > 5 {
            self.counter1 = 0;

            let btype = match self.level {
                WeaponLevel::Level1 => 10,
                WeaponLevel::Level2 => 11,
                WeaponLevel::Level3 => 12,
                WeaponLevel::None => unreachable!(),
            };

            if !self.consume_ammo(1) {
                self.draw_empty(state, player.x(), player.y());
                return;
            }

            match () {
                _ if player.up() => {
                    if self.level == WeaponLevel::Level3 {
                        player.set_vel_y(player.vel_y() + 0x100);
                    }

                    match player.direction() {
                        Direction::Left => {
                            bullet_manager.create_bullet(
                                player.gun_offset_x() + (12 * 0x200),
                                player.gun_offset_y() + (3 * 0x200),
                                btype,
                                player_id,
                                Direction::Up,
                                &state.constants,
                            );
                            state.create_caret(
                                player.gun_offset_x() + (12 * 0x200), 
                                player.gun_offset_y() + (3 * 0x200), 
                                CaretType::Shoot, Direction::Left);
                        }
                        Direction::Right => {
                            bullet_manager.create_bullet(
                                player.gun_offset_x() + (11 * 0x200),
                                player.gun_offset_y() + (3 * 0x200),
                                btype,
                                player_id,
                                Direction::Up,
                                &state.constants,
                            );
                            state.create_caret(
                                player.gun_offset_x() + (11 * 0x200), 
                                player.gun_offset_y() + (3 * 0x200), 
                                CaretType::Shoot, Direction::Left);
                        }
                        _ => {}
                    }
                }
                _ if player.down() => {
                    if self.level == WeaponLevel::Level3 {
                        if player.vel_y() > 0 {
                            //player.vel_y /= 2;
                            player.set_vel_y(player.vel_y() / 2);
                        }
                        if player.vel_y() > -0x400 {
                            //player.vel_y = (player.vel_y - 0x200).max(-0x400);
                            player.set_vel_y((player.vel_y() - 0x200).max(-0x400));
                        }
                    }

                    match player.direction() {
                        Direction::Left => {
                            bullet_manager.create_bullet(
                                player.gun_offset_x() + (14 * 0x200),
                                player.gun_offset_y() + (13 * 0x200),
                                btype,
                                player_id,
                                Direction::Bottom,
                                &state.constants,
                            );
                            state.create_caret(
                                player.gun_offset_x() + (14 * 0x200), 
                                player.gun_offset_y() + (13 * 0x200), 
                                CaretType::Shoot, Direction::Left);
                        }
                        Direction::Right => {
                            bullet_manager.create_bullet(
                                player.gun_offset_x() + (9 * 0x200),
                                player.gun_offset_y() + (13 * 0x200),
                                btype,
                                player_id,
                                Direction::Bottom,
                                &state.constants,
                            );
                            state.create_caret(
                                player.gun_offset_x() + (9 * 0x200), 
                                player.gun_offset_y() + (13 * 0x200), 
                                CaretType::Shoot, Direction::Left);
                        }
                        _ => {}
                    }
                }
                _ => match player.direction() {
                    Direction::Left => {
                        bullet_manager.create_bullet(
                            player.gun_offset_x() + (5 * 0x200),
                            player.gun_offset_y() + (11 * 0x200),
                            btype,
                            player_id,
                            Direction::Left,
                            &state.constants,
                        );
                        state.create_caret(
                            player.gun_offset_x() + (5 * 0x200), 
                            player.gun_offset_y() + (11 * 0x200), 
                            CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(
                            player.gun_offset_x() + (18 * 0x200),
                            player.gun_offset_y() + (11 * 0x200),
                            btype,
                            player_id,
                            Direction::Right,
                            &state.constants,
                        );
                        state.create_caret(
                            player.gun_offset_x() + (18 * 0x200), 
                            player.gun_offset_y() + (11 * 0x200), 
                            CaretType::Shoot, Direction::Right);
                    }
                    _ => {}
                },
            }

            if self.level == WeaponLevel::Level3 {
                state.sound_manager.play_sfx(49);
            } else {
                state.sound_manager.play_sfx(32);
            }
        }
    }
}
