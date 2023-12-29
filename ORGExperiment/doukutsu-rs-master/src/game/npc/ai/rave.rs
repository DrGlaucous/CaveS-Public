use std::f32::consts::PI;
use std::process::Child;
use core::ops::Range;

use crate::common::Direction;
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
    let l_radius = (cone_width.end - cone_width.start).abs() / 2;
    let avar = (rad_to_deg(npc_angle)-l_radius)..(rad_to_deg(npc_angle)+l_radius);
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


impl NPC {


    //the active part of the GOBO
    pub(crate) fn tick_n371_gobo(&mut self, state: &mut SharedGameState, npc_list: &NPCList,) -> GameResult {

        match self.action_num
        {
            //initialize
            0 =>
            {
                // spawn base
                let mut npc = NPC::create(372, &state.npc_table);
                npc.cond.set_alive(true);

                //get NPC id from the child npc spawning in, verify that it was created correctly
                // let nnid = match npc_list.spawn(0x100, npc.clone())
                // {
                //     Ok(rra) => rra as i32,
                //     Err(_) => -1,
                // };
                // if nnid == -1
                // {
                //     //go to idle case because initialization failed
                //     self.action_num = 998; //some random number
                //     return Ok(());
                // }

                //re-using 'parent' as child
                self.parent_id = nnid as u16;

                //set up the child NPC if there is one
                //if let Some(npc) = self.get_parent_ref_mut(npc_list)
                {
                    //pass the child our ID
                    npc.parent_id =  self.id;
                    //re-using parent_id to keep track of child
                    self.parent_id = npc.id;

                    //head emits light
                    npc.npc_flags.set_emits_cone_light(true);
                    //npc.light_power = 0.0; //start with light off
                    npc.light_power = 1.0;

                    //for initialization, we just need width, we'll compute direction further down the line
                    npc.light_angle = 0..20;

                    //set rotation anchor (hitbox needs to be the same size as the RECt for this to work)
                    npc.anchor_x = (npc.hit_bounds.width() / 2) as f32;
                    npc.anchor_y = (npc.hit_bounds.height() / 2) as f32 + 2.0;

                    //alt direction, spawn hanging
                    if self.direction != Direction::from_int(0).unwrap()
                    {
                        //npc.angle = 3.0 * PI / 2.0; //point GOBO head down

                        //take light width and center it on GOBO direction
                        npc.light_angle = center_light_cone(npc.angle, &npc.light_angle);
                        

                        //place below
                        npc.x = self.x;
                        npc.y = self.y + 8;
                    }
                    else
                    {
                        //npc.angle = PI / 2.0; //point GOBO head up
                        npc.light_angle = center_light_cone(npc.angle, &npc.light_angle);
                        
                        //place above
                        npc.x = self.x;
                        npc.y = self.y - 8;
                    }
                }

                self.action_num = 1;


            }

            //default (do nothing)
            _ => {}
        }


        self.anim_rect = state.constants.npc.n371_gobo[0];


        Ok(())
    }

    //the base of the gobo unit
    pub(crate) fn tick_n372_gobo_base(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {

        //self.get_parent_ref_mut(npc_list)

        self.anim_rect = state.constants.npc.n372_gobo_base[0];
        Ok(())
    }
}

