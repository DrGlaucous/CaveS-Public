use std::f32::consts::PI;

use mr_connectome::Connectome;
use winapi::um::winnt::CFG_CALL_TARGET_CONVERT_EXPORT_SUPPRESSED_TO_VALID;

use crate::common::{Direction, Rect};
use crate::framework::error::GameResult;
use crate::game::npc::list::NPCList;
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::util::rng::RNG;

//all NPCs build off this class, defined by the functions placed in here.
//the game runs the corresponding functions based on /src/game/npc/mod.rs
impl NPC {



    pub fn tick_mythcrew(
        &mut self, //self-refrence
        state: &mut SharedGameState, //global game state
        players: [&mut Player; 2], //where the players are (only 2?)
        //npc_list: &NPCList,
        playertype: u16, //what skin to wear (adam, jamie, kari, grant, tory)
    ) -> GameResult
    {

        //default: turn to face nearest player
        //A: talk
        //B: face static direction


        //test flipping
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

        // self.action_counter += 1;
        // if self.action_counter > 50
        // {
        //     self.action_counter = 0;
        //     if (self.angle - core::f32::consts::PI / 2.0).abs() < 0.001
        //     {
        //         self.angle = 0.0;
        //     }
        //     else
        //     {
        //         self.angle = core::f32::consts::PI / 2.0;
        //     }
        // }

        //anchor on NPC center
        //self.anchor_x = (self.display_bounds.left / 0x200) as f32;//((self.display_bounds.left) / 2) as f32;
        //self.anchor_y = (self.display_bounds.top / 0x200) as f32;//((self.anim_rect.bottom - self.anim_rect.top) / 2) as f32;

        //end test


        //switch statement
        match self.action_num
        {
            //turn to face nearest player
            0 =>
            {
                self.anim_counter = 0;
                self.anim_num = 0;
                let player = self.get_closest_player_ref(&players);
                if player.x < self.x {self.direction = Direction::Left} else {self.direction = Direction::Right}
            }
            //talk
            1 =>
            {
                //switch animations
                //if self.rng.range(0..1) == 0
                if true//self.anim_counter.wrapping_add(1) > 5
                {
                    self.anim_counter = 0;
                    self.anim_num = self.rng.range(0..2) as u16; //cycle between taling animations
                }
            }
            //face static direction
            _ =>
            {
                self.anim_counter = 0;
                self.anim_num = 0;
            }


        }

        //velocities
        self.vel_x = 0;
        self.vel_y += 0x20;

        //limit gravity
        if self.vel_y > 0x512 {self.vel_y = 0x512}
        else if self.vel_y < -0x512 {self.vel_y = -0x512}


        //switch direction
        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n371_adam[self.anim_num as usize + dir_offset];

        //shift down based on which skin we're wearing
        let pt_offset = (self.anim_rect.bottom - self.anim_rect.top) * playertype;
        self.anim_rect.top += pt_offset;
        self.anim_rect.bottom += pt_offset;


        return Ok(())

    }

    //run the function above for each of these NPCs
    pub fn tick_n371_thru_n375(
        &mut self, //self-refrence
        state: &mut SharedGameState, //global game state
        players: [&mut Player; 2], //where the players are (Note: this needs to be a vector, not a static array)
        //npc_list: &NPCList,
    ) -> GameResult
    {
        match self.npc_type
        {
            //adam
            371 =>{return self.tick_mythcrew(state, players, 0)}
            //jamie
            372 =>{return self.tick_mythcrew(state, players, 1)}
            //kari
            373 =>{return self.tick_mythcrew(state, players, 2)}
            //grant
            374 =>{return self.tick_mythcrew(state, players, 3)}
            //tory (also default because rust requires it)
            375 | _ =>{return self.tick_mythcrew(state, players, 4)}
        }
    }

