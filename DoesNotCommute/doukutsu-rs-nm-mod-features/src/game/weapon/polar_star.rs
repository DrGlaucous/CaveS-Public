use crate::common::Direction;
use crate::game::caret::CaretType;
//use crate::game::player::{Player, TargetPlayer};
use crate::game::weapon::{Shooter, TargetShooter};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::BulletManager;

impl Weapon {
    pub(crate) fn tick_polar_star(
        &mut self,
        player: &dyn Shooter,
        player_id: TargetShooter,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        if !player.trigger_shoot() || bullet_manager.count_bullets_multi(&[4, 5, 6], player_id) > 1 {
            return;
        }

        let btype = match self.level {
            WeaponLevel::Level1 => 4,
            WeaponLevel::Level2 => 5,
            WeaponLevel::Level3 => 6,
            WeaponLevel::None => unreachable!(),
        };

        if !self.consume_ammo(1) {
            state.sound_manager.play_sfx(37);
            return;
        }

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
                    player.x() + 0x200 + (9 * 0x200), 
                    player.y() + 0x1000 + (11 * 0x200), 
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

        if self.level == WeaponLevel::Level3 {
            state.sound_manager.play_sfx(49);
        } else {
            state.sound_manager.play_sfx(32);
        }
    }
}
