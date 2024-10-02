use crate::common::Direction;
use crate::game::caret::CaretType;
use crate::game::player::{Player, TargetPlayer};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::BulletManager;

impl Weapon {
    pub(crate) fn tick_spur(
        &mut self,
        player: &mut Player,
        player_id: TargetPlayer,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        const BULLETS: [u16; 3] = [10, 11, 12];

        //charging (only if we've finished firing)
        if player.controller.shoot() && self.counter2 == 0 {
            self.add_xp(if player.equip.has_turbocharge() { 3 } else { 2 }, player, state);

            if self.level == WeaponLevel::Level1 && self.counter1 == 0 {
                state.sound_manager.play_sfx(103); //play power up sound 1x
                self.counter1 = 1;
            }

            //play full sound 1x if we're at max EXP
            if let (_, _, true) = self.get_max_exp(&state.constants) {

                if self.counter1 == 1 {
                    state.sound_manager.play_sfx(65);
                    self.counter1 = 2;
                }
            }
        } else if self.counter1 > 0 {
            state.sound_manager.stop_sfx(103); //halt charge sound if not already

            //player released the shoot button, fire the charge only if it's been full
            if let (_, _, true) = self.get_max_exp(&state.constants) {
                self.counter2 = 120; //20 bullets, 6 tick spacing
            }
            self.counter1 = 0;
        } else {
            self.reset_xp();
        }


        //this is the only type that will be shot 
        let btype = BULLETS[2];

        //tick down the firing counter
        if self.counter2 > 0 {
            self.counter2 -= 1;
        }

        //every 6 ticks
        if self.counter2 % 6 == 1 {
            if !self.consume_ammo(1) {
                state.sound_manager.play_sfx(37);
            } else {


                match () {
                    _ if player.up => {
                        player.vel_y += 0x100;
    
                        match player.direction {
                            Direction::Left => {
                                bullet_manager.create_bullet(
                                    player.x - 0x600,
                                    player.y - 0x1000,
                                    btype,
                                    player_id,
                                    Direction::Up,
                                    &state.constants,
                                );
                                state.create_caret(player.x - 0x600, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                            }
                            Direction::Right => {
                                bullet_manager.create_bullet(
                                    player.x + 0x600,
                                    player.y - 0x1000,
                                    btype,
                                    player_id,
                                    Direction::Up,
                                    &state.constants,
                                );
                                state.create_caret(player.x + 0x600, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                            }
                            _ => {}
                        }
                    }
                    _ if player.down => {
                        if player.vel_y > 0 {
                            player.vel_y /= 2;
                        }
                        if player.vel_y > -0x400 {
                            player.vel_y = (player.vel_y - 0x200).max(-0x400);
                        }
    
                        match player.direction {
                            Direction::Left => {
                                bullet_manager.create_bullet(
                                    player.x - 0x600,
                                    player.y + 0x1000,
                                    btype,
                                    player_id,
                                    Direction::Bottom,
                                    &state.constants,
                                );
                                state.create_caret(player.x - 0x600, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                            }
                            Direction::Right => {
                                bullet_manager.create_bullet(
                                    player.x + 0x600,
                                    player.y + 0x1000,
                                    btype,
                                    player_id,
                                    Direction::Bottom,
                                    &state.constants,
                                );
                                state.create_caret(player.x + 0x600, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                            }
                            _ => {}
                        }
                    }
                    _ => match player.direction {
                        Direction::Left => {
                            bullet_manager.create_bullet(
                                player.x - 0x1800,
                                player.y + 0x600,
                                btype,
                                player_id,
                                Direction::Left,
                                &state.constants,
                            );
                            state.create_caret(player.x - 0x1800, player.y + 0x600, CaretType::Shoot, Direction::Left);
                        }
                        Direction::Right => {
                            bullet_manager.create_bullet(
                                player.x + 0x1800,
                                player.y + 0x600,
                                btype,
                                player_id,
                                Direction::Right,
                                &state.constants,
                            );
                            state.create_caret(player.x + 0x1800, player.y + 0x600, CaretType::Shoot, Direction::Right);
                        }
                        _ => {}
                    },
                }
    
                state.sound_manager.play_sfx(49);
    
            }
        }

    }
}
