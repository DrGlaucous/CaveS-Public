use num_derive::FromPrimitive;

use crate::common::{Condition, Direction, Equipment};
use crate::engine_constants::EngineConstants;
use crate::game::caret::CaretType;
//use crate::game::player::{Player, TargetPlayer};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::bullet::BulletManager;

mod blade;
mod bubbler;
pub mod bullet;
mod fireball;
mod machine_gun;
mod missile_launcher;
mod nemesis;
mod polar_star;
mod snake;
mod spur;
mod super_missile_launcher;

#[derive(Debug, PartialEq, Eq, Copy, Clone, FromPrimitive)]
#[repr(u8)]
pub enum WeaponType {
    None = 0,
    Snake = 1,
    PolarStar = 2,
    Fireball = 3,
    MachineGun = 4,
    MissileLauncher = 5,
    Bubbler = 7,
    Blade = 9,
    SuperMissileLauncher = 10,
    Nemesis = 12,
    Spur = 13,
}
impl WeaponType {
    pub fn from_u8(id: u8) -> WeaponType {
        match id {
            1 => Self::Snake,
            2 => Self::PolarStar,
            3 => Self::Fireball,
            4 => Self::MachineGun,
            5 => Self::MissileLauncher,
            7 => Self::Bubbler,
            9 => Self::Blade,
            10 => Self::SuperMissileLauncher,
            12 => Self::Nemesis,
            13 => Self::Spur,
            _ => Self::None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum WeaponLevel {
    None = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
}

impl WeaponLevel {
    pub fn from_u8(id: u8) -> WeaponLevel {
        match id {
            1 => Self::Level1,
            2 => Self::Level2,
            3 => Self::Level3,
            _ => Self::None,
        }
    }

    pub fn next(self) -> WeaponLevel {
        match self {
            WeaponLevel::None => WeaponLevel::Level1,
            WeaponLevel::Level1 => WeaponLevel::Level2,
            WeaponLevel::Level2 => WeaponLevel::Level3,
            WeaponLevel::Level3 => WeaponLevel::Level3,
        }
    }

    pub fn prev(self) -> WeaponLevel {
        match self {
            WeaponLevel::None => WeaponLevel::Level1,
            WeaponLevel::Level1 => WeaponLevel::Level1,
            WeaponLevel::Level2 => WeaponLevel::Level1,
            WeaponLevel::Level3 => WeaponLevel::Level2,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Weapon {
    pub wtype: WeaponType,
    pub level: WeaponLevel,
    pub experience: u16,
    pub ammo: u16,
    pub max_ammo: u16,
    refire_timer: u16,
    empty_counter: u16,
    counter1: u16,
    counter2: u16,
}

impl Weapon {
    pub fn new(wtype: WeaponType, level: WeaponLevel, experience: u16, ammo: u16, max_ammo: u16) -> Weapon {
        Weapon { wtype, level, experience, ammo, max_ammo, refire_timer: 0, empty_counter: 0, counter1: 0, counter2: 0 }
    }

    /// Consume a specified amount of bullets, returns true if there was enough ammo.
    pub fn consume_ammo(&mut self, amount: u16) -> bool {
        if self.max_ammo == 0 {
            return true;
        }

        if self.ammo >= amount {
            self.ammo -= amount;
            return true;
        }

        false
    }

    /// Draw empty! caret only once every 50 ticks
    pub fn draw_empty(&mut self, state: &mut SharedGameState, x: i32, y: i32) {
        state.sound_manager.play_sfx(37);

        if self.empty_counter == 0 {
            state.create_caret(x, y, CaretType::EmptyText, Direction::Left);
            self.empty_counter = 50;
        }
    }

    /// Set refire timer
    pub fn set_refire_timer(&mut self) {
        self.refire_timer = 4;
    }

    /// Refill a specified amount of bullets.
    pub fn refill_ammo(&mut self, amount: u16) {
        if self.max_ammo != 0 {
            self.ammo = self.ammo.saturating_add(amount).min(self.max_ammo);
        }
    }

    pub fn get_max_exp(&self, constants: &EngineConstants) -> (u16, u16, bool) {
        if self.level == WeaponLevel::None {
            return (0, 0, false);
        }

        let level_idx = self.level as usize - 1;
        let max_exp = constants.weapon.level_table[self.wtype as usize][level_idx];
        let max = self.level == WeaponLevel::Level3 && self.experience == max_exp;

        (self.experience, max_exp, max)
    }

    pub fn add_xp(&mut self, exp: u16, player: &mut dyn Shooter, state: &mut SharedGameState) {
        let curr_level_idx = self.level as usize - 1;
        let lvl_table = state.constants.weapon.level_table[self.wtype as usize];

        self.experience = self.experience.saturating_add(exp);

        if self.level == WeaponLevel::Level3 {
            if self.experience >= lvl_table[2] {
                self.experience = lvl_table[2];

                if player.equip().has_whimsical_star() && player.stars() < 3 {
                    player.set_stars(player.stars() + 1);
                }
            }
        } else if self.experience >= lvl_table[curr_level_idx] {
            self.level = self.level.next();
            self.experience = 0;

            if self.wtype != WeaponType::Spur {
                state.sound_manager.play_sfx(27);
                state.create_caret(player.x(), player.y(), CaretType::LevelUp, Direction::Left);
            }
        }

        player.set_xp_counter(if self.wtype != WeaponType::Spur { 30 } else { 10 });
    }

    pub fn reset_xp(&mut self) {
        self.level = WeaponLevel::Level1;
        self.experience = 0;
    }

    pub fn tick(
        &mut self,
        state: &mut SharedGameState,
        player: &mut dyn Shooter,
        player_id: TargetShooter,
        bullet_manager: &mut BulletManager,
    ) {
        if !player.cond().alive() || player.cond().hidden() {
            return;
        }

        self.empty_counter = self.empty_counter.saturating_sub(1);
        self.refire_timer = self.refire_timer.saturating_sub(1);

        //wait 4 ticks before allowing another shot
        if player.trigger_shoot() {
            if self.refire_timer > 0 {
                return;
            }
            self.refire_timer = 4;
        }

        // todo lua hook

        match self.wtype {
            WeaponType::None => {}
            WeaponType::Snake => self.tick_snake(player, player_id, bullet_manager, state),
            WeaponType::PolarStar => self.tick_polar_star(player, player_id, bullet_manager, state),
            WeaponType::Fireball => self.tick_fireball(player, player_id, bullet_manager, state),
            WeaponType::MachineGun => self.tick_machine_gun(player, player_id, bullet_manager, state),
            WeaponType::MissileLauncher => self.tick_missile_launcher(player, player_id, bullet_manager, state),
            WeaponType::Bubbler => self.tick_bubbler(player, player_id, bullet_manager, state),
            WeaponType::Blade => self.tick_blade(player, player_id, bullet_manager, state),
            WeaponType::SuperMissileLauncher => {
                self.tick_super_missile_launcher(player, player_id, bullet_manager, state)
            }
            WeaponType::Nemesis => self.tick_nemesis(player, player_id, bullet_manager, state),
            WeaponType::Spur => self.tick_spur(player, player_id, bullet_manager, state),
        }
    }
}

//player or NPC that can shoot
pub trait Shooter {

    //true if the shooter is shooting
    fn shoot(&self) -> bool;

    //true if this is the first shooting frame
    fn trigger_shoot(&self) -> bool;

    fn cond(&self) -> Condition;

    //shooter's locations
    fn x(&self) -> i32;

    fn y(&self) -> i32;

    fn vel_x(&self) -> i32;

    fn vel_y(&self) -> i32;

    fn set_vel_x(&mut self, num: i32);

    fn set_vel_y(&mut self, num: i32);


    //shooter's equip flags
    fn equip(&self) -> Equipment;

    //shooter's direction
    fn direction(&self) -> Direction;

    fn up(&self) -> bool;

    fn down(&self) -> bool;

    fn gun_offset_x(&self) -> i32;

    fn gun_offset_y(&self) -> i32;

    fn stars(&self) -> u8;

    fn set_stars(&mut self, num: u8);

    fn set_xp_counter(&mut self, num: u8);

}


#[derive(PartialEq, Eq, Copy, Clone)]
pub enum TargetShooter {
    Player1,
    Player2,
    NPC(u32),
}

