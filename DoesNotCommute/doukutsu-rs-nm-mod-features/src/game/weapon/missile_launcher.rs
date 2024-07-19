use crate::common::Direction;
use crate::game::caret::CaretType;
//use crate::game::player::{Player, TargetPlayer};
use crate::game::weapon::{Shooter, TargetShooter};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::{Bullet, BulletManager};

impl Weapon {
    pub(crate) fn tick_missile_launcher(
        &mut self,
        player: &dyn Shooter,
        player_id: TargetShooter,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        const BULLETS: [u16; 6] = [13, 14, 15, 16, 17, 18];

        if !player.trigger_shoot() {
            return;
        }

        let btype = match self.level {
            WeaponLevel::Level1 => 13,
            WeaponLevel::Level2 => 14,
            WeaponLevel::Level3 => 15,
            WeaponLevel::None => unreachable!(),
        };

        match self.level {
            WeaponLevel::Level1 if bullet_manager.count_bullets_multi(&BULLETS, player_id) > 0 => {
                return;
            }
            WeaponLevel::Level2 if bullet_manager.count_bullets_multi(&BULLETS, player_id) > 1 => {
                return;
            }
            WeaponLevel::Level3 if bullet_manager.count_bullets_multi(&BULLETS, player_id) > 3 => {
                return;
            }
            _ => {}
        }

        if !self.consume_ammo(1) {
            self.draw_empty(state, player.x(), player.y());
            return;
        }

        match player.direction() {
            Direction::Left if player.up() => {
                let mut bullet =
                    Bullet::new(
                        player.gun_offset_x() + (13 * 0x200), 
                        player.gun_offset_y() + (5 * 0x200), 
                        btype, player_id, Direction::Up, &state.constants);
                
                bullet_manager.push_bullet(bullet.clone());

                state.create_caret(
                    player.gun_offset_x() + (13 * 0x200), 
                    player.gun_offset_y() + (1 * 0x200), 
                    CaretType::Shoot, Direction::Left);

                if self.level == WeaponLevel::Level3 {
                    bullet.x = player.gun_offset_x() + (11 * 0x200);
                    bullet.y = player.gun_offset_y() + (12 * 0x200);
                    bullet.counter2 = 1;
                    bullet_manager.push_bullet(bullet.clone());

                    bullet.x = player.gun_offset_x() + (17 * 0x200);
                    bullet.counter2 = 2;
                    bullet_manager.push_bullet(bullet);
                }
            }
            Direction::Right if player.up() => {
                let mut bullet =
                    Bullet::new(
                        player.gun_offset_x() + (10 * 0x200), 
                        player.gun_offset_y() + (5 * 0x200), 
                        btype, player_id, Direction::Up, &state.constants);

                bullet_manager.push_bullet(bullet.clone());

                state.create_caret(
                    player.gun_offset_x() + (10 * 0x200), 
                    player.gun_offset_y() + (1 * 0x200), 
                    CaretType::Shoot, Direction::Left);

                if self.level == WeaponLevel::Level3 {
                    bullet.x = player.gun_offset_x() + (6 * 0x200);
                    bullet.y = player.gun_offset_y() + (12 * 0x200);
                    bullet.counter2 = 1;
                    bullet_manager.push_bullet(bullet.clone());

                    bullet.x = player.gun_offset_x() + (12 * 0x200);
                    bullet.counter2 = 2;
                    bullet_manager.push_bullet(bullet);
                }
            }
            Direction::Left if player.down() => {
                let mut bullet = Bullet::new(
                    player.gun_offset_x() + (14 * 0x200),
                    player.gun_offset_y() + (10 * 0x200),
                    btype,
                    player_id,
                    Direction::Bottom,
                    &state.constants,
                );

                bullet_manager.push_bullet(bullet.clone());

                state.create_caret(
                    player.gun_offset_x() + (14 * 0x200), 
                    player.gun_offset_y() + (14 * 0x200), 
                    CaretType::Shoot, Direction::Left);

                if self.level == WeaponLevel::Level3 {
                    bullet.x = player.gun_offset_x() + (16 * 0x200);
                    bullet.y = player.gun_offset_y() + (3 * 0x200);
                    bullet.counter2 = 1;
                    bullet_manager.push_bullet(bullet.clone());

                    bullet.x = player.gun_offset_x() + (10 * 0x200);
                    bullet.counter2 = 2;
                    bullet_manager.push_bullet(bullet);
                }
            }
            Direction::Right if player.down() => {
                let mut bullet = Bullet::new(
                    player.gun_offset_x() + (9 * 0x200),
                    player.gun_offset_y() + (10 * 0x200),
                    btype,
                    player_id,
                    Direction::Bottom,
                    &state.constants,
                );

                bullet_manager.push_bullet(bullet.clone());

                state.create_caret(
                    player.gun_offset_x() + (9 * 0x200), 
                    player.gun_offset_y() + (14 * 0x200), 
                    CaretType::Shoot, Direction::Left);

                if self.level == WeaponLevel::Level3 {
                    bullet.x = player.gun_offset_x() + (7 * 0x200);
                    bullet.y = player.gun_offset_y() + (3 * 0x200);
                    bullet.counter2 = 1;
                    bullet_manager.push_bullet(bullet.clone());

                    bullet.x = player.gun_offset_x() + (13 * 0x200);
                    bullet.counter2 = 2;
                    bullet_manager.push_bullet(bullet);
                }
            }
            Direction::Left => {
                let yoffset = (self.level == WeaponLevel::Level3) as i32 * 0x200;
                let mut bullet =
                    Bullet::new(
                        player.gun_offset_x() + (9 * 0x200), 
                        player.gun_offset_y() + yoffset + (8 * 0x200), 
                        btype, player_id, Direction::Left, &state.constants);

                bullet_manager.push_bullet(bullet.clone());

                state.create_caret(
                    player.gun_offset_x() + (5 * 0x200), 
                    player.gun_offset_y() + yoffset + (8 * 0x200), 
                    CaretType::Shoot, Direction::Left);

                if self.level == WeaponLevel::Level3 {
                    bullet.x = player.gun_offset_x() + (19 * 0x200);
                    bullet.y = player.gun_offset_y() + (7 * 0x200);
                    bullet.counter2 = 1;
                    bullet_manager.push_bullet(bullet.clone());

                    bullet.x = player.gun_offset_x() + (15 * 0x200);
                    bullet.y = player.gun_offset_y() + (0 * 0x200);
                    bullet.counter2 = 2;
                    bullet_manager.push_bullet(bullet);
                }
            }
            Direction::Right => {
                let yoffset = (self.level == WeaponLevel::Level3) as i32 * 0x200;
                let mut bullet =
                    Bullet::new(
                        player.gun_offset_x() + (14 * 0x200), 
                        player.gun_offset_y() + yoffset + (8 * 0x200), 
                        btype, player_id, Direction::Right, &state.constants);

                bullet_manager.push_bullet(bullet.clone());

                state.create_caret(
                    player.gun_offset_x() + (18 * 0x200), 
                    player.gun_offset_y() + yoffset + (8 * 0x200), 
                    CaretType::Shoot, Direction::Left);

                if self.level == WeaponLevel::Level3 {
                    bullet.x = player.gun_offset_x() + (4 * 0x200);
                    bullet.y = player.gun_offset_y() + (7 * 0x200);
                    bullet.counter2 = 1;
                    bullet_manager.push_bullet(bullet.clone());

                    bullet.x = player.gun_offset_x() + (0 * 0x200);
                    bullet.y = player.gun_offset_y() + (8 * 0x200);
                    bullet.counter2 = 2;
                    bullet_manager.push_bullet(bullet);
                }
            }
            _ => {}
        }

        state.sound_manager.play_sfx(32)
    }
}
