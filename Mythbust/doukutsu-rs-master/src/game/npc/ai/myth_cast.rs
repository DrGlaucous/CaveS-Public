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
    pub fn tickt_n376_mini_buster
    (
        &mut self, //self-refrence
        state: &mut SharedGameState, //global game state
        players: [&mut Player; 2], //where the players are
        //npc_list: &NPCList,
    ) ->GameResult
    {
        let player = self.get_closest_player_ref(&players);

        //switch actions
        match self.action_num
        {
            0 =>
            {
                self.direction = if player.x < self.x {Direction::Left} else {Direction::Right};

            }
            _ =>{}
        }
 

        self.anim_rect = state.constants.npc.n371_adam[self.anim_num as usize];

        return Ok(())
    }



}
