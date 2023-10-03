use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::game::shared_game_state::SharedGameState;

pub trait PlayerController: PlayerControllerClone {
    fn update(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult;

    fn update_trigger(&mut self);

    /// True if "move up" button is down.
    fn move_up(&self) -> bool;

    /// True if "move left" button is down.
    fn move_left(&self) -> bool;

    /// True if "move down" button is down.
    fn move_down(&self) -> bool;

    /// True if "move right" button is down.
    fn move_right(&self) -> bool;

    /// True if "prev weapon" button is down.
    fn prev_weapon(&self) -> bool;

    /// True if "next weapon" button is down.
    fn next_weapon(&self) -> bool;

    /// True if "map" button is down.
    fn map(&self) -> bool;

    /// True if "inventory" button is down.
    fn inventory(&self) -> bool;

    /// True if "jump" button is down.
    fn jump(&self) -> bool;

    /// True if "shoot" button is down.
    fn shoot(&self) -> bool;

    /// True if "skip" button is down.
    fn skip(&self) -> bool;

    /// True if "strafe" button is down.
    fn strafe(&self) -> bool;

    fn trigger_up(&self) -> bool;

    fn trigger_left(&self) -> bool;

    fn trigger_down(&self) -> bool;

    fn trigger_right(&self) -> bool;

    fn trigger_prev_weapon(&self) -> bool;

    fn trigger_next_weapon(&self) -> bool;

    fn trigger_map(&self) -> bool;

    fn trigger_inventory(&self) -> bool;

    fn trigger_jump(&self) -> bool;

    fn trigger_shoot(&self) -> bool;

    fn trigger_skip(&self) -> bool;

    fn trigger_strafe(&self) -> bool;

    fn trigger_menu_ok(&self) -> bool;

    fn trigger_menu_back(&self) -> bool;

    fn trigger_menu_pause(&self) -> bool;

    /// Optional, useful for controllers with two analog sticks.
    /// Returns true if player looks towards upper direction.
    fn look_up(&self) -> bool;

    /// Optional, useful for controllers with two analog sticks.
    /// Returns true if player looks towards left direction.
    fn look_left(&self) -> bool;

    /// Optional, useful for controllers with two analog sticks.
    /// Returns true if player looks towards bottom direction.
    fn look_down(&self) -> bool;

    /// Optional, useful for controllers with two analog sticks.
    /// Returns true if player looks towards right direction.
    fn look_right(&self) -> bool;

    /// Returns movement analog stick state in X axis within (-1.0..=1.0) range
    /// In case of non-analog controllers this should return -1.0, 0.0 or 1.0, depending on keys pressed.
    fn move_analog_x(&self) -> f64;

    /// Returns movement analog stick state in Y axis within (-1.0..=1.0) range
    /// In case of non-analog controllers this should return -1.0, 0.0 or 1.0, depending on keys pressed.
    fn move_analog_y(&self) -> f64;

    /// Activates the rumble motors to the specified intensities for a given amount of time (in millis).
    fn set_rumble(&mut self, low_freq: u16, hi_freq: u16, ticks: u32);
}

pub trait PlayerControllerClone {
    fn clone_box(&self) -> Box<dyn PlayerController>;
}

impl<T: 'static + PlayerController + Clone> PlayerControllerClone for T {
    fn clone_box(&self) -> Box<dyn PlayerController> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn PlayerController> {
    fn clone(&self) -> Box<dyn PlayerController> {
        self.clone_box()
    }
}
