use std::borrow::Borrow;
use std::hint::unreachable_unchecked;
//use std::intrinsics::sqrtf32;

use num_traits::Pow;

use crate::common::{Direction, Rect, get_dist};
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
        if self.action_counter3 == 0
        {
            //create cursor
            let mut npc_c = NPC::create(380, &state.npc_table);
            npc_c.cond.set_alive(true);
            npc_c.parent_id = self.id;

            //create hammer
            let mut npc_h = NPC::create(379, &state.npc_table);
            npc_h.cond.set_alive(true);
            npc_h.parent_id = self.id;

            //create pole
            let mut npc_p = NPC::create(381, &state.npc_table);
            npc_p.cond.set_alive(true);
            npc_p.parent_id = self.id;

            //create hand
            let mut npc_ha: NPC = NPC::create(382, &state.npc_table);
            npc_ha.cond.set_alive(true);
            npc_ha.parent_id = self.id;
            npc_ha.direction = Direction::Left;


            //spawn cursor, keep ID to add to child list
            if let Ok(cursor_id) = npc_list.spawn(0x100, npc_c.clone())
            {self.child_ids.push(cursor_id);} //cursor has index 0
            if let Ok(cursor_id) = npc_list.spawn(0x100, npc_h.clone())
            {self.child_ids.push(cursor_id);} //hammer has index 1
            if let Ok(cursor_id) = npc_list.spawn(0x100, npc_p.clone())
            {self.child_ids.push(cursor_id);} //pole has index 2
            if let Ok(cursor_id) = npc_list.spawn(0x100, npc_ha.clone())
            {self.child_ids.push(cursor_id);} //pole has index 2
            npc_ha.direction = Direction::Right;
            if let Ok(cursor_id) = npc_list.spawn(0x100, npc_ha.clone())
            {self.child_ids.push(cursor_id);} //pole has index 2


            self.action_counter3 = 1;

        }


        //set direction
        //look for the hammer
        let mut ids = 0;
        for u in self.child_ids.as_slice()
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
            self.direction = if hammer.x > self.x{Direction::Right} else {Direction::Left};

        }


        //animate (using action_counter in lieu of action_num)
        match self.action_counter2 {
            0 | 1 => {
                if self.action_counter2 == 0 {
                    self.action_counter2 = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.vel_x = 0;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_counter2 = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_counter2 = 1;
                    self.anim_num = 0;
                }
            }
            _ => {self.action_counter2 = 0;}
        }

        //snap player to us
        match self.action_num
        {
            //action 1: snap to player, then begin action 2
            1 => {
                self.x = players[0].x;
                self.y = players[0].y;
                self.action_num = 2;
            }
            //action 2: snap player to us
            2 => {
                players[0].x = self.x;
                players[0].y = self.y;
            }
            _ => {}
        }


        //test: follow PC for now
        //let tgt_x = players[0].x;
        //let tgt_y = players[0].y;
        //todo: floor or ceiling condition for this
        if self.flags.hit_top_wall()
        {self.vel_x = self.vel_x * 4 / 5;}

        self.vel_y += 0x40;

        //speed limit
        let sp_lim = 0x600;
        if self.vel_x > sp_lim
        {self.vel_x = sp_lim}
        if self.vel_x < -sp_lim
        {self.vel_x = -sp_lim}

        if self.vel_y > sp_lim
        {self.vel_y = sp_lim}
        if self.vel_y < -sp_lim
        {self.vel_y = -sp_lim}


        self.x += self.vel_x;
        self.y += self.vel_y;

        //self.layer = NPCLayer::Foreground;

        if self.direction == Direction::Left {
            self.anim_rect = state.constants.npc.n378_climber_base[self.anim_num as usize]; //left
        }
        else {
            self.anim_rect = state.constants.npc.n378_climber_base[2 + self.anim_num as usize]; //right
        }

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

        self.anim_rect = state.constants.npc.n379_382_climber_parts[1];

        Ok(())
    }


    pub(crate) fn tick_n380_climber_cursor(
        &mut self,
        state: &mut SharedGameState,
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



        //use tgt_x instead of x and vel_x2 instead of vel_x to have the cursor move realtive to the base entity
        //follow mouse
        (self.vel_x2, self.vel_y2) = (ctx.mouse_context.abs_mouse_coords.0 as i32 * 0x200, ctx.mouse_context.abs_mouse_coords.1 as i32 * 0x200);

        //hammer drift 
        self.vel_x2 += (tgt_drif_x - self.x) / 4;
        self.vel_y2 += (tgt_drif_y - self.y) / 4;

        //x, y relative to the base npc
        self.target_x += self.vel_x2;
        self.target_y += self.vel_y2;


        self.x = tgt_cons_x + self.target_x;
        self.y = tgt_cons_y + self.target_y;

        //self.x += self.vel_x;
        //self.y += self.vel_y;





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

        self.anim_rect = state.constants.npc.n379_382_climber_parts[0];

        Ok(())
    }


    pub(crate) fn tick_n381_stick(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        

        let mut tgt_x = 0;
        let mut tgt_y = 0;
        //get coordinate of the parent (the climber base)
        if let Some(base) = npc_list.get_npc(self.parent_id as usize)
        {
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
                self.x = hammer.x;
                self.y = hammer.y;
            }
            //get centerpoint location of the base NPC
            tgt_x = base.x;
            tgt_y = base.y;
        }

        self.angle = ((tgt_y - self.y) as f32).atan2((tgt_x - self.x) as f32);


        self.anchor_x = (self.display_bounds.left / 0x200) as f32;
        self.anchor_y = (self.display_bounds.top / 0x200) as f32;

        self.anim_rect = state.constants.npc.n379_382_climber_parts[2];
        
        Ok(())

    }

    pub(crate) fn tick_n382_hand(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        

        let mut tgt_x = 0;
        let mut tgt_y = 0;
        //get coordinate of the parent (the climber base)
        if let Some(base) = npc_list.get_npc(self.parent_id as usize)
        {
            //look for the stick
            let mut ids = 0;
            for u in base.child_ids.as_slice()
            {
                if let Some(npc) = npc_list.get_npc(*u as usize)
                {
                    if npc.npc_type == 381 //is stick
                    {
                        ids = *u;
                        break;
                    }
                }
            }

            //get coordinates of the stick
            if let Some(stick) = npc_list.get_npc(ids as usize)
            {
                //grab 8 pixels from the end
                let grab_along = if self.direction == Direction::Left {8} else {10};

                tgt_x = stick.x + (stick.angle.cos() * (stick.display_bounds.right - grab_along * 0x200) as f32) as i32;
                tgt_y = stick.y + (stick.angle.sin() * (stick.display_bounds.right - grab_along * 0x200) as f32) as i32;
                
            }
            //get centerpoint location of the base NPC
            self.y = base.y - 3 * 0x200;
            self.x  = if self.direction == Direction::Left {base.x - 4 * 0x200} else {base.x + 4 * 0x200} 

        }

        self.angle = ((tgt_y - self.y) as f32).atan2((tgt_x - self.x) as f32);


        self.anchor_x = (self.display_bounds.left / 0x200) as f32;
        self.anchor_y = (self.display_bounds.top / 0x200) as f32;

        if self.direction == Direction::Left {
            self.anim_rect = state.constants.npc.n379_382_climber_parts[3];
        }
        else {
            self.anim_rect = state.constants.npc.n379_382_climber_parts[4];
        }
        
        Ok(())

    }



}