    //runs and tries to hit the player
    pub fn tick_n376_mini_buster
    (
        &mut self, //self-refrence
        state: &mut SharedGameState, //global game state
        players: [&mut Player; 2], //where the players are
        //npc_list: &NPCList,
    ) ->GameResult
    {



        //will walk randomly until player is within range
        //will try to run to player's location
        //if player is on the same level, will try to lunge towards player
        //if player is above or below, will walk to player X and stand

        let player = self.get_closest_player_ref(&players);

        //stop if OOB, snap to standing animation (widescreen-proof)
        if !(self.x <= player.x + 0x28000
            && self.x >= player.x - 0x28000
            && self.y <= player.y + 0x1E000
            && self.y >= player.y - 0x1E000)
        {
            self.anim_rect = state.constants.npc.n371_adam[0];
            return Ok(());
        }

        //determine agro mode
        if (player.x - self.x).abs() < 0x200 * 16 * 8
            && (player.y - self.y).abs() < 0x200 * 16 * 4
        {
            if self.action_num < 10 {self.action_num = 10} //only init this if we're not already in it
        }
        else if self.action_num > 3 {self.action_num = 0} //only set back if we haven't already done it

        //switch actions
        match self.action_num
        {

            0 | 1 => //init/walk random or check for player
            {

                if self.action_num == 0
                {
                    self.action_counter = 0;
                    self.anim_counter = 0;
                    self.anim_num = 0;
                    self.action_num = 1;
                    self.vel_x = 0;
                }

                else
                {
                    //1 in 30 chance to walk somewhere
                    if self.rng.range(0..60) == 1
                    {
                        self.action_num = 2;
                        self.anim_num = 5;
                        self.anim_counter = 0;
                        self.action_counter = 0;
                    }
                }

            }
            2 | 3 => //walking
            {
                if self.action_num == 2
                {
                    //init starting direction
                    self.action_num = 3;
                    self.direction = if (self.rng.range(0..9) % 2) != 0 {Direction::Left} else {Direction::Right}
                }

                //wall bump
                if self.flags.hit_left_wall() || self.flags.hit_right_wall()
                {
                    self.direction = self.direction.opposite();
                }

                //animate walk
                self.anim_counter += 1;
                if self.anim_counter > 4
                {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num > 8
                    {
                        self.anim_num = 5;
                    }
                }

                //apply velocity
                self.vel_x = self.direction.vector_x() * 0x200;


                self.action_counter += 1;
                if self.action_counter > 32
                {
                    self.action_num = 0; //back to checking for walking
                }



            }


            10 | 11=> //run to player (init)/go
            {

                //when starting out, snap to correct animation frame
                if self.action_num == 10 && (self.anim_num < 1 ||  self.anim_num < 4)
                {
                    self.anim_num = 1;
                    self.action_num = 11;
                }

                //animate run
                self.anim_counter += 1;
                if self.anim_counter > 4
                {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num > 4
                    {
                        self.anim_num = 1;
                    }
                }

                //evaluate direction
                if player.x < self.x {self.direction = Direction::Left} else {self.direction = Direction::Right}

                //run to player
                self.vel_x += self.direction.vector_x() * 0x20;


                //jump if we're touching a wall to get over it.
                if self.flags.hit_bottom_wall() //note: this does not account for slopes
                    && (self.flags.hit_right_wall() || self.flags.hit_right_wall() //touching wall
                )
                {
                    self.vel_y -= 0x200;
                }

            }

            _ =>{/*do nothing*/}

        }

 
        //grav
        self.vel_y += 0x20;

        //speed limits
        if self.vel_y > 0x50C {self.vel_y = 0x50C} else if self.vel_y < -0x50C {self.vel_y = -0x50C}
        if self.vel_x > 0x400 {self.vel_x = 0x400} else if self.vel_x < -0x400 {self.vel_x = -0x400}

        self.y += self.vel_y;
        self.x += self.vel_x;


        self.anim_rect = state.constants.npc.n376_mini_buster[
            (if self.direction != Direction::Left {self.anim_num + 9} else {self.anim_num}) as usize //right facing rects
        ];

        return Ok(())
    }

    //run NULL program code (set in mod.rs)
    // pub fn tick_n377_light
    // (&mut self)
    // {
    //     self.tick_n000_null();
    // }

