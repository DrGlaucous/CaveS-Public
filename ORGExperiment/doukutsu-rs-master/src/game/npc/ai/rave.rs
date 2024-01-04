use std::f32::consts::PI;
use std::process::Child;
use core::ops::Range;

use std::time::SystemTime;

use crate::common::{Direction, value_map};
use crate::framework::error::GameResult;
use crate::game::npc::boss::BossNPC;
use crate::game::npc::list::NPCList;
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::game::stage::Stage;
use crate::game::npc::NPCLayer;

use crate::scene::game_scene::GameScene;
use crate::util::rng::RNG;


#[inline(always)]
fn float_is_equal(a: f32, b: f32) -> bool
{
    (a - b).abs() < 0.001
}


fn deg_to_rad(deg: i32) -> f32
{
    deg as f32 * PI / 180.0
}
fn rad_to_deg(rad: f32) -> i32
{
    (rad * 180.0 / PI) as i32
}

//puts the light cone on the direction the NPC is facing
fn center_light_cone(npc_angle: f32, cone_width: &Range<i32>) -> Range<i32>
{
    //take light width and center it on GOBO direction
    let l_radius = (cone_width.end % 360 - cone_width.start % 360).abs() / 2;

    //we also add 180 to make the light and the npc angle agree with each other
    let avar = (rad_to_deg(npc_angle) + 180   -l_radius)..(rad_to_deg(npc_angle) + 180 +l_radius);
    avar
}

//returns 1 if positive direction, 0 if negative direction
fn determine_shortest_direct(angle_start: &mut f32, angle_end: &mut f32) -> i32
{
    //for testing
    //let mut angle_start = PI;
    //let mut angle_end = -PI / 2.0;

    //let old_val_a = *angle_start;
    //let old_val_b = *angle_end;


    //set coordinates relative to start point
    let mut new_end = *angle_end - *angle_start;

    //normalize end angle
    if new_end < 0.0 {
        new_end += ((new_end / (2.0 * PI)).abs().floor() + 1.0) * 2.0 * PI;
    }
    else {
        new_end -= (new_end / (2.0 * PI)).floor() * 2.0 * PI;
    }

    let way_pos = new_end;
    let way_neg = 2.0 * PI - new_end;

    //normalize entered start angle, used to crop down start and end angles to one positive rotation
    if *angle_start < 0.0 {
        *angle_start += ((*angle_start / (2.0 * PI)).abs().floor() + 1.0) * 2.0 * PI;
    }
    else {
        *angle_start -= (*angle_start / (2.0 * PI)).floor() * 2.0 * PI;
    }

    *angle_end = *angle_start + if way_pos < way_neg {way_pos} else {-way_neg};

    return if way_pos < way_neg {1} else {-1};



}



//rust raycast unit circle:
//        90
//        |
//        |
//<--0---[ ]---180-->
//        |
//        |
//       270

//rust sprite rotation unit circle:
//       3PI/2
//         |
//         |
//<--PI---[ ]---0-->
//         |
//         |
//        PI/2





impl NPC {


