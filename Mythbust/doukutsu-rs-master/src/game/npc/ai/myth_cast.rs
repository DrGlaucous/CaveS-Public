use crate::common::Direction;
use crate::framework::error::GameResult;
//use crate::game::npc::list::NPCList; //add this if you want to refrence other NPCs
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
        let now;// = 0;
        match  std::time::SystemTime::now().duration_since( std::time::SystemTime::UNIX_EPOCH)
        {
            Ok(n) => 
            {
                now = n.as_millis();
            }
            Err(_) => panic!("SystemTime before UNIX EPOCH!")
        }
        self.angle = core::f32::consts::PI * (((now % 100000) as f32)/ 1000.0); 

        //anchor on NPC center
        self.anchor_x = ((self.anim_rect.right - self.anim_rect.left) / 2) as f32;
        self.anchor_y = ((self.anim_rect.bottom - self.anim_rect.top) / 2) as f32;

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
            self.action_num = 10;
    
        }

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
                    self.action_num = 3;
                    self.direction = if (self.rng.range(0..9) % 2) != 0 {Direction::Left} else {Direction::Right}
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

                //why doesn't rust let me increment?
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
                if self.flags.hit_bottom_wall() && (self.flags.hit_right_wall() || self.flags.hit_right_wall())
                {
                    self.vel_y -= 0x200;
                }

            }

            _ =>{/*do nothign*/}

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



}
