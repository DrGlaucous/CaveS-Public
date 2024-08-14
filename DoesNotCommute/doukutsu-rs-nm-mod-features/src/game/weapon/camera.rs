use crate::common::Direction;
use crate::common::Rect;
use crate::game::weapon::{Shooter, TargetShooter};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::{Bullet, BulletManager};
use crate::game::npc::list::NPCList;
use crate::game::caret::CaretType;

impl Weapon {
    pub(crate) fn tick_camera(
        &mut self,
        player: &mut dyn Shooter,
        player_id: TargetShooter,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {

        //recharge when button is not held
        if !player.shoot() {
            self.counter2 += 1;
            if (player.equip().has_turbocharge() && self.counter2 > 25) || self.counter2 > 50 {
                self.counter2 = 0;
                self.refill_ammo(1);
            }
            return;
        }


        if player.trigger_shoot() {

            if !self.consume_ammo(1) {
                self.draw_empty(state, player.x(), player.y());
                return;
            }

            
            let btype = match self.level {
                WeaponLevel::Level1 => 55,
                WeaponLevel::Level2 => 56,
                WeaponLevel::Level3 => 57,
                WeaponLevel::None => unreachable!(),
            };
            
            //all targeting logic has been moved to the bullet because we need to access NPCs and PCs alike
            let bullet = Bullet::new(player.x(), player.y(),
                btype, player_id, Direction::Left, &state.constants
            );

            bullet_manager.push_bullet(bullet);

            let (x,y) = match (player.up(), player.down(), player.direction()) {
                (false, false, Direction::Left) => (11,11),
                (true, false, Direction::Left) => (15,8),
                (false, true, Direction::Left) => (14,9),

                (false, false, Direction::Right) => (16,11),
                (true, false, Direction::Right) => (10,7),
                (false, true, Direction::Right) => (11,10),
                _ =>(0,0)
            };

            
            //camera noise + caret
            state.sound_manager.play_sfx(37);
            state.create_caret(player.gun_offset_x() + x * 0x200, player.gun_offset_y() + y * 0x200, CaretType::Shoot, Direction::Left);


        }
    }




}
