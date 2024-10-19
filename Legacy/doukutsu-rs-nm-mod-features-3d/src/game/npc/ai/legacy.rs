use crate::common::{Direction, Rect};
use crate::framework::error::GameResult;

use crate::game::npc::NPC;



impl NPC {

    //frames of Mik laying down and getting up
    pub fn tick_n371_mik(
        &mut self
    ) -> GameResult {

        let left_frames = [
            Rect::new(0, 0, 48, 48), //lay
            Rect::new(48, 0, 96, 48), //head up
            Rect::new(96, 0, 144, 48), //craw low
            Rect::new(144, 0, 192, 48), //crawl high

            Rect::new(0, 48, 48, 96), //sit transition
            Rect::new(48, 48, 96, 96), //sit
            Rect::new(96, 48, 144, 96), //squat
            Rect::new(144, 48, 192, 96), //stand
        ];

        let right_frames = [
            Rect::new(0, 96, 48, 144),
            Rect::new(48, 96, 96, 144),
            Rect::new(96, 96, 144, 144),
            Rect::new(144, 96, 192, 144),

            Rect::new(0, 144, 48, 192),
            Rect::new(48, 144, 96, 192),
            Rect::new(96, 144, 144, 192),
            Rect::new(144, 144, 192, 192),
        ];

        let frame_ref = if self.direction == Direction::Left {
            &left_frames.as_slice()
        } else {
            &right_frames.as_slice()
        };


        match self.action_num {


            //get up
            10 | 11 => {

                self.anim_counter += 1;

                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_counter = 0;
                    self.anim_num = 0;
                }
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    //end-of-animation, freeze
                    if self.anim_num >= 7 {
                        self.action_num = 0;
                    }
                }
            }

            //lay down
            20 | 21=> {

                self.anim_counter += 1;
                
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.anim_counter = 0;
                    self.anim_num = 7;
                }
                if self.anim_counter > 4 {
                    self.anim_counter = 0;
                    self.anim_num -= 1;
                    //end-of-animation, freeze
                    if self.anim_num == 0 {
                        self.action_num = 0;
                    }
                }

            }

            _ => {
                //do nothing
            }
        }


        self.anim_rect = frame_ref[self.anim_num as usize];

        
        Ok(())
    }






}