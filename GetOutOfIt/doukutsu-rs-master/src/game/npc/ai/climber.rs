use std::borrow::Borrow;
use std::hint::unreachable_unchecked;
//use std::intrinsics::sqrtf32;

use num_traits::Pow;

use crate::common::{Direction, Rect};
use crate::components::flash::Flash;
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::{NPC, NPCLayer};
use crate::game::npc::boss::BossNPC;
use crate::game::npc::list::NPCList;
use crate::game::player::Player;
use crate::game::shared_game_state::{GameDifficulty, SharedGameState};
use crate::game::stage::Stage;
use crate::util::rng::RNG;
use crate::framework::context::Context;

fn get_dist((x1, y1): (f32, f32), (x2, y2): (f32, f32)) -> f32
{
    if let ((sqr_1), (sqr_2)) = ((x2 - x1).pow(2.0), (y2 - y1).pow(2.0))
    {
        return (sqr_1 + sqr_2).sqrt();
    }
    else
    {
        return 0.0;
    }
}

impl NPC {

    //I'm not deleting the leftover mythbster npcs, so this will start at 378


    //how the climber works:
    //climber base (kampachi in bucket)
    //climber hammer (fishing rod end)
    //cursor (controled by mouse)

    //cursor object is constrained within a radius of the climber base, but drifts toward the climber hammer
    //hammer is constrained within a radius of the base (slightly smaller than that of the hammer, by about 1 tile)
    //hammer is also constrained outside a radius of the base (about 1 tile again)
    //hammer drifts toward cursor, accelerating to a max of booster2 speed relative to climber base


