use std::cell::RefCell;
use std::time::SystemTime;


use crate::common::{Color, Rect};
use crate::framework::backend::{BackendTexture, SpriteBatchCommand};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem::{user_create, user_open};
use crate::framework::graphics;
use crate::game::scripting::tsc::text_script::TextScriptExecutionState;
use crate::game::shared_game_state::SharedGameState;
use crate::game::stage::Stage;
use crate::graphics::font::Font;
use crate::graphics::texture_set::SpriteBatch;
use crate::input::touch_controls::TouchControlType;
use crate::engine_constants::EngineConstants;

use crate::components::draw_common::draw_number;
use crate::components::draw_common::Alignment;

use crate::input::dummy_player_controller::DummyPlayerController;
use crate::input::player_controller::PlayerController;

use crate::sound::SoundManager;

use super::shared_game_state::TimingMode;

use serde::{Deserialize, Serialize};



//holds individual map info, vectored in GuitarScores
#[derive(Serialize, Deserialize)]
pub struct LevelScore {
    name: String,
    score: i32,
}
impl LevelScore
{
    pub fn new() ->LevelScore {
        LevelScore {
            name: "".to_owned(),
            score: 0,
        }
    }
}


//holds the settings to be passed off to the json
#[derive(Serialize, Deserialize)]
pub struct GuitarScores {
    stages: Vec<LevelScore>,
}
impl GuitarScores
{
    pub fn new() -> GuitarScores {
        GuitarScores {
            stages: Vec::new(),
        }
    }

}





//notes used by the guitar struct
struct GuitarNote
{
    note_length_decimal: f32, //total length of the note, is represented in board length percentage
    note_length: u8, //length of the note in dots

    //where the front of the note is (for note hit timing)
    //is a decimal from 0 to 1, where 1 is at the bottom and 0 is at the top
    note_head_loc: f32,

    last_time: SystemTime, //the last time the location of the note was updated

    was_hit: bool, //true if the note was pressed

}



//renders and handles the guitar overlay, starting, and stopping
pub struct Guitar
{
    //everything is drawn to this before this is then drawn to the main screen
    texture: RefCell<Option<Box<dyn BackendTexture>>>,
    last_size: RefCell<(u16, u16)>, //used to see if the window has been resized and the texure should also be resized
    ref_size: (f32, f32), //pixel size of this texture, set here so it can be used everywhere

    draw_corners: [[u32;2];4], //screen coordinates where the overlay should be placed
    current_song: usize,
    visible: bool,
    key_state: [bool; 5], //5 keys that can be pressed
    onscreen_time: f32, //how many seconds the note should last on the highway
    current_score: LevelScore,
    pub controller: Box<dyn PlayerController>, //controls the reader (duh)


    //used to determine if a new note is going to be played
    //note_key: [u8; 4], //lengths of last notes created by the tracker
    //note_len: [u8; 4], //key of the last note created by the tracker (note: pitch doesn't matter, only that it changed)
    //above is not needed, note change logic is now handled in get_latest_track_state()
    
    notes: [Vec<GuitarNote>; 4],
}



//set show/hide
impl Guitar
{




