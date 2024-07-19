use crate::common::Direction;
use crate::game::caret::CaretType;
//use crate::game::player::{Player, TargetPlayer};
use crate::game::weapon::{Shooter, TargetShooter};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::BulletManager;

impl Weapon {
    pub(crate) fn tick_nemesis(
        &mut self,
        player: &dyn Shooter,
        player_id: TargetShooter,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        const BULLETS: [u16; 3] = [34, 35, 36];

        if !player.trigger_shoot() || bullet_manager.count_bullets_multi(&BULLETS, player_id) > 1 {
            return;
        }

        let btype = match self.level {
            WeaponLevel::Level1 => 34,
            WeaponLevel::Level2 => 35,
            WeaponLevel::Level3 => 36,
            WeaponLevel::None => unreachable!(),
        };

        if !self.consume_ammo(1) {
            state.sound_manager.play_sfx(37);
            // The vanilla game doesn't spawn "empty" text for some reason
            return;
        }

        if player.up() {
            match player.direction() {
                Direction::Left => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (12 * 0x200), 
                        player.gun_offset_y() + (-9 * 0x200), 
                        btype, player_id, Direction::Up, &state.constants);
                    state.create_caret(
                        player.gun_offset_x() + (12 * 0x200), 
                        player.gun_offset_y() + (-2 * 0x200), 
                        CaretType::Shoot, Direction::Left);
                }
                Direction::Right => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (11 * 0x200), 
                        player.gun_offset_y() + (-9 * 0x200), 
                        btype, player_id, Direction::Up, &state.constants);
                    state.create_caret(
                        player.gun_offset_x() + (11 * 0x200), 
                        player.gun_offset_y() + (-2 * 0x200), 
                        CaretType::Shoot, Direction::Left);
                }
                _ => {}
            }
        } else if player.down() {
            match player.direction() {
                Direction::Left => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (14 * 0x200), 
                        player.gun_offset_y() + (24 * 0x200), 
                        btype, player_id, Direction::Bottom, &state.constants);
                    state.create_caret(
                        player.gun_offset_x() + (14 * 0x200), 
                        player.gun_offset_y() + (17 * 0x200), 
                        CaretType::Shoot, Direction::Left);
                }
                Direction::Right => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (9 * 0x200), 
                        player.gun_offset_y() + (24 * 0x200), 
                        btype, player_id, Direction::Bottom, &state.constants);
                    state.create_caret(
                        player.gun_offset_x() + (9 * 0x200), 
                        player.gun_offset_y() + (17 * 0x200), 
                        CaretType::Shoot, Direction::Left);
                }
                _ => {}
            }
        } else {
            match player.direction() {
                Direction::Left => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (-7 * 0x200), 
                        player.gun_offset_y() + (11 * 0x200), 
                        btype, player_id, Direction::Left, &state.constants);
                    state.create_caret(
                        player.gun_offset_x() + (0 * 0x200), 
                        player.gun_offset_y() + (11 * 0x200), 
                        CaretType::Shoot, Direction::Left);
                }
                Direction::Right => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (30 * 0x200), //22
                        player.gun_offset_y() + (11 * 0x200), 
                        btype, player_id, Direction::Right, &state.constants);
                    state.create_caret(
                        player.gun_offset_x() + (23 * 0x200), 
                        player.gun_offset_y() + (11 * 0x200), 
                        CaretType::Shoot, Direction::Right);
                }
                _ => {}
            }
        }

        match self.level {
            WeaponLevel::Level1 => state.sound_manager.play_sfx(117),
            WeaponLevel::Level2 => state.sound_manager.play_sfx(49),
            WeaponLevel::Level3 => state.sound_manager.play_sfx(60),
            _ => unreachable!(),
        }
    }
}