    //test connectome NPC
    pub(crate) fn tick_n378_connectome(
        &mut self, 
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {

        //NPC with tank controls

        /*
            unit circle:
              3pi/2
                |
            pi-------0
                |
               pi/2
        */

        self.anim_rect = Rect::new(96, 64, 128, 96);

        //update child NPC rects
        {
            for a in self.child_ids.iter() {
                let npc = npc_list.get_npc(*a as usize).unwrap();

                if npc.flags.hit_anything() {
                    npc.anim_num = 1;
                } else {
                    npc.anim_num = 0;
                }
            }
            
        }

        match self.action_num {
            //connectome init
            0 => {
                //set up connectome
                self.action_num += 1;
                self.connectome = Some(Connectome::new(None));

                //set up rotation offset
                self.anchor_x = (self.anim_rect.right - self.anim_rect.left) as f32 / 2.0;
                self.anchor_y = (self.anim_rect.bottom - self.anim_rect.top) as f32 / 2.0;

                //make child NPCs (order: L B R T, like the unit circle)
                for a in 0..4 {
                    let mut npc = NPC::create(379, &state.npc_table);
                    npc.cond.set_alive(true);
                    let id = npc_list.spawn(0x100, npc).unwrap();
                    self.child_ids.push(id);
                }


            }
            //idle
            1 => {
                self.vel_y2 = 0;
                self.vel_x2 = 0;
                self.vel_x = 0;
                self.angle = 0.0;
            }
            //drive manual
            2 => {

                //distance of each track from the center of the NPC
                let track_distance = 0x200 * 16;
        
                //set movement
                {
                    //right up
                    if players[0].controller.move_up() {
                        self.vel_y2 += 10;
                    }
            
                    //right down
                    if players[0].controller.move_down() {
                        self.vel_y2 -= 10;
                    }

                    //left up
                    if players[0].controller.map() {
                        self.vel_x2 += 10;
                    }

                    //left down
                    if players[0].controller.next_weapon() {
                        self.vel_x2 -= 10;
                    }
                }

                let center_speed = (self.vel_x2 + self.vel_y2) / 2;

                
                let rotation_speed = ((self.vel_x2 - self.vel_y2) as f32).atan2(track_distance as f32 * 2.0);

                self.angle += rotation_speed;
                self.vel_x = center_speed;


        
            }

            //connectome
            10 => {
                if let Some(connectome) = &mut self.connectome {

                    //bump detection
                    {
                        //front
                        let npc = npc_list.get_npc(self.child_ids[0] as usize).unwrap();
                        if npc.flags.hit_anything() {
                            connectome.stimulate_front_bump();
                        }

                        //right
                        let npc = npc_list.get_npc(self.child_ids[1] as usize).unwrap();
                        if npc.flags.hit_anything() {

                            //worm seems to turn away from this side when touched here (which is what we want)
                            connectome.stimulate_anterior_harsh_touch(); //not sure if this is the correct side, but...
                        
                            //connectome.stimulate_posterior_harsh_touch();
                        }

                        //left
                        let npc = npc_list.get_npc(self.child_ids[3] as usize).unwrap();
                        if npc.flags.hit_anything() {
                            connectome.stimulate_posterior_harsh_touch();

                            //connectome.stimulate_anterior_harsh_touch();
                        }

                        //stim food
                        if players[0].controller.move_up() {
                            connectome.stimulate_food();
                        }
                    }

                    connectome.tick();


                    //set track speed
                    let mdl = connectome.get_mdl();
                    let mvl = connectome.get_mvl();
                    let mdr = connectome.get_mdr();
                    let mvr = connectome.get_mvr();

                    //these tend to always be positive, the key is by how much
                    let left_speed = mdl + mvl;
                    let right_speed = mdr + mvr;

                    //turns out we don't really need to clamp the speed here
                    let total_speed = left_speed + right_speed;//.clamp(70, 150);

                    let turn_ratio = if left_speed == 0 {1.0} else {right_speed as f32 / left_speed as f32};

                    self.angle += (turn_ratio - 1.0) * 0.5; //ratio of difference + some random constant to slow it down

                    //todo: better this
                    self.vel_x = total_speed * 3 * if total_speed <= 0 {-1} else {1};



                    self.action_counter2 += 1;
                    if self.action_counter2 > 10 {

                        self.action_counter2 = 0;
                        log::info!("Turn ratio: {:.2},  Speed:{}", turn_ratio, total_speed);
                    }

                    //translate tank controls
                    // {
                    //     //distance of each track from the center of the NPC
                    //     let track_distance = 0x200 * 16;    
                    //     let center_speed = (self.vel_x2 + self.vel_y2) / 2;
                    //     let rotation_speed = ((self.vel_x2 - self.vel_y2) as f32).atan2(track_distance as f32 * 2.0);
                    //     self.angle += rotation_speed;
                    //     self.vel_x = center_speed;
                    // }
                    

                }
            }


            //nothing
            _ => {},
        }

        
        //position child NPCs
        {
            //individual positioning since each one means something different

            let offset_len = 512.0 * 20.0;

            //left (front)
            {
                let npc = npc_list.get_npc(self.child_ids[0] as usize).unwrap();

                let angle = self.angle;

                npc.x = self.x + (offset_len * (angle).cos()) as i32;
                npc.y = self.y + (offset_len * (angle).sin()) as i32;
            }

            //bottom (R rel. to front)
            {
                let npc = npc_list.get_npc(self.child_ids[1] as usize).unwrap();

                let angle = self.angle + PI / 2.0;

                npc.x = self.x + (offset_len * (angle).cos()) as i32;
                npc.y = self.y + (offset_len * (angle).sin()) as i32;
            }

            //right
            {
                let npc = npc_list.get_npc(self.child_ids[2] as usize).unwrap();

                let angle = self.angle + PI;

                npc.x = self.x + (offset_len * (angle).cos()) as i32;
                npc.y = self.y + (offset_len * (angle).sin()) as i32;
            }

            //top (L rel. to front)
            {
                let npc = npc_list.get_npc(self.child_ids[3] as usize).unwrap();

                let angle = self.angle + 3.0 * PI / 2.0;

                npc.x = self.x + (offset_len * (angle).cos()) as i32;
                npc.y = self.y + (offset_len * (angle).sin()) as i32;
            }
        }



        //calculate movement direction
        {
            let angle = self.angle;

            let vx = (angle.cos() * self.vel_x as f32) as i32;
            let vy = (angle.sin() * self.vel_x as f32) as i32;

            self.x += vx;
            self.y += vy;

        }


        Ok(())
    }


    //location determined by connectome, reports a bump on the side (all work done by parent)
    pub(crate) fn tick_n379_connectome_bumper(
        &mut self,
    ) -> GameResult {

        /*
            unit circle:
              3pi/2
                |
            pi-------0
                |
               pi/2
        */

        let rc_list = [
            Rect::new(128, 64, 144, 80), //yel
            Rect::new(128, 80, 144, 96), //pnk
        ];

        self.anim_rect = rc_list[self.anim_num as usize];



        Ok(())
    }



}