    pub(crate) fn tick_n378_climber_base(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2], 
        npc_list: &NPCList) -> GameResult {

        //construct and spawn in a cursor and a hammer, centered on the base
        if self.action_num == 0
        {
            //create cursor
            let mut npc_c = NPC::create(380, &state.npc_table);
            npc_c.cond.set_alive(true);
            npc_c.parent_id = self.id;

            //create hammer
            let mut npc_h = NPC::create(379, &state.npc_table);
            npc_h.cond.set_alive(true);
            npc_h.parent_id = self.id;

            //spawn cursor, keep ID to add to child list
            if let Ok(cursor_id) = npc_list.spawn(0x100, npc_c.clone())
            {self.child_ids.push(cursor_id);} //cursor has index 0
            if let Ok(cursor_id) = npc_list.spawn(0x100, npc_h.clone())
            {self.child_ids.push(cursor_id);} //hammer has index 1

            self.action_num = 1;

        }

        //test: follow PC for now
        let tgt_x = players[0].x;
        let tgt_y = players[0].y;

        self.x = tgt_x;
        self.y = tgt_y;

        //self.layer = NPCLayer::Foreground;

        self.anim_rect.left = 16;
        self.anim_rect.top = 0;
        self.anim_rect.right = 32;
        self.anim_rect.bottom = 16;

        Ok(())
    }

    pub(crate) fn tick_n379_climber_hammer(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2], 
        npc_list: &NPCList) -> GameResult {



        //location of radius of constraint
        let mut tgt_cons_x = 0;
        let mut tgt_cons_y = 0;
        //velocity of the constraint
        let mut tgt_cons_vx = 0;
        let mut tgt_cons_vy = 0;

        //location of drift target
        let mut tgt_drif_x = 0;
        let mut tgt_drif_y = 0;


        //get coordinate of the parent (the climber base)
        if let Some(base) = npc_list.get_npc(self.parent_id as usize)
        {
            tgt_cons_x = base.x;
            tgt_cons_y = base.y;

            //get velocity without relying on vel_x or vel_y
            //we story old veloc in tgt_x
            tgt_cons_vx = base.x - self.target_x;
            tgt_cons_vy = base.y - self.target_y;

            self.target_x = base.x;
            self.target_y = base.y;


            //look for the cursor
            let mut ids = 0;
            for u in base.child_ids.as_slice()
            {
                if let Some(npc) = npc_list.get_npc(*u as usize)
                {
                    if npc.npc_type == 380 //is hammer
                    {
                        ids = *u;
                        break;
                    }
                }
            }

            //get coordinates of the cursor
            if let Some(cursor) = npc_list.get_npc(ids as usize)
            {
                tgt_drif_x = cursor.x;
                tgt_drif_y = cursor.y;
            }
        }

        //oscillation problems:
        //if we're heading towards the target, speed up at a slower speed
        //if we're heading away from the target, slow down at a higher speed

        //drift 
        self.vel_x = (tgt_drif_x - self.x) / 4;
        self.vel_y = (tgt_drif_y - self.y) / 4;

        //speed limit (relative)
        let sp_lim = 0x1000;
        if self.vel_x > sp_lim + tgt_cons_vx
        {self.vel_x = sp_lim + tgt_cons_vx}
        if self.vel_x < -sp_lim + tgt_cons_vx
        {self.vel_x = -sp_lim + tgt_cons_vx}

        if self.vel_y > sp_lim + tgt_cons_vy
        {self.vel_y = sp_lim + tgt_cons_vy}
        if self.vel_y < -sp_lim + tgt_cons_vy
        {self.vel_y = -sp_lim + tgt_cons_vy}


        self.x += self.vel_x;
        self.y += self.vel_y;

        //limit distance
        let dist = get_dist((self.x as f32, self.y as f32), (tgt_cons_x as f32, tgt_cons_y as f32));
        let dist_max = (48 * 0x200) as f32;
        let dist_min = (16 * 0x200) as f32;
        let angle = ((tgt_cons_y - self.y) as f32).atan2((tgt_cons_x - self.x) as f32);
        if dist > dist_max
        {
            self.x = tgt_cons_x - (angle.cos() * dist_max) as i32;
            self.y = tgt_cons_y - (angle.sin() * dist_max) as i32;
        }
        else if dist < dist_min
        {
            self.x = tgt_cons_x - (angle.cos() * dist_min) as i32;
            self.y = tgt_cons_y - (angle.sin() * dist_min) as i32;
        }

        self.anim_rect.left = 0;
        self.anim_rect.top = 16;
        self.anim_rect.right = 16;
        self.anim_rect.bottom = 32;

        Ok(())
    }


    pub(crate) fn tick_n380_climber_cursor(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2], 
        npc_list: &NPCList,
        ctx: &Context) -> GameResult {

        //location of radius of constraint
        let mut tgt_cons_x = 0;
        let mut tgt_cons_y = 0;

        //location of drift target
        let mut tgt_drif_x = 0;
        let mut tgt_drif_y = 0;

        //in parent child ID set:
        //cursor has index 0
        //hammer has index 1
        self.layer = NPCLayer::Foreground;

        //get coordinate of the parent (the climber base)
        if let Some(base) = npc_list.get_npc(self.parent_id as usize)
        {
            tgt_cons_x = base.x;
            tgt_cons_y = base.y;

            //look for the hammer
            let mut ids = 0;
            for u in base.child_ids.as_slice()
            {
                if let Some(npc) = npc_list.get_npc(*u as usize)
                {
                    if npc.npc_type == 379 //is hammer
                    {
                        ids = *u;
                        break;
                    }
                }
            }

            //get coordinates of the hammer
            if let Some(hammer) = npc_list.get_npc(ids as usize)
            {
                tgt_drif_x = hammer.x;
                tgt_drif_y = hammer.y;
            }
        }





        //follow mouse
        (self.vel_x, self.vel_y) = (ctx.mouse_context.abs_mouse_coords.0 as i32 * 0x200, ctx.mouse_context.abs_mouse_coords.1 as i32 * 0x200);

        //drift 
        self.vel_x += (tgt_drif_x - self.x) / 4;
        self.vel_y += (tgt_drif_y - self.y) / 4;

        self.x += self.vel_x;
        self.y += self.vel_y;

        //limit distance
        let dist = get_dist((self.x as f32, self.y as f32), (tgt_cons_x as f32, tgt_cons_y as f32));
        let dist_max = (64 * 0x200) as f32;
        let dist_min = (4 * 0x200) as f32;

        let angle = ((tgt_cons_y - self.y) as f32).atan2((tgt_cons_x - self.x) as f32);

        if dist > dist_max
        {
            self.x = tgt_cons_x - (angle.cos() * dist_max) as i32;
            self.y = tgt_cons_y - (angle.sin() * dist_max) as i32;
        }
        else if dist < dist_min
        {
            self.x = tgt_cons_x - (angle.cos() * dist_min) as i32;
            self.y = tgt_cons_y - (angle.sin() * dist_min) as i32;
        }

        //log::info!("{angle}");




        //self.layer = NPCLayer::Foreground;

        self.anim_rect.left = 0;
        self.anim_rect.top = 0;
        self.anim_rect.right = 16;
        self.anim_rect.bottom = 16;

        Ok(())
    }




}


