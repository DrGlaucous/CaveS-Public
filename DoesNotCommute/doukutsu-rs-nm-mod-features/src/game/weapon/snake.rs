use crate::common::Direction;
use crate::game::caret::CaretType;
//use crate::game::player::{Player, TargetPlayer};
use crate::game::weapon::{Shooter, TargetShooter};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::{Bullet, BulletManager};

impl Weapon {
    pub(crate) fn tick_snake(&mut self, player: &dyn Shooter, player_id: TargetShooter, bullet_manager: &mut BulletManager, state: &mut SharedGameState) {
        if !player.trigger_shoot() || bullet_manager.count_bullets_multi(&[1, 2, 3], player_id) > 3 {
            return;
        }

        let btype = match self.level {
            WeaponLevel::Level1 => 1,
            WeaponLevel::Level2 => 2,
            WeaponLevel::Level3 => 3,
            WeaponLevel::None => unreachable!(),
        };

        if !self.consume_ammo(1) {
            // todo switch to first weapon
            return;
        }

        self.counter1 = self.counter1.wrapping_add(1);

        match player.direction() {
            Direction::Left if player.up() => {
                let mut bullet = Bullet::new(
                    player.gun_offset_x() + (14 * 0x200), 
                    player.gun_offset_y() + (6 * 0x200), 
                    btype, player_id, Direction::Up, &state.constants);
                bullet.target_x = self.counter1 as i32;
                bullet_manager.push_bullet(bullet);
                state.create_caret(
                    player.gun_offset_x() + (14 * 0x200), 
                    player.gun_offset_y() + (2 * 0x200), 
                    CaretType::Shoot, Direction::Left);
            }
            Direction::Right if player.up() => {
                let mut bullet = Bullet::new(
                    player.gun_offset_x() + (9 * 0x200), 
                    player.gun_offset_y() + (6 * 0x200), 
                    btype, player_id, Direction::Up, &state.constants);
                bullet.target_x = self.counter1 as i32;
                bullet_manager.push_bullet(bullet);
                state.create_caret(
                    player.gun_offset_x() + (9 * 0x200), 
                    player.gun_offset_y() + (2 * 0x200), 
                    CaretType::Shoot, Direction::Left);
            }
            Direction::Left if player.down() => {
                let mut bullet = Bullet::new(
                    player.gun_offset_x() + (13 * 0x200), 
                    player.gun_offset_y() + (9 * 0x200), 
                    btype, player_id, Direction::Bottom, &state.constants);
                bullet.target_x = self.counter1 as i32;
                bullet_manager.push_bullet(bullet);
                state.create_caret(
                    player.gun_offset_x() + (13 * 0x200), 
                    player.gun_offset_y() + (13 * 0x200), 
                    CaretType::Shoot, Direction::Left);
            }
            Direction::Right if player.down() => {
                let mut bullet = Bullet::new(
                    player.gun_offset_x() + (10 * 0x200), 
                    player.gun_offset_y() + (9 * 0x200), 
                    btype, player_id, Direction::Bottom, &state.constants);
                bullet.target_x = self.counter1 as i32;
                bullet_manager.push_bullet(bullet);
                state.create_caret(
                    player.gun_offset_x() + (10 * 0x200), 
                    player.gun_offset_y() + (13 * 0x200), 
                    CaretType::Shoot, Direction::Left);
            }
            Direction::Left => {
                let mut bullet = Bullet::new(
                    player.gun_offset_x() + (9 * 0x200), 
                    player.gun_offset_y() + (10 * 0x200), 
                    btype, player_id, Direction::Left, &state.constants);
                bullet.target_x = self.counter1 as i32;
                bullet_manager.push_bullet(bullet);
                state.create_caret(
                    player.gun_offset_x() + (5 * 0x200), 
                    player.gun_offset_y() + (10 * 0x200), 
                    CaretType::Shoot, Direction::Left);
            }
            Direction::Right => {
                let mut bullet = Bullet::new(
                    player.gun_offset_x() + (14 * 0x200), 
                    player.gun_offset_y() + (10 * 0x200), 
                    btype, player_id, Direction::Right, &state.constants);
                bullet.target_x = self.counter1 as i32;
                bullet_manager.push_bullet(bullet);
                state.create_caret(
                    player.gun_offset_x() + (18 * 0x200), 
                    player.gun_offset_y() + (10 * 0x200), 
                    CaretType::Shoot, Direction::Right);
            }
            _ => {}
        }

        state.sound_manager.play_sfx(33);
    }
}