    pub fn new() -> Guitar
    {
        Guitar
        {
            texture: RefCell::new(None),
            last_size: RefCell::new((0, 0)),
            ref_size: (96.0, 176.0),
            draw_corners: [[0,0]; 4],
            current_song: 0,
            visible: true,
            key_state: [false; 5], //state of user input
            onscreen_time: 1.0, //2.25,
            current_score: LevelScore::new(),
            controller: Box::new(DummyPlayerController::new()),

            //note_key: [0; 4],
            //note_len: [0; 4],
            notes: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],

        }

    }

    //theory of operation:
    //4 key input, keys are additional options in the keybind menu
    //notes will be spawned in sync with the ORG tracker, so offsets will need to be done in there
    
    //TSC skip behavior will need to be removed
    //TSC can start, pause, and resume a song (stopping = pausing). It can bring up the special save menu described below
    //TSC can speed up and slow down not scroll speed (faster speed means less offset in the tracker file as well as less view time for the notes)
    //Need a function to display and save stats in a visible menu and in a save file
    //need a map selection function that will query all maps in the stage.TBL, one stage per song, flag non-visible maps with what? boss ID maybe?

    //tracker features and implementation
    //channels 1-4 will send patterns to buttons 1-4
    //channels 2-8 will be reserved for co-op?
    //Q will be used to run TSC events, with the event corresponding to the note being sent (only have a range or around 100+)

    //I want the game to mimick this: https://wiki.clonehero.net/books/general-info/

    //notes will award 50 points each x notes in a chord
    //held notes will award 25 points per beat of holding ()
    //multiplier will


    ///////////////////
    /// Helper functions
    ///////////////////

    //look at bound buttons and change trapdoor state accordingly
    fn handle_buttons(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult
    {
        self.controller.update(state, ctx)?;
        self.controller.update_trigger();


        self.key_state[0] = self.controller.one();
        self.key_state[1] = self.controller.two();
        self.key_state[2] = self.controller.three();
        self.key_state[3] = self.controller.four();
        self.key_state[4] = self.controller.strum();

        // if self.controller.trigger_left() || self.controller.trigger_up() || self.controller.trigger_right() || self.controller.trigger_down()
        // {
        //     let mut my_baord = 4;
        //     my_baord += 9;
        // }

        return Ok(());
    }

    //convert a change in time to a percent-of-track completion
    fn get_offset(onscreen_time: f32, time_now: SystemTime, time_then: SystemTime) -> f32
    {

        let delta_time = time_now
            .duration_since(time_then)
            .expect("Err, time moved backwards");

        let delta_micros = delta_time.as_micros();

        let percent_c = (delta_micros as f64) / (1e6 * if onscreen_time < 0.0001 {0.0001} else {onscreen_time as f64});
        percent_c as f32

    }

    //convert note length into a percent-of-track completion
    fn get_length(onscreen_time: f32, note_length: u8, tempo: u16) -> f32
    {
        let time_len_millis = tempo as u32 * note_length as u32;

        let percent_c = (time_len_millis as f32 / 1000.0) / onscreen_time;
        percent_c

    }

    //capture events from the tracker and assign proper behavior
    fn handle_tracker(&mut self, state: &mut SharedGameState)
    {
        //pull in fresh notes
        let notess = state.sound_manager.get_latest_track_state();

        //initialize new notes
        //checks tracks 1-4
        let this_time = state.sound_manager.music_time.now();
        for i in 0..4
        {
            if notess.keys[i] == 0xFF {continue;}

            let change_decimal = Self::get_offset(self.onscreen_time, this_time, state.sound_manager.music_time.from_systime(notess.timestamp));
            let length_decimal = Self::get_length(self.onscreen_time, notess.lengths[i], state.sound_manager.current_commander_tempo());   
            self.notes[i].push( GuitarNote{note_length_decimal: length_decimal, note_length: notess.lengths[i], note_head_loc: change_decimal, last_time: this_time, was_hit: false} );
        }

        //run events with drums
        for i in 0..8
        {
            if notess.drums[i] != 0xFF && notess.drums[i] != 0x00
            {
                //let mut yyt = i;
                //yyt += i;
                //print!("{}\n", notess.drums[i]);
                state.control_flags.set_tick_world(true);
                state.control_flags.set_interactions_disabled(true);
                //state.textscript_vm.executor_player = id;
                state.textscript_vm.start_script(notess.drums[i] as u16);

            }
        }


    }

    //move the notes down their rows and checks for hits from the buttons
    fn handle_notes(&mut self, state: &mut SharedGameState, ctx: &mut Context)
    {

        //50 or 60 FPS
        state.settings.timing_mode;



        //for each note strip

        // for i in 0..4
        // {
        //     //go through all notes backward and remove ones that have moved out of range
        //     //we go backwards because notes are shifted left as they are deleted
        //     for j in (self.notes[i].len() - 1)..=0
        //     {
        //         if self.notes[i][j].note_head_loc - self.notes[i][j].note_length > bottom_coord
        //         {
        //             self.notes[i].remove(j);
        //             continue;
        //         }
        //         //move note down
        //         self.notes[i][j].note_head_loc += 1;
        //     }
        // }

        for n_strip in self.notes.as_mut_slice()
        {
            //go through all notes backward and remove ones that have moved out of range
            //we go backwards because notes are shifted left as they are deleted
            for i in (0..n_strip.len()).rev() //(n_strip.len() - 1)..=0
            {

                if n_strip[i].note_head_loc - n_strip[i].note_length_decimal > 1.0
                {
                    n_strip.remove(i);
                    continue;
                }

                //move note down
                let present = state.sound_manager.music_time.now();
                let down_movement = Self::get_offset(self.onscreen_time, present, n_strip[i].last_time);
                n_strip[i].note_head_loc += down_movement;
                n_strip[i].last_time = present;

                
            }
        }




    }

    //draw a rect, making sure no parts of it can be seen outside the constraints of the larger rect
    fn crop_and_draw_rect(mut x_loc: f32, mut y_loc: f32, source_rect_u16: Rect<u16>, crop_rect: Rect<f32>, batch: &mut Box<dyn SpriteBatch>)
    {
        let mut source_rect = Rect {left: source_rect_u16.left as f32,
                                                top: source_rect_u16.top as f32,
                                                right: source_rect_u16.right as f32,
                                                bottom: source_rect_u16.bottom as f32};

        //xloc and yloc are at the top left corner
        
        //OOB conditions
        if x_loc + source_rect.width() < crop_rect.left
        || y_loc + source_rect.height() < crop_rect.top
        || x_loc > crop_rect.right
        || y_loc > crop_rect.bottom
        { return }

        //partial OOB conditons

        //scraping left wall
        if x_loc < crop_rect.left
        {
            let overshoot = crop_rect.left - x_loc;
            source_rect.left += overshoot;
            x_loc += overshoot;
        }
        //scraping roof
        if y_loc < crop_rect.left
        {
            let overshoot = crop_rect.top - y_loc;
            source_rect.top += overshoot;
            y_loc += overshoot;
        }
        //scraping right wall
        if x_loc + source_rect.width() > crop_rect.right
        {
            let overshoot = (x_loc + source_rect.width() as f32) - crop_rect.right;
            source_rect.right -= overshoot;
        }
        //scraping floor
        if y_loc + source_rect.height() > crop_rect.bottom
        {
            let overshoot = (y_loc + source_rect.height() as f32) - crop_rect.bottom;
            source_rect.bottom -= overshoot;
        }

        batch.add_rect_float(x_loc,
            y_loc,
            1.0,1.0,
            &source_rect);

    }


    ///////////////////
    /// Control functions
    ///////////////////

    //starts a song X with tracker pattern Y
    pub fn start_program(music: String, pattern: String)
    {

    }

    //show or hide the tracker bar
    pub fn set_visibility(&mut self, state: bool)
    {
        self.visible = state;
    }

    //saves the current stats to the map designated by stage number
    pub fn store_stats(&mut self, state: &mut SharedGameState, stage_no: usize)
    {
        if stage_no >= state.stages.len()
        {
            return;
        }
        state.stages[stage_no].score = self.current_score.score;

    }






    ///////////////////
    /// Main ticker functions
    ///////////////////

    //advance the ticker, call this as often as possible
    pub fn update(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult
    {

        //create the surface to draw to
        //the bitmap size depends on window scale, so this surface also needs to change in size with the window
        let width = (self.ref_size.0 * state.scale) as u16;
        let height = (self.ref_size.1 * state.scale) as u16;
        //re-create the surface when the window size changes
        if *self.last_size.borrow() != (width, height)
        {
            *self.last_size.borrow_mut() = (width, height);//.into(RefCell<(u16, u16)>);

            *self.texture.borrow_mut() = graphics::create_texture_mutable(ctx, width, height).ok();
        }

        //get latest input state
        self.handle_buttons(state, ctx);

        //get the latest and greatest notes
        self.handle_tracker(state);

        //move existing notes down the chain
        self.handle_notes(state, ctx);






        Ok(())

    }


    //put note bar onto the screen (this is kind of messy)
    /*
    pub fn draw_old(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult
    {
        if !self.visible {return Ok(())}

        //tallness of the fretboard in pixels (used for note positioning)
        let board_height  = 176.0;

        //rect of the note highway board, split into top, mid, and bottom sections to mask off 
        let board_mid_rect = Rect { left: 0, top: 0, right: 96, bottom: 176};
        

        //rect of the intermediate surface that will be drawn to the screen after everything has been drawn to it
        let board_rect_2: Rect<f32> = Rect { left: 0.0, top: 0.0, right: 96.0 * state.scale, bottom: 176.0 * state.scale };

        let button_inactive = Rect { left: 176, top: 0, right: 192, bottom: 16 };
        let button_active = Rect { left: 176, top: 16, right: 192, bottom: 32 };

        let note_head = Rect { left: 176, top: 48, right: 192, bottom: 64 };
        let note_body = Rect { left: 176, top: 40, right: 192, bottom: 48 };
        let note_tail = Rect { left: 176, top: 32, right: 192, bottom: 48 };

        //push all shapes to the piano roll texture
        {
            //shift rects so we only have to define a few
            fn shift_right(orig_rect: &Rect<u16>, shift: usize) -> Rect<u16>
            {
                let mut new_rect: Rect<u16> = orig_rect.clone();
                let rect_width = new_rect.right - new_rect.left;
                new_rect.left += rect_width * shift as u16;
                new_rect.right = rect_width + new_rect.left;
                new_rect
            }

            //use the piano roll bitmap
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "PianoRoll")?;

            //set the render target to the texture
            graphics::set_render_target(ctx, self.texture.borrow().as_ref())?;
            //erase all old
            graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 0.0));


            //draw note highway
            batch.add_rect(0.0, 0.0, &board_mid_rect);

            let button_offset = 16.0; //where the buttons and notes should start being drawn horizontally


            //draw buttons
            {
                for i in 0..4//self.key_state.len()
                {
                    let button_rect = if self.key_state[i]
                    {shift_right(&button_active, i)}
                    else
                    {shift_right(&button_inactive, i)};

                    batch.add_rect((16 * i) as  f32 + button_offset, board_height - 32.0, &button_rect); 
                }
            }

            //draw notes
            {
                //for all note strips (4 of them)
                for i in 0..self.notes.len()
                {
                    //color changing head
                    let note_h_rect: Rect<u16> = shift_right(&note_head, i);
                    //color changing body
                    let note_b_rect: Rect<u16> = shift_right(&note_body, i);
                    //color changing tail
                    let note_t_rect: Rect<u16> = shift_right(&note_tail, i);




                    //for all notes in the list
                    for j in 0..self.notes[i].len()
                    {
                        //how many pixels are within the note's total length
                        let note_px_len = board_height * self.notes[i][j].note_length_decimal;

                        //draw tail segments
                        {
                            //divide distance from start to end by size of the body, placing a segment for each place

                            //let delta_len = note_b_rect.bottom - note_b_rect.top;
                            //let stub_have = if (note_px_len as i32) % (note_b_rect.bottom - note_b_rect.top) as i32 > 0 {1} else {0};

                            let main_have = (note_px_len as i32) / (note_b_rect.bottom - note_b_rect.top) as i32;
                            for t in 0..(main_have)// + stub_have)
                            {
                                batch.add_rect((16 * i) as  f32 + button_offset,
                                    (self.notes[i][j].note_head_loc * board_height) - (8 * t) as f32,
                                    &note_b_rect);
                            }
                        }
                        //cap with tail tip
                        batch.add_rect((16 * i) as  f32 + button_offset,
                                    ((self.notes[i][j].note_head_loc - self.notes[i][j].note_length_decimal) * board_height) as f32,
                                    &note_t_rect);


                        //draw head
                        batch.add_rect((16 * i) as  f32 + button_offset, self.notes[i][j].note_head_loc * board_height, &note_h_rect);  
                    }

                }
            }




            //blit all shapes to intermediate texture
            batch.draw(ctx)?;

            //set target back to main surface
            graphics::set_render_target(ctx, None)?;

        }


        
        //draw texture onto the main screen
        if let Some(tex) = self.texture.borrow_mut().as_mut()
        {
            tex.clear();
            tex.add(SpriteBatchCommand::DrawRectSkewedTinted(
                board_rect_2, //src

                //top LR
                (64.0 * state.scale, 0.0),
                ((64.0 + 80.0) * state.scale, 0.0),

                //bottom LR
                (0.0, 144.0 * state.scale),
                ((64.0 + 80.0 + 64.0) * state.scale, (144.0) * state.scale),

                //(0.0 * state.scale, 0.0 * state.scale),
                //(64.0 * state.scale, 0.0 * state.scale),
                //(0.0 * state.scale, 144.0 * state.scale),
                //(64.0 * state.scale, 144.0 * state.scale),


                Color::from_rgb(0xFF, 0xFF, 0xFF),
            ));
            tex.draw()?;


        }

 
        Ok(())

    }
    */

    //trying again but making things more modular
    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult
    {

        if !self.visible {return Ok(())}


        //rect of the note highway board
        let board_rect = Rect { left: 0, top: 0, right: 96, bottom: 176};

        //rect of where the note strips will be positioned, not a bitmap RECT.
        //left and top represent where on the fretboard they will be drawn,
        let notespace_rect = Rect { left: 16.0, top: 8.0, right: 16.0 + 64.0, bottom: 8.0 + 160.0};
        //tallness of the fretboard in pixels (used for note positioning) (notespace_rect.height())
   
        //where the buttons should be drawn vertically
        let button_offset = notespace_rect.height() as f32 - 32.0; 


        //rect of the intermediate surface that will be drawn to the screen after everything has been drawn to it
        let board_rect_2: Rect<f32> = Rect { left: 0.0, top: 0.0, right: self.ref_size.0 * state.scale, bottom: self.ref_size.1 * state.scale };


        //rect for user buttons
        let button_inactive = Rect { left: 176, top: 0, right: 192, bottom: 16 };
        let button_active = Rect { left: 176, top: 16, right: 192, bottom: 32 };
        //rect for note parts
        let note_head = Rect { left: 176, top: 48, right: 192, bottom: 64 };
        let note_body = Rect { left: 176, top: 40, right: 192, bottom: 48 };
        let note_tail = Rect { left: 176, top: 32, right: 192, bottom: 48 };

        let nrg_bar_frame: Rect<u16> = Rect { left: 144, top: 0, right: 152, bottom: 64 };
        let nrg_bar_fuel: Rect<u16> = Rect { left: 152, top: 0, right: 160, bottom: 64 };
        let nrg_bar_max: Rect<u16> = Rect { left: 160, top: 0, right: 168, bottom: 64 };
        let nrg_bar_flash: Rect<u16> = Rect { left: 168, top: 0, right: 176, bottom: 64 };




        //push all shapes to the piano roll texture
        {
            //shift rects so we only have to define a few
            fn shift_right(orig_rect: &Rect<u16>, shift: usize) -> Rect<u16>
            {
                let mut new_rect: Rect<u16> = orig_rect.clone();
                let rect_width = new_rect.right - new_rect.left;
                new_rect.left += rect_width * shift as u16;
                new_rect.right = rect_width + new_rect.left;
                new_rect
            }

            //use the piano roll bitmap
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "PianoRoll")?;

            //set the render target to the texture
            graphics::set_render_target(ctx, self.texture.borrow().as_ref())?;
            //erase all old
            graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 0.0));


            //draw note highway
            batch.add_rect(0.0, 0.0, &board_rect);


            //set up offsets based on notezone rect
            let note_spacing = notespace_rect.width() / 4.0; //4 because 4 notes
            let note_offset = note_spacing / 2.0;


            //draw buttons
            {
                for i in 0..4
                {
                    //choose either the 'on' or 'off' button rect and shift it so the color matches
                    let button_rect = if self.key_state[i]
                    {shift_right(&button_active, i)}
                    else
                    {shift_right(&button_inactive, i)};
                    
                    let button_width = (button_rect.width() / 2) as f32;
                    batch.add_rect((notespace_rect.left + note_offset - button_width + note_spacing * i as f32) as f32, notespace_rect.top as f32 + button_offset, &button_rect); 
                }
            }

            //draw notes
            {

                //for all note strips (4 of them)
                for i in 0..4
                {
                    //color changing head
                    let note_h_rect: Rect<u16> = shift_right(&note_head, i);
                    //color changing body
                    let note_b_rect: Rect<u16> = shift_right(&note_body, i);
                    //color changing tail
                    let note_t_rect: Rect<u16> = shift_right(&note_tail, i);

                    let head_center = (note_h_rect.width() / 2) as f32;
                    let body_center = (note_b_rect.width() / 2) as f32;
                    let tail_center = (note_t_rect.width() / 2) as f32;
                   
                    //is notespace_rect.height(), but we also include the vertical offset so the note can despawn offscreen
                    let travel_px = notespace_rect.height() + note_h_rect.height() as f32;

                    //for all notes in the list
                    for j in 0..self.notes[i].len()
                    {
                        
                        //convert from percent-of-board length into pixels
                        let note_px_len = travel_px * self.notes[i][j].note_length_decimal;

                        //draw tail segments
                        {
                            //number of body segments that fit completely between the start and end of the note
                            let main_have = (note_px_len as i32) / (note_b_rect.bottom - note_b_rect.top) as i32;

                            for t in 0..(main_have)
                            {
                                let seg_x = notespace_rect.left + note_offset - body_center as f32 + note_spacing * i as f32;
                                let seg_y = (self.notes[i][j].note_head_loc * travel_px) - note_h_rect.height() as f32 + notespace_rect.top as f32 - (8 * t) as f32;

                                Self::crop_and_draw_rect(seg_x, seg_y, note_b_rect, notespace_rect, batch);
                            }
                        }

                        //cap with tail tip
                        let seg_x = notespace_rect.left + note_offset - tail_center as f32 + note_spacing * i as f32;
                        let seg_y = (self.notes[i][j].note_head_loc - self.notes[i][j].note_length_decimal) * travel_px - note_h_rect.height() as f32 + notespace_rect.top;
                        Self::crop_and_draw_rect(seg_x, seg_y, note_t_rect, notespace_rect, batch);

                        //draw head
                        let seg_x = notespace_rect.left + note_offset - head_center as f32 + note_spacing * i as f32;
                        let seg_y = self.notes[i][j].note_head_loc * travel_px - note_h_rect.height() as f32 + notespace_rect.top;
                        Self::crop_and_draw_rect(seg_x, seg_y, note_h_rect, notespace_rect, batch);
                        

                    }

                }
            }

            //draw NRG bar
            {
                let nrg_x = (board_rect.right - nrg_bar_frame.width()) as f32;
                let nrg_y = (board_rect.bottom - nrg_bar_frame.height() - 32) as f32;

                batch.add_rect(nrg_x, nrg_y, &nrg_bar_frame);
                batch.add_rect(nrg_x, nrg_y, &nrg_bar_fuel);
                //batch.add_rect(nrg_x, nrg_y, &nrg_bar_flash);
                batch.add_rect(nrg_x, nrg_y, &nrg_bar_max);

            }

            //blit all shapes to intermediate texture
            batch.draw(ctx)?;


            
            //draw points counter and HUD
            {
                draw_number(64.0, 16.0, 76 as usize, Alignment::Left, state, ctx)?;
            }


            //set target back to main surface
            graphics::set_render_target(ctx, None)?;

        }


        
        //draw texture onto the main screen
        if let Some(tex) = self.texture.borrow_mut().as_mut()
        {
            tex.clear();
            tex.add(SpriteBatchCommand::DrawRectSkewedTinted(
                board_rect_2, //src

                //top LR
                (64.0 * state.scale, 0.0),
                ((64.0 + 80.0) * state.scale, 0.0),

                //bottom LR
                (0.0, 144.0 * state.scale),
                ((64.0 + 80.0 + 64.0) * state.scale, (144.0) * state.scale),

                // (0.0 * state.scale, 0.0 * state.scale),
                // (self.ref_size.0 * state.scale, 0.0 * state.scale),
                // (0.0 * state.scale, self.ref_size.1 * state.scale),
                // (self.ref_size.0 * state.scale, self.ref_size.1 * state.scale),


                Color::from_rgb(0xFF, 0xFF, 0xFF),
            ));
            tex.draw()?;


        }

 
        Ok(())


    }


    ///////////////////
    /// Save/load functions
    ///////////////////


    //load guitar stats from the json into the stage table
    fn load(state: &mut SharedGameState, ctx: &Context) -> GameResult<GuitarScores> {

        if let Ok(file) = user_open(ctx, "/save_data.json") {
            match serde_json::from_reader::<_, GuitarScores>(file) {
                Ok(scores) => {
                    return Ok(scores);
                },
                Err(err) => {
                    log::warn!("Failed to deserialize settings: {}", err)
                },
            }
        }

        Ok(GuitarScores::new())
    }
    //save a vector of guitar stats to a json
    fn save(ctx: &Context, scores: &GuitarScores) -> GameResult
    {
        let file = user_create(ctx, "/save_data.json")?;
        serde_json::to_writer_pretty(file, &scores)?;

        Ok(())
    }


    //populates the stage table with the latest scores from the JSON
    pub fn get_saved_scores(state: &mut SharedGameState, ctx: &Context) -> GameResult
    {
        let fresh_data = Self::load(state, ctx).unwrap();
        for stage in state.stages.iter_mut()
        {
            for data in fresh_data.stages.iter()
            {
                if stage.name == data.name
                {
                    stage.score = data.score;
                }
            }
        }
        Ok(())
    }
    //saves the scores into the JSON
    pub fn put_saved_scores(state: &mut SharedGameState, ctx: &mut Context) -> GameResult
    {
        let mut prepped_data: Vec<LevelScore> = Vec::with_capacity(state.stages.len());
        
        for stage in state.stages.iter_mut()
        {
            prepped_data.push(LevelScore{name: stage.name.clone(), score: stage.score});
        }

        Self::save(ctx, &GuitarScores{stages: prepped_data})
    }




}


