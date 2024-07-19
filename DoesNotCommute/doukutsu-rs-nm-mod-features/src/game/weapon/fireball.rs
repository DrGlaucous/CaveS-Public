use crate::common::Direction;
use crate::game::caret::CaretType;
//use crate::game::player::{Player, TargetPlayer};
use crate::game::weapon::{Shooter, TargetShooter};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::BulletManager;

impl Weapon {
    pub(crate) fn tick_fireball(
        &mut self,
        player: &dyn Shooter,
        player_id: TargetShooter,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        let max_bullets = self.level as usize + 1;
        if !player.trigger_shoot() || bullet_manager.count_bullets_multi(&[7, 8, 9], player_id) >= max_bullets {
            return;
        }

        let btype = match self.level {
            WeaponLevel::Level1 => 7,
            WeaponLevel::Level2 => 8,
            WeaponLevel::Level3 => 9,
            WeaponLevel::None => {
                unreachable!()
            }
        };

        if !self.consume_ammo(1) {
            // todo switch to first weapon
            return;
        }

        match player.direction() {
            Direction::Left if player.up() => {
                bullet_manager.create_bullet(
                    player.gun_offset_x() + (13 * 0x200), 
                    player.gun_offset_y() + (6 * 0x200), 
                    btype, player_id, Direction::Up, &state.constants);
                state.create_caret(
                    player.gun_offset_x() + (13 * 0x200), 
                    player.gun_offset_y() + (2 * 0x200), 
                    CaretType::Shoot, Direction::Left);
            }
            Direction::Right if player.up() => {
                bullet_manager.create_bullet(
                    player.gun_offset_x() + (10 * 0x200), 
                    player.gun_offset_y() + (6 * 0x200), 
                    btype, player_id, Direction::Up, &state.constants);
                state.create_caret(
                    player.gun_offset_x() + (10 * 0x200), 
                    player.gun_offset_y() + (2 * 0x200), 
                    CaretType::Shoot, Direction::Left);
            }
            Direction::Left if player.down() => {
                bullet_manager.create_bullet(
                    player.gun_offset_x() + (13 * 0x200), 
                    player.gun_offset_y() + (9 * 0x200), 
                    btype, player_id, Direction::Bottom, &state.constants);
                state.create_caret(
                    player.gun_offset_x() + (13 * 0x200), 
                    player.gun_offset_y() + (13 * 0x200), 
                    CaretType::Shoot, Direction::Left);
            }
            Direction::Right if player.down() => {
                bullet_manager.create_bullet(
                    player.gun_offset_x() + (10 * 0x200), 
                    player.gun_offset_y() + (9 * 0x200), 
                    btype, player_id, Direction::Bottom, &state.constants);
                state.create_caret(
                    player.gun_offset_x() + (10 * 0x200), 
                    player.gun_offset_y() + (13 * 0x200), 
                    CaretType::Shoot, Direction::Left);
            }
            Direction::Left => {
                bullet_manager.create_bullet(
                    player.gun_offset_x() + (9 * 0x200), 
                    player.gun_offset_y() + (10 * 0x200), 
                    btype, player_id, Direction::Left, &state.constants);
                state.create_caret(
                    player.gun_offset_x() + (5 * 0x200), 
                    player.gun_offset_y() + (10 * 0x200), 
                    CaretType::Shoot, Direction::Left);
            }
            Direction::Right => {
                bullet_manager.create_bullet(
                    player.gun_offset_x() + (14 * 0x200), 
                    player.gun_offset_y() + (10 * 0x200), 
                    btype, player_id, Direction::Right, &state.constants);
                state.create_caret(
                    player.gun_offset_x() + (18 * 0x200), 
                    player.gun_offset_y() + (10 * 0x200), 
                    CaretType::Shoot, Direction::Right);
            }
            _ => {}
        }

        state.sound_manager.play_sfx(34)
    }
}
