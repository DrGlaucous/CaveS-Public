

pub struct MouseContext {
    pub(crate) abs_mouse_coords: (i32, i32),
    pub(crate) rel_mouse_coords: (f32, f32),
    pub(crate) mouse_is_locked: bool,
    pub(crate) mouse_veloc: (f32, f32),
    pub(crate) mouse_accel_sensitivity: f32,

    //private stuff
    //last_mouse_coords: (i32, i32),
    last_lock_state: bool
}

impl MouseContext {

    pub(crate) fn new() -> Self {
        Self {
            abs_mouse_coords: (0,0),
            rel_mouse_coords: (0.0,0.0),
            mouse_is_locked: false,
            mouse_veloc: (0.0,0.0),
            last_lock_state: false,
            mouse_accel_sensitivity: 0.5,
        }
    }


    pub(crate) fn set_lock_state(&mut self, locked: bool)
    {
        self.mouse_is_locked = locked;
    }
    pub(crate) fn get_lock_state(&mut self) -> bool
    {
        self.mouse_is_locked
    }
    pub(crate) fn lock_state_changed(&mut self) -> bool
    {
        let result = !(self.last_lock_state == self.mouse_is_locked);
        self.last_lock_state = self.mouse_is_locked;
        return result;
    }

    pub(crate) fn update_mouse_coords(&mut self, x: i32, y: i32, scale: f32)
    {
        //self.mouse_veloc = (self.mouse_accel_sensitivity * (x - self.abs_mouse_coords.0) as f32,
        //                    self.mouse_accel_sensitivity * (y - self.abs_mouse_coords.1) as f32);

        
        self.abs_mouse_coords = (x,y);

        //relative to pixels, but since we are using velocity, not location, we want abs coords * sensitivity
        //self.rel_mouse_coords = (self.abs_mouse_coords.0 as f32 / scale, self.abs_mouse_coords.1 as f32 / scale);
        self.rel_mouse_coords = (self.abs_mouse_coords.0 as f32 * self.mouse_accel_sensitivity, self.abs_mouse_coords.1 as f32 * self.mouse_accel_sensitivity);

    }





}









