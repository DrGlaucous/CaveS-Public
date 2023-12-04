use std::cell::RefCell;

use crate::common::{Color, Rect};
use crate::framework::backend::{BackendTexture, SpriteBatchCommand};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::game::scripting::tsc::text_script::TextScriptExecutionState;
use crate::game::shared_game_state::SharedGameState;
use crate::game::stage::Stage;
use crate::graphics::font::Font;
use crate::input::touch_controls::TouchControlType;
use crate::engine_constants::EngineConstants;

use crate::sound::SoundManager;

//#[derive(Eq, PartialEq)]



//notes used by the guitar struct
struct GuitarNote
{
    note_length: i32, //total length of the note (will we need this?)
    note_head_loc: i32, //where the front of the note is (for note hit timing)
    //note_tail_loc: i32, //where the end of the note is (for note release timing)

}



//renders and handles the guitar overlay, starting, and stopping
pub struct Guitar
{
    //everything is drawn to this before this is then drawn to the main screen
    texture: RefCell<Option<Box<dyn BackendTexture>>>,
    last_size: RefCell<(u16, u16)>,
    draw_corners: [[u32;2];4], //screen coordinates where the overlay should be placed
    current_song: usize,
    visible: bool,
    key_state: [bool; 4], //4 keys that can be pressed

    //used to determine if a new note is going to be played
    note_key: [u8; 4], //lengths of last notes created by the tracker
    note_len: [u8; 4], //key of the last note created by the tracker (note: pitch doesn't matter, only that it changed)
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
            draw_corners: [[0,0]; 4],
            current_song: 0,
            visible: false,
            key_state: [false; 4], //state of user input

            note_key: [0; 4],
            note_len: [0; 4],
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



    //look at bound buttons and change trapdoor state accordingly
    fn handle_buttons()
    {

    }

    //capture events from the tracker and assign proper behavior
    fn handle_tracker(&mut self, state: &mut SharedGameState)
    {
        //pull in fresh notes
        let notess = state.sound_manager.get_latest_track_state();

        //currently will only make a note if the note isn't playing right now, requires at least 1 tick between notes
        //checks tracks 1-4
        for i in 0..4
        {
            let this_note = notess.keys[i];
            let this_len = notess.lengths[i];

            if(this_note != self.note_key[i])// || this_len != self.note_len[i])
            {
                //create new note
                self.notes[i].push( GuitarNote{note_length: this_len as i32, note_head_loc: 0} );

                self.note_key[i] = this_note;
                self.note_len[i] = this_len;

            }
        }


    }

    //move the notes down their rows and checks for hits from the buttons
    fn handle_notes(&mut self, state: &mut SharedGameState, ctx: &mut Context)
    {
        let bottom_coord = 200 as i32;

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

                if n_strip[i].note_head_loc - n_strip[i].note_length > bottom_coord
                {
                    n_strip.remove(i);
                    continue;
                }
                //move note down
                n_strip[i].note_head_loc += 1;
                
            }
        }




    }

    //starts a song X with tracker pattern Y
    pub fn start_program(music: String, pattern: String)
    {

    }

    //pauses program execution
    pub fn pause_program()
    {

    }

    //resumes at the same place it left off
    pub fn resume_program()
    {

    }

    //get current scores and stats
    pub fn get_stats()
    {

    }


    //advance the ticker, call this as often as possible
    pub fn update(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult
    {

        //create the surface to draw to
        //the bitmap size depends on window scale, so this surface also needs to change in size with the window
        let width = (64.0 * state.scale) as u16;
        let height = (144.0 * state.scale) as u16;
        //re-create the surface when the window size changes
        if *self.last_size.borrow() != (width, height)
        {
            *self.last_size.borrow_mut() = (width, height);//.into(RefCell<(u16, u16)>);

            *self.texture.borrow_mut() = graphics::create_texture_mutable(ctx, width, height).ok();
        }

        //get the latest and greatest notes
        self.handle_tracker(state);

        //move existing notes down the chain
        self.handle_notes(state, ctx);






        Ok(())

    }

    //put note bar onto the screen
    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult
    {
        //if !self.visible {return Ok(())}


        let string_rect = Rect { left: 0, top: 0, right: 64, bottom: 144};

        let string_rect_2: Rect<f32> = Rect { left: 0.0, top: 0.0, right: 64.0 * state.scale, bottom: 144.0 * state.scale };

        let button_inactive = Rect { left: 64, top: 0, right: 80, bottom: 46 };
        let button_active = Rect { left: 64, top: 46, right: 80, bottom: 32 };

        let note_head = Rect { left: 64, top: 64, right: 80, bottom: 80 };
        let note_body = Rect { left: 64, top: 48, right: 80, bottom: 64 };
        let note_tail = Rect { left: 64, top: 32, right: 80, bottom: 48 };


        //push all shapes to the piano roll texture
        {
            //use the piano roll bitmap
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "PianoRoll")?;

            //set the render target to the texture
            graphics::set_render_target(ctx, self.texture.borrow().as_ref())?;
            //erase all old
            graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 0.0));


            //draw note highway
            batch.add_rect(0.0, 0.0, &string_rect);

            //draw notes
            {
                //for all note strips
                for i in 0..self.notes.len()
                {
                    let mut note_rect: Rect<u16> = note_head.clone();
                    let rect_width = note_rect.right - note_rect.left;

                    note_rect.left += rect_width * i as u16;
                    note_rect.right = rect_width + note_rect.left;

                    for j in 0..self.notes[i].len()
                    {
                        batch.add_rect((16 * i) as  f32, self.notes[i][j].note_head_loc as f32, &note_rect);  
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
                string_rect_2, //src

                //top LR
                (64.0 * state.scale, 0.0),
                ((64.0 + 80.0) * state.scale, 0.0),

                //bottom LR
                (0.0, 144.0 * state.scale),
                ((64.0 + 80.0 + 64.0) * state.scale, (144.0) * state.scale),

                Color::from_rgb(0xFF, 0xFF, 0xFF),
            ));
            tex.draw()?;


        }


        Ok(())

    }


}


