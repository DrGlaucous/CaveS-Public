use crate::common::Direction;
use crate::game::caret::CaretType;
//use crate::game::player::{Player, TargetPlayer};
use crate::game::weapon::{Shooter, TargetShooter};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::BulletManager;

impl Weapon {
    pub(crate) fn tick_electric_therapy(
        &mut self,
        player: &mut dyn Shooter,
        player_id: TargetShooter,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {

        if !player.shoot() {
            self.counter1 = 1;
            self.counter2 += 1;

            //original counts: 0, 2
            if (player.equip().has_turbocharge() && self.counter2 > 2) || self.counter2 > 4 {
                self.counter2 = 0;
                self.refill_ammo(1);
            }
            return;
        }

        // self.counter2 : recharge time counter
        //self.counter1 += 1; // autofire counter

        //if self.counter1 >= 0 {
        //    self.counter1 = 0;
        {

            let btype = match self.level {
                WeaponLevel::Level1 => 46,
                WeaponLevel::Level2 => 47,
                WeaponLevel::Level3 => 48,
                WeaponLevel::None => unreachable!(),
            };

            if !self.consume_ammo(1) {
                self.draw_empty(state, player.x(), player.y());
                return;
            }

            match player.direction() {
                Direction::Left if player.up() => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (13 * 0x200), 
                        player.gun_offset_y() + (4 * 0x200), 
                        btype, player_id, Direction::Up, &state.constants);
                }
                Direction::Right if player.up() => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (10 * 0x200), 
                        player.gun_offset_y() + (8 * 0x200), 
                        btype, player_id, Direction::Up, &state.constants);
                }
                Direction::Left if player.down() => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (14 * 0x200), 
                        player.gun_offset_y() + (9 * 0x200), 
                        btype, player_id, Direction::Bottom, &state.constants);
                }
                Direction::Right if player.down() => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (9 * 0x200), 
                        player.gun_offset_y() + (9 * 0x200), 
                        btype, player_id, Direction::Bottom, &state.constants);
                }
                Direction::Left => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (9 * 0x200), 
                        player.gun_offset_y() + (11 * 0x200), 
                        btype, player_id, Direction::Left, &state.constants);
                }
                Direction::Right => {
                    bullet_manager.create_bullet(
                        player.gun_offset_x() + (14 * 0x200), 
                        player.gun_offset_y() + (11 * 0x200), 
                        btype, player_id, Direction::Right, &state.constants);
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
}
