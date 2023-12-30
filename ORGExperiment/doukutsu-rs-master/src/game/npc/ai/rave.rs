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
use crate::util::rng::RNG;

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

//rust raycast unit circle:

//        90
//        |
//        |
//<--0---[ ]---180-->
//        |
//        |
//       270

//rust sprite rotation unit circle:
//      3PI/2
//        |
//        |
//<--PI---[ ]---0-->
//        |
//        |
//       PI/2

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

            //Set animation speed
            20 => {
                //re-using anim_num to store animation speed
                self.anim_num = self.pass_var_a;
                self.action_num = 1; //idle
            }
            //set animation range (the two angles the gobo will rock between when in animation state)
            21 => {
                self.action_counter2 = self.pass_var_a;
                self.action_counter3 = self.pass_var_b;
                self.action_num = 1; //idle
            }
            //goto angle
            22 => {

                self.action_num = 1; //idle
            }



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

                self.npc_flags.set_emits_point_light(true)
            }

            //strobe mode
            2 => {
                
                if (self.action_counter as i32 / self.gen_var_b) % 2 == 0
                {
                    self.npc_flags.set_emits_cone_light(false);
                }
                else {self.npc_flags.set_emits_cone_light(true)}
            }

            //flat color mode
            _ => {
                self.npc_flags.set_emits_cone_light(true)
            }
        }

        let min_limit = self.action_counter2 as f32 / 1000.0;
        let max_limit = self.action_counter3 as f32 / 1000.0;
        let step_size = self.anim_num as f32 / 1000.0;

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
                    //within range, start motion
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


            //drive to position and halt, linear
            30 => {
                //determine shortest way
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
                self.gen_var_a = 1; //light mode
                self.gen_var_b = 40; //effect rate
                
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

            //flat color mode
            _ => {
                self.npc_flags.set_emits_cone_light(true)
            }
        }


        self.light_angle = center_light_cone(PI / 2.0, &self.light_angle);

        self.anim_rect = state.constants.npc.n373_flood_light[0];

        Ok(())

    }





}