    //the active part of the GOBO
    pub(crate) fn tick_n371_gobo(&mut self, state: &mut SharedGameState, npc_list: &NPCList,) -> GameResult {

        //establish gobo behavior
        match self.action_num
        {
            //initialize
            0 =>
            {

                //TEST
                // self.gen_var_a = 1; //light mode
                // self.gen_var_b = 40; //effect rate

                // self.action_counter2 = 0;
                // self.action_counter3 = 3142 / 2; //pi / 2
                // self.anim_counter = 10;//SINE movement
                // self.anim_num = 30;


                // spawn base
                let mut npc = NPC::create(372, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x;
                npc.y = self.y;
                npc.direction = self.direction;

                //spawned with alt direction, hanging
                if self.direction != Direction::from_int(0).unwrap()
                {
                    //swap viewbox dimensions
                    let old_bottom = npc.display_bounds.bottom;
                    npc.display_bounds.bottom = npc.display_bounds.top;
                    npc.display_bounds.top = old_bottom;

                    self.y += 8 * 0x200; //I feel like there's a function somewhere to standardize the 0x200 bit.
                
                    self.angle = PI / 2.0; //90 deg

                    //TEST
                    self.action_counter2 = 3142 / 4;
                    self.action_counter3 = 3 * 3142 / 4; //pi / 2


                }
                else //standing
                {
                    self.y -= 8 * 0x200;

                    self.angle = 3.0 * PI / 2.0; //270 deg


                    //TEST
                    self.action_counter2 = 5 * 3142 / 4;
                    self.action_counter3 = 7 * 3142 / 4; //pi / 2

                }

                let _ = npc_list.spawn(0x100, npc.clone());


                //flag gobo as emissive
                self.npc_flags.set_emits_cone_light(true);

                //set center of rotation
                self.anchor_x = (self.display_bounds.left / 0x200) as f32;
                self.anchor_y = (self.display_bounds.top / 0x200) as f32;

                self.action_num = 1; //goto nothing


            }

            //color modes

            //set color R
            10 => {
                //clamp to char size
                if self.pass_var_a > 0xFF {self.pass_var_a = 0xFF}
                //normalize and set red
                self.light_color.r = self.pass_var_a as f32/ 255.0;

                self.action_num = 1; //idle
            }
            //set color G
            11 => {
                //clamp to char size
                if self.pass_var_a > 0xFF {self.pass_var_a = 0xFF}
                //normalize and set red
                self.light_color.g = self.pass_var_a as f32/ 255.0;

                self.action_num = 1; //idle
            }
            //set color B
            12 => {
                //clamp to char size
                if self.pass_var_a > 0xFF {self.pass_var_a = 0xFF}
                //normalize and set red
                self.light_color.b = self.pass_var_a as f32/ 255.0;

                self.action_num = 1; //idle
            }
            //set color mode and speed
            13 => {

                //remember mode (1: fade, 2: strobe, anything else: solid)
                self.gen_var_a = self.pass_var_a as i32;
                //remember speed
                self.gen_var_b = self.pass_var_b as i32;

                self.action_num = 1; //idle
            }

            //Set light cone power
            14 => {
                //divide by 1000 (1003 = 1.003)
                self.light_power = self.pass_var_a as f32 / 1000.0;
                self.action_num = 1; //idle
            }


            //Set light cone width
            15 => {
                //clamp to 360
                if self.pass_var_a > 359 {self.pass_var_a = 359}

                self.light_angle = 0..self.pass_var_a as i32;

                self.action_num = 1; //goto nothing
            }
            

            //movement modes

            //Set animation type and speed
            20 => {

                //remember animation mode (10: sine, 20: triangle, 30: moveTo)
                self.anim_counter = self.pass_var_a;





                //re-using anim_num to store animation speed
                self.anim_num = self.pass_var_b;
                self.action_num = 1; //idle
            }
            //set animation range (the two angles the gobo will rock between when in animation state) (in radians / 1000)
            //in moveTo mode, the first argument (min angle) will be the target location
            21 => {

                if self.pass_var_a <  self.pass_var_b {
                    self.action_counter2 = self.pass_var_a;
                    self.action_counter3 = self.pass_var_b;
                }
                else {
                    self.action_counter2 = self.pass_var_b;
                    self.action_counter3 = self.pass_var_a;
                }


                self.action_num = 1; //idle
            }


            //everything below the settings section will only run if this is the case
            100 => {}


            //default (do nothing)
            _ => {

                // //test flipping
                // let now;// = 0;
                // match  std::time::SystemTime::now().duration_since( std::time::SystemTime::UNIX_EPOCH)
                // {
                //     Ok(n) => 
                //     {
                //         now = n.as_millis();
                //     }
                //     Err(_) => panic!("SystemTime before UNIX EPOCH!")
                // }
                // self.angle = core::f32::consts::PI * (((now % 100000) as f32)/ 1000.0); 

                // self.light_angle = center_light_cone(self.angle, &(0..20));

            }
        }

        //don't run anything else if this is not the correct action number
        if self.action_num != 100
        {
            self.light_angle = center_light_cone(self.angle, &(0..20));
            self.anim_rect = state.constants.npc.n371_gobo[0];
            return Ok(());
        }

        //do actions based on modes

        //variable purpose chart:
        //self.action_counter: animation ticker for movement and color
        //self.action_counter2: min animation angle (radians / 1000): 1000 = 1.000
        //self.action_counter3: max animation angle (radians / 1000): 1000 = 1.000
        //self.anim_num: animation speed * 1000: 1000 = 1.000
        //self.anim_counter: animation mode

        //self.gen_var_a: color mode
        //self.gen_var_b: color animation speed

        //tick the animator
        self.action_counter = self.action_counter.wrapping_add(1);

        //use ID to start the gobo in a "unique" spot (will never overload because action_counter will never exceed u16)
        let unique_action_counter = self.action_counter as u32 + (self.id * 100) as u32;

        //color mode
        match self.gen_var_a
        {
            //gradient mode
            1 => {
                let angle = unique_action_counter as f32 / self.gen_var_b as f32;

                self.light_color.r = angle.sin();
                self.light_color.g = (angle + 2.0*PI/3.0).sin();
                self.light_color.b = (angle + 4.0*PI/3.0).sin();

                self.npc_flags.set_emits_cone_light(true)
            }

            //strobe mode
            2 => {
                
                if (self.action_counter as i32 / self.gen_var_b) % 2 == 0
                {
                    self.npc_flags.set_emits_cone_light(false);
                }
                else {self.npc_flags.set_emits_cone_light(true)}
            }

            //turn light off
            3 => {
                self.npc_flags.set_emits_cone_light(false);
            }

            //flat color mode (turn light on)
            _ => {
                self.npc_flags.set_emits_cone_light(true)
            }
        }

        let min_limit = self.action_counter2 as f32 / 1000.0;
        let max_limit = self.action_counter3 as f32 / 1000.0;
        let step_size = self.anim_num as f32 / 1000.0;

        //invalid range checker
        //if they're the same, run goto linear and halt
        if !([30, 31, 40, 41,].contains(&self.anim_counter)) //is NOT one of these cases
        && float_is_equal(max_limit, min_limit) //and is the same value
        {
            self.anim_counter = 40;
            self.action_counter3 = (self.angle * 1000.0).floor() as u16;
        }

        //movement mode
        match self.anim_counter
        {
            //move back and forth sine
            //drive to position if out of bounds
            10 => {

                //if OOB
                if (self.angle) < min_limit //less than min
                {
                    self.angle += step_size; // += self.anim_num as i32; //move back at animation speed
                }
                else if (self.angle) > max_limit //greater than max
                {
                    self.angle -= step_size;// -= self.anim_num as i32; //move back at animation speed
                }
                else {
                    //within range, start motion (will never be hit if limits are the same)
                    self.anim_counter = 11;
                }

            }
            //move in sine motion (forward)
            11 => {
                //map current location between min and max between 0 and pi
                let sin_loc = value_map(self.angle,
                    min_limit,
                    max_limit,
                    0.0 + 0.1,
                    PI - 0.1,
                );

                //increase by that fraction of speed
                self.angle += step_size * sin_loc.sin();

                //close enough to the edge of travel, go the other way
                if (PI - sin_loc).abs() < 0.1
                {
                    self.anim_counter = 12;
                }
            }
            //move in sine motion (backward)
            12 => {
                //map current location between min and max between 0 and pi
                let sin_loc = value_map(self.angle,
                    min_limit,
                    max_limit,
                    0.0 + 0.1,
                    PI - 0.1,
                );

                //increase by that fraction of speed
                self.angle -= step_size * sin_loc.sin();

                //close enough to the edge of travel, go the other way
                if sin_loc.abs() < 0.1
                {
                    self.anim_counter = 11;
                }
            }

            //move back and forth linear

            //drive to position if out of bounds
            20 => {

                //if OOB
                if (self.angle) < min_limit //less than min
                {
                    self.angle += step_size;
                }
                else if (self.angle) > max_limit //greater than max
                {
                    self.angle -= step_size;
                }
                else {
                    //within range, start motion
                    self.anim_counter = 21;
                }

            }
            //linear forward
            21 => {
                //increase by that fraction of speed
                self.angle += step_size;

                //close enough to the edge of travel, go the other way
                if self.angle >= max_limit
                {
                    self.anim_counter = 22;
                }
            }
            //linear backward
            22 => {
                //increase by that fraction of speed
                self.angle -= step_size;

                //close enough to the edge of travel, go the other way
                if self.angle <= min_limit
                {
                    self.anim_counter = 21;
                }
            }


            //drive to position and halt, sine
            //init, mark old location
            30 => {
                //using the max limit as a stand-in for old location
                self.action_counter3 = (self.angle * 1000.0).floor() as u16;

                //we're already at location, don't move there
                if float_is_equal(self.action_counter3 as f32 / 1000.0, min_limit)
                {
                    self.angle = min_limit;
                    self.anim_counter = 1; //goto idle
                }
                else {
                    self.anim_counter = 31; //next case                    
                }

                //self.anim_counter = 31; //next case   

            }
            //drive and halt
            31 => {

                let mut nu_tgt = min_limit; //target angle in radians

                let dir = determine_shortest_direct(&mut self.angle, &mut nu_tgt);

                //map current location between min and max between 0 and pi
                let sin_loc = value_map(self.angle,
                    nu_tgt,
                    max_limit,
                    0.0 + 0.1,
                    PI - 0.1,
                );

                //increase by that fraction of speed
                self.angle += step_size * sin_loc.sin() * dir as f32;

                //close enough to the edge of travel, goto idle
                if (self.angle - nu_tgt).abs() < step_size //sin_loc.abs() < 0.1 ||
                {
                    self.angle = nu_tgt;
                    self.anim_counter = 1; //goto idle
                }
            }

            //drive to position and halt, linear
            40 => {

                //do location check
                if float_is_equal(self.angle, min_limit) 
                {
                    self.angle = min_limit;
                    self.anim_counter = 1; //goto idle
                }
                else {
                    self.anim_counter = 41; //goto move case
                }

                //self.anim_counter = 41; //goto move case
            }
            41 => {
                //determine shortest way to get to location
                //note: angle limits will always be positive
                
                let mut nu_tgt = min_limit; //target angle in radians

                let dir = determine_shortest_direct(&mut self.angle, &mut nu_tgt);

                self.angle += step_size * dir as f32;

                if (self.angle - nu_tgt).abs() < step_size
                {
                    self.angle = nu_tgt;
                    self.anim_counter = 1; //goto idle
                }
            }


            //sit at current angle
            _ => {
            }
        }


        self.light_angle = center_light_cone(self.angle, &(0..20));

        self.anim_rect = state.constants.npc.n371_gobo[0];


        Ok(())
    }

    //the base of the gobo unit
    pub(crate) fn tick_n372_gobo_base(&mut self, state: &mut SharedGameState) -> GameResult {

        //up/down direction
        let anim_offset = match self.direction
        {
            Direction::Left => 0,
            _ => 2,
        };

        //animation
        self.anim_counter += 1;
        if self.anim_counter > 8
        {
            self.anim_counter = 0;
            self.anim_num += 1;
            if self.anim_num > 1
            {
                self.anim_num = 0;
            }
        }


        self.anim_rect = state.constants.npc.n372_gobo_base[(anim_offset + self.anim_num) as usize];
        Ok(())
    }

    //stationary stage lights
    pub(crate) fn tick_n373_stage_light(&mut self, state: &mut SharedGameState) -> GameResult {

        //establish stage light behavior
        match self.action_num
        {
            
            //initialize
            0 => {

                //TEST
                // self.gen_var_a = 1; //light mode
                // self.gen_var_b = 40; //effect rate
                
                //flag light as emissive
                self.npc_flags.set_emits_cone_light(true);

                //set light direction
                self.light_angle = 0..20;


                self.action_num = 1; //goto nothing
            }

            //color modes

            //set color R
            10 => {
                //clamp to char size
                if self.pass_var_a > 0xFF {self.pass_var_a = 0xFF}
                //normalize and set red
                self.light_color.r = self.pass_var_a as f32/ 255.0;

                self.action_num = 1; //idle
            }
            //set color G
            11 => {
                //clamp to char size
                if self.pass_var_a > 0xFF {self.pass_var_a = 0xFF}
                //normalize and set red
                self.light_color.g = self.pass_var_a as f32/ 255.0;

                self.action_num = 1; //idle
            }
            //set color B
            12 => {
                //clamp to char size
                if self.pass_var_a > 0xFF {self.pass_var_a = 0xFF}
                //normalize and set red
                self.light_color.b = self.pass_var_a as f32/ 255.0;

                self.action_num = 1; //idle
            }
            //set color mode
            13 => {

                //remember mode
                self.gen_var_a = self.pass_var_a as i32;

                match self.pass_var_a
                {
                    //color gradient mode and strobe mode take speed parameter
                    1 | 2=>{

                        self.gen_var_b = self.pass_var_b as i32;
                    }

                    //flat color mode
                    _ =>{}
                }
                self.action_num = 1; //idle
            }
            //Set light power
            14 => {
                //divide by 1000 (1003 = 1.003)
                self.light_power = self.pass_var_a as f32 / 1000.0;
                self.action_num = 1; //idle
            }
            //Set light cone width
            15 => {
                //clamp to 360
                if self.pass_var_a > 359 {self.pass_var_a = 359}

                self.light_angle = 0..self.pass_var_a as i32;

                self.action_num = 1; //goto nothing
            }

            _ => {}
        }

        //tick the animator
        self.action_counter = self.action_counter.wrapping_add(1);

        //use ID to start the stage light in a "unique" spot (will never overload because action_counter will never exceed u16)
        let unique_action_counter = self.action_counter as u32 + (self.id * 100) as u32;

        //color mode
        match self.gen_var_a
        {
            //gradient mode
            1 => {
                let angle = unique_action_counter as f32 / self.gen_var_b as f32;

                self.light_color.r = angle.sin();
                self.light_color.g = (angle + 2.0*PI/3.0).sin();
                self.light_color.b = (angle + 4.0*PI/3.0).sin();

                self.npc_flags.set_emits_cone_light(true)

            }

            //strobe mode
            2 => {
                
                if (self.action_counter as i32 / self.gen_var_b) % 2 == 0
                {
                    self.npc_flags.set_emits_cone_light(false);
                }
                else {self.npc_flags.set_emits_cone_light(true)}
            }

            //turn light off
            3 => {
                self.npc_flags.set_emits_cone_light(false);
            }

            //flat color mode
            _ => {
                self.npc_flags.set_emits_cone_light(true)
            }
        }


        self.light_angle = center_light_cone(PI / 2.0, &self.light_angle);

        self.anim_rect = state.constants.npc.n373_flood_light[0];

        Ok(())

    }

    //strobe lights
    pub(crate) fn tick_n374_strobe_light(&mut self, state: &mut SharedGameState) -> GameResult {

        //establish stage light behavior
        match self.action_num
        {
            
            //initialize
            0 => {

                //TEST
                // self.gen_var_a = 1; //light mode
                // self.gen_var_b = 40; //effect rate
                
                //flag light as emissive
                self.npc_flags.set_emits_point_light(true);

                //set light direction (not needed for point light)
                //self.light_angle = 0..20;

                //put on top layer if we use the alt direction
                if self.direction != Direction::Left {self.layer = NPCLayer::Foreground}


                self.action_num = 1; //goto nothing
            }

            //color modes

            //set color R
            10 => {
                //clamp to char size
                if self.pass_var_a > 0xFF {self.pass_var_a = 0xFF}
                //normalize and set red
                self.light_color.r = self.pass_var_a as f32/ 255.0;

                self.action_num = 1; //idle
            }
            //set color G
            11 => {
                //clamp to char size
                if self.pass_var_a > 0xFF {self.pass_var_a = 0xFF}
                //normalize and set red
                self.light_color.g = self.pass_var_a as f32/ 255.0;

                self.action_num = 1; //idle
            }
            //set color B
            12 => {
                //clamp to char size
                if self.pass_var_a > 0xFF {self.pass_var_a = 0xFF}
                //normalize and set red
                self.light_color.b = self.pass_var_a as f32/ 255.0;

                self.action_num = 1; //idle
            }
            //set color mode
            13 => {

                //remember mode
                self.gen_var_a = self.pass_var_a as i32;

                match self.pass_var_a
                {
                    //color gradient mode and strobe mode take speed parameter
                    1 | 2=>{

                        self.gen_var_b = self.pass_var_b as i32;
                    }

                    //flat color mode
                    _ =>{}
                }
                self.action_num = 1; //idle
            }
            //Set light power
            14 => {
                //divide by 1000 (1003 = 1.003)
                self.light_power = self.pass_var_a as f32 / 1000.0;
                self.action_num = 1; //idle
            }

            _ => {}
        }

        //tick the animator
        self.action_counter = self.action_counter.wrapping_add(1);

        //use ID to start the stage light in a "unique" spot (will never overload because action_counter will never exceed u16)
        let unique_action_counter = self.action_counter as u32 + (self.id * 100) as u32;

        //color mode
        match self.gen_var_a
        {
            //gradient mode
            1 => {
                let angle = unique_action_counter as f32 / self.gen_var_b as f32;

                self.light_color.r = angle.sin();
                self.light_color.g = (angle + 2.0*PI/3.0).sin();
                self.light_color.b = (angle + 4.0*PI/3.0).sin();

                self.npc_flags.set_emits_point_light(true)

            }

            //strobe mode
            2 => {
                
                if (self.action_counter as i32 / self.gen_var_b) % 2 == 0
                {
                    self.npc_flags.set_emits_point_light(false);
                }
                else {self.npc_flags.set_emits_point_light(true)}
            }

            //turn light off
            3 => {
                self.npc_flags.set_emits_point_light(false);
            }

            //flat color mode
            _ => {
                self.npc_flags.set_emits_point_light(true)
            }
        }


        //self.light_angle = center_light_cone(PI / 2.0, &self.light_angle);

        self.anim_rect = state.constants.npc.n374_strobe_light[0];

        Ok(())

    }

    //spawns and coordinates the actions of the crowd
    pub(crate) fn tick_n375_crowd_manager(&mut self, state: &mut SharedGameState, stage: &mut Stage, npc_list: &NPCList,) -> GameResult {

        match self.action_num
        {
            //init
            0 => {

                let mut min_x = 0;
                let mut max_x = 0;

                let crowd_per_tile = 1;

                // find limits to spawn NPCs
                let recognized_solids: [u8; 9] = [0x02, 0x60, 0x61, 0x04, 0x05, 0x41, 0x43, 0x46, 0x03];

                let tile_size = state.tile_size.as_int() * 0x200;
                let mut x = self.x / tile_size;
                let y = self.y / tile_size;

                while x > 0
                {
                    //get attribute for this tile
                    let aa = stage.map.get_attribute((x) as usize, (y) as usize);
                
                    //found solid block, mark edge
                    if recognized_solids.contains(&aa) {
                        min_x = x;
                        break;
                    }
                    else {
                        x -= 1;
                    }

                }

                //reset x
                x = self.x / tile_size;
                while x < stage.map.width as i32
                {
                    //get attribute for this tile
                    let aa = stage.map.get_attribute((x) as usize, (y) as usize);
                
                    //found solid block, mark edge
                    if recognized_solids.contains(&aa) {
                        max_x = x;
                        break;
                    }
                    else {
                        x += 1;
                    }
                }

                //only continue of we have a range to lay crowd members down on
                if min_x != max_x
                {
                    //for each tile in the span, create an NPC
                    for i in (min_x + 1)..(max_x - 1)
                    {
                        
                        for _o in 0..crowd_per_tile
                        {
                            
                            // spawn crowd member
                            let mut npc = NPC::create(376, &state.npc_table);
                            npc.cond.set_alive(true);
                            npc.direction = self.direction.clone();
                            npc.x = i * tile_size + self.rng.range(-8..8) * 0x200;
                            npc.y = self.y;

                            //choose NPC skin
                            npc.action_counter3 = self.rng.range(0..4) as u16;

                            //choose NPC draw level
                            npc.layer = match self.rng.range(0..2)
                            {
                                0 => NPCLayer::Background,
                                1 => NPCLayer::Middleground,
                                _ => NPCLayer::Foreground,
                            };

                            let _ = npc_list.spawn(0x100, npc.clone());

                        }
                    }
                }

            
            }

            _ => {}
        }

        Ok(())
    }

    pub(crate) fn tick_n376_crowd_member(&mut self, state: &mut SharedGameState) -> GameResult {


        let mut rect_offset = 0;

        match self.action_num
        {
            //init, choose random action
            0 => {

                match self.rng.range(0..120)
                {
                    0 => {
                        self.action_num = 10; //look up
                    }
                    1 => {
                        self.action_num = 20; //hop
                    }

                    _ => {
                        //do nothing
                    }
                }
            }

            //look up
            10 => {
                //self.vel_y = -0x200;
                //rect_offset = 2;
                self.action_num = 0;
            }

            //hop
            20 => {
                self.vel_y = self.rng.range(-0x400..-0x200);
                self.action_num = 21;
                rect_offset = 5;
            }
            //watch for decent
            21 => {

                //go back to choice, we hit the ground
                if self.flags.hit_bottom_wall()
                {
                    self.action_num = 0;
                }
                else
                {
                    //if falling down
                    if self.vel_y > 0 {
                        rect_offset = 4; //hands at side
                    }
                    else {
                        rect_offset = 5; //hands out
                    }
                }

            }

            //random delay hop
            30 => {
                self.action_counter = self.rng.range((self.pass_var_a as i32)..(self.pass_var_b as i32)) as u16;
                self.action_num = 31;
            }
            31 => {
                
                //goto jump when timed out
                if self.action_counter > 0
                {
                    self.action_counter -= 1;
                } 
                else {
                    self.action_num = 20;
                }
            }


            _ => {
                //do nothing
            }
        }

        //gravity
        self.vel_y += 0x40;

        self.vel_x = self.vel_x.clamp(-0x400, 0x400);
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;


        //manage blinking only for certain main-actions
        let mut blink_offset = 0;
        if !([20, 21].contains(&self.action_num))
        {
            //if eyes should be closed
            if self.action_counter2 > 0
            {
                self.action_counter2 -= 1;
                if self.action_counter2 <= 0 {
                    self.action_num = 0;
                }
                else {
                    blink_offset = 1;                    
                }
            }
            else if self.rng.range(0..120) == 10{
                self.action_counter2 = 8; //close eyes for 8 ticks
            }
        }


        self.anim_num = blink_offset + rect_offset;

        self.anim_rect = state.constants.npc.n376_crowd_member[self.anim_num as usize];
        
        //shift down by direction
        if self.direction != Direction::Left
        {
            let height = self.anim_rect.height();
            self.anim_rect.top += height;
            self.anim_rect.bottom += height;
        }

        //shift down by skin variable, which here we are using as actionCounter3
        let height = self.anim_rect.height();
        self.anim_rect.top += height * self.action_counter3 * 2;
        self.anim_rect.bottom += height * self.action_counter3 * 2;



        Ok(())
    }


    //band members:
    //guitar
    //bass
    //vocalist
    //keys
    //drummer
    pub(crate) fn tick_n377_band_member(&mut self, state: &mut SharedGameState) -> GameResult {

        match self.action_num
        {
            //init / set parameters
            0 => {

                //skin offset
                self.gen_var_a = self.pass_var_a as i32;
                //speed
                self.gen_var_b = self.pass_var_b as i32;
                self.action_num = 3;

            }

            //Animate at speed
            1 =>
            {
                self.anim_counter += 1;
                if self.anim_counter > self.gen_var_b as u16
                {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 7
                    {
                        self.anim_num = 0;
                    }
                }
            }

            //idle (TODO: idle animation)
            _ => {
                self.anim_num = 0;
            }
        }

        self.anim_rect = state.constants.npc.n377_band_member[self.anim_num as usize];

        //used to switch between left and right (vertically stacked)
        let vert_offset = self.anim_rect.height() * 8 * if self.direction == Direction::Left {0} else {1};
        //used to choose the band member
        let horiz_offset =  self.anim_rect.width() * self.gen_var_a as u16;

        self.anim_rect.left += horiz_offset;
        self.anim_rect.right += horiz_offset;
        self.anim_rect.top += vert_offset;
        self.anim_rect.bottom += vert_offset;

        

        Ok(())
    }





}

