use crate::common::Direction;
use crate::game::weapon::{Shooter, TargetShooter};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::BulletManager;

impl Weapon {
    pub(crate) fn tick_melee(
        &mut self,
        player: &mut dyn Shooter,
        player_id: TargetShooter,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        const BULLETS: [u16; 3] = [49, 50, 51];


        if bullet_manager.count_bullets_multi(&BULLETS, player_id) > 0 {
            return;
        }

        if player.trigger_shoot() {

            let btype = match self.level {
                WeaponLevel::Level1 => 49,
                WeaponLevel::Level2 => 50,
                WeaponLevel::Level3 => 51,
                WeaponLevel::None => unreachable!(),
            };

            match player.direction() {
                Direction::Left if player.up() => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (8 * 0x200), 
                        player.gun_offset_y() + (8 * 0x200), 
                        btype, player_id, Direction::Up, &state.constants);
                }
                Direction::Right if player.up() => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (8 * 0x200), 
                        player.gun_offset_y() + (8 * 0x200), 
                        btype, player_id, Direction::Up, &state.constants);
                }
                Direction::Left if player.down() => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (8 * 0x200), 
                        player.gun_offset_y() + (8 * 0x200), 
                        btype, player_id, Direction::Bottom, &state.constants);
                }
                Direction::Right if player.down() => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (8 * 0x200), 
                        player.gun_offset_y() + (8 * 0x200), 
                        btype, player_id, Direction::Bottom, &state.constants);
                }
                Direction::Left => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (8 * 0x200), 
                        player.gun_offset_y() + (8 * 0x200), 
                        btype, player_id, Direction::Left, &state.constants);
                }
                Direction::Right => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (8 * 0x200), 
                        player.gun_offset_y() + (8 * 0x200), 
                        btype, player_id, Direction::Right, &state.constants);
                }
                _ => {}
            }
    
            state.sound_manager.play_sfx(34);

        }
    }
}
