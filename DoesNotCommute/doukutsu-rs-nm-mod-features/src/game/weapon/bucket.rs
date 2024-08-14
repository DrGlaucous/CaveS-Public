use crate::common::Direction;
use crate::game::weapon::{Shooter, TargetShooter};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::{Bullet, BulletManager};

impl Weapon {
    pub(crate) fn tick_bucket(
        &mut self,
        player: &mut dyn Shooter,
        player_id: TargetShooter,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
        is_pills: bool,
    ) {

        const BULLETS: [u16; 3] = [52, 53, 54];

        if bullet_manager.count_bullets_multi(&BULLETS, player_id) >= 3 * self.level as usize {
            return;
        }

        if player.trigger_shoot() {

            let (btype, repeat) = match self.level {
                WeaponLevel::Level1 => (52, 1),
                WeaponLevel::Level2 => (53, 2),
                WeaponLevel::Level3 => (54, 3),
                WeaponLevel::None => unreachable!(),
            };

            for _ in 0..repeat {

                let mut bullet = match player.direction() {
                    Direction::Left if player.up() => {
                        Bullet::new(
                            player.gun_offset_x() + (8 * 0x200), 
                            player.gun_offset_y() + (8 * 0x200),
                            btype, player_id, Direction::Up, &state.constants,
                        )
                    },
                    Direction::Right if player.up() => {
                        Bullet::new(
                            player.gun_offset_x() + (16 * 0x200), 
                            player.gun_offset_y() + (8 * 0x200),
                            btype, player_id, Direction::Up, &state.constants,
                        )
                    },
                    Direction::Left if player.down() => {
                        Bullet::new(
                            player.gun_offset_x() + (8 * 0x200), 
                            player.gun_offset_y() + (8 * 0x200),
                            btype, player_id, Direction::Bottom, &state.constants,
                        )
                    },
                    Direction::Right if player.down() => {
                        Bullet::new(
                            player.gun_offset_x() + (16 * 0x200), 
                            player.gun_offset_y() + (8 * 0x200),
                            btype, player_id, Direction::Bottom, &state.constants,
                        )
                    },
                    Direction::Left => {
                        Bullet::new(
                            player.gun_offset_x() + (8 * 0x200),
                            player.gun_offset_y() + (8 * 0x200),
                            btype, player_id, Direction::Left, &state.constants,
                        )
                    },
                    Direction::Right => {
                        Bullet::new(
                            player.gun_offset_x() + (16 * 0x200), 
                            player.gun_offset_y() + (8 * 0x200),
                            btype, player_id, Direction::Right, &state.constants,
                        )
                    },
                    _ => { unreachable!() }
                };

                bullet.counter1 = if is_pills {1} else {0};
                bullet_manager.push_bullet(bullet);
        
            }

            state.sound_manager.play_sfx(34);

        }
    }




}
