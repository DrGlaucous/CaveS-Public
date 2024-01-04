use std::cell::RefCell;
use std::ops::Index;
use std::time::SystemTime;
use std::time::Duration;
use std::cmp::Ordering;

use crate::bitfield;
use crate::common::{Color, Rect};
use crate::framework::backend::{BackendTexture, SpriteBatchCommand};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem::{user_create, user_open};
use crate::framework::gamepad::Axis;
use crate::framework::graphics;
use crate::game::scripting::tsc::text_script::TextScriptExecutionState;
use crate::game::shared_game_state::SharedGameState;
use crate::game::graphics::screen_insets_scaled;
use crate::game::stage::Stage;
use crate::graphics::font::Font;
use crate::graphics::texture_set::SpriteBatch;
use crate::input::touch_controls::TouchControlType;
use crate::engine_constants::EngineConstants;

use crate::components::draw_common::draw_number_int;
use crate::components::draw_common::Alignment;

use crate::input::dummy_player_controller::DummyPlayerController;
use crate::input::player_controller::PlayerController;

use crate::sound::SoundManager;

use super::shared_game_state::TimingMode;


use log::Level;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Relativity
{
    ZeroWall, //left, top
    CenterPoint, //center
    OutsideWall, //right, bottom
}

pub enum CornerIndex
{
    TopLeft = 0,
    TopRight,
    BottomLeft,
    BottomRight,
}


//holds scores for each level
#[derive(Serialize, Deserialize, Debug)]
pub struct LevelScore {
    pub correct_notes: u32,
    pub incorrect_notes: u32, //NOT "notes missed", also counts wrongful presses
    pub total_notes: u32, //correct notes + notes missed (total note count)
    pub longest_streak: u32,
    pub last_streak: u32,
    pub score: i32,
}
impl LevelScore
{
    pub fn new() -> Self {
        LevelScore {
            correct_notes: 0,
            incorrect_notes: 0,
            total_notes: 0,
            score: 0,
            longest_streak: 0,
            last_streak: 0,
        }
    }
    pub fn clone(&self) -> Self {
        LevelScore {
            correct_notes: self.correct_notes,
            incorrect_notes: self.incorrect_notes,
            total_notes: self.total_notes,
            score: self.score,
            longest_streak: self.longest_streak,
            last_streak: self.last_streak,
        }
    }

    pub fn accuracy(&self) -> f32
    {
        self.correct_notes as  f32 / (self.incorrect_notes + self.correct_notes) as f32 * 100.0
    }




}


//wraps LevelScore with a name, is vectored in GuitarScores
#[derive(Serialize, Deserialize, Debug)]
pub struct LevelScoreWrapper
{
    name: String,
    score: LevelScore,
}
impl LevelScoreWrapper
{
    pub fn new() ->LevelScoreWrapper {
        LevelScoreWrapper {
            name: "".to_owned(),
            score: LevelScore::new(),
        }
    }
}


//holds the settings to be passed off to the json
#[derive(Serialize, Deserialize, Debug)]
pub struct GuitarScores {
    stages: Vec<LevelScoreWrapper>,
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
    out_of_range: bool, //true if the note cannot be hit anymore

}


//used to sync events with hits
struct DrumEvent
{
    //where the front of the note is (for note hit timing)
    //is a decimal from 0 to 1, where 1 is at the bottom and 0 is at the top
    note_head_loc: f32,

    last_time: SystemTime, //the last time the location of the note was updated

    event_no: u8,
}


//infor for each axis in a corner
#[derive(Clone, Copy)]
struct AxisCornerInfo {
    draw_coord: f32, //screen coordinates where the overlay should be placed (relative to anchor)
    last_draw_coord: f32, //where it was when the new command was recieved
    next_draw_coord: f32, //where we want it to end up
    
    corner_anchor: Relativity, //what the coordinates are realtivive to
    last_corner_anchor: Relativity,
    corner_travel_time: u32, //how many ticks until the corner reaches its spot
}
impl AxisCornerInfo {
    pub fn new() -> AxisCornerInfo
    {
        AxisCornerInfo {
            draw_coord: 0.0,
            last_draw_coord: 0.0,
            next_draw_coord: 0.0,
            corner_anchor: Relativity::ZeroWall,
            last_corner_anchor: Relativity::ZeroWall,
            corner_travel_time: 0,
        }
    }
}

//info for each corner's movement
#[derive(Clone, Copy)]
struct CornerInfo
{
    //x and y
    point: [AxisCornerInfo; 2],
}

impl CornerInfo
{
    pub fn new() -> CornerInfo
    {
        CornerInfo {
            point: [AxisCornerInfo::new();2],
        }
    }
}

//renders and handles the guitar overlay, starting, and stopping
pub struct Guitar
{
    //everything is drawn to this before this is then drawn to the main screen
    texture: RefCell<Option<Box<dyn BackendTexture>>>,
    last_size: RefCell<(u16, u16)>, //used to see if the window has been resized and the texure should also be resized
    ref_size: (f32, f32), //pixel size of this texture, set here so it can be used everywhere

    onscreen_time: f32, //how many seconds the note should last on the highway

    current_score: LevelScore,
    pub controller: Box<dyn PlayerController>, //controls the reader (duh)

    //using arrays instead of bitfields because of (easier) iteration
    hit_state: [bool;4],
    hit_trigger: [bool;4],

    //holds position and state of all active notes
    notes: [Vec<GuitarNote>; 4],

    events: Vec<DrumEvent>,

    /////////////////////////////////////
    //settings to be configured:
    /////////////////////////////////////
    
    draw_corners: [CornerInfo; 4],

    visible: bool,

    //how far down the notes should be on the note highway
    //ranges from 0 to 1
    button_offset: f32,
    //the error between note start
    hit_leniency: f32,

    //for each multiplier level, this many notes need to be hit consecutively
    streak_per_level: usize,




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
            draw_corners: [CornerInfo::new(); 4],
            visible: true,
            hit_state: [false;4],
            hit_trigger: [false;4],
            onscreen_time: 1.472,//2.994, //2.25,
            current_score: LevelScore::new(),
            controller: Box::new(DummyPlayerController::new()),
            notes: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            events: Vec::new(),
            button_offset: 0.75,
            hit_leniency: 0.1,

            streak_per_level: 5,


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



        self.hit_state[0] = self.controller.one();
        self.hit_state[1] = self.controller.two();
        self.hit_state[2] = self.controller.three();
        self.hit_state[3] = self.controller.four();

        self.hit_trigger[0] = self.controller.trigger_one();
        self.hit_trigger[1] = self.controller.trigger_two();
        self.hit_trigger[2] = self.controller.trigger_three();
        self.hit_trigger[3] = self.controller.trigger_four();


        return Ok(());
    }

    //convert a change in time to a percent-of-track completion
    fn get_offset(onscreen_time: f32, time_now: SystemTime, time_then: SystemTime) -> f32
    {
        //protect against backwards times (this is a glitch with the global stopwatch)
        if time_then > time_now
        {return 0.0}

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
            self.notes[i].push( GuitarNote{note_length_decimal: length_decimal,
                note_length: notess.lengths[i],
                note_head_loc: change_decimal,
                last_time: this_time,
                was_hit: false,
                out_of_range: false,
                } );


            self.current_score.total_notes += 1;
        }

        //push back any events to be run
        for i in 0..8
        {
            let change_decimal = Self::get_offset(self.onscreen_time, this_time, state.sound_manager.music_time.from_systime(notess.timestamp));
            if notess.drums[i] != 0xFF && notess.drums[i] != 0x00
            {
                self.events.push(DrumEvent{
                    note_head_loc: change_decimal,
                    last_time: this_time,
                    event_no: notess.drums[i],
                });
            }
        }
        

        //run events with drums
        //moved to 'handle_notes'
        // for i in 0..8
        // {
        //     if notess.drums[i] != 0xFF && notess.drums[i] != 0x00
        //     {
        //         //let mut yyt = i;
        //         //yyt += i;
        //         //print!("{}\n", notess.drums[i]);
        //         state.control_flags.set_tick_world(true);
        //         state.control_flags.set_interactions_disabled(true);
        //         //state.textscript_vm.executor_player = id;
        //         state.textscript_vm.start_script(notess.drums[i] as u16);
        //     }
        // }


    }

    //move events down and check for when they cross the hit position
    fn handle_events(&mut self, state: &mut SharedGameState)
    {
        for n in (0..self.events.len()).rev()
        {
            //move event down
            let present = state.sound_manager.music_time.now();
            let down_movement = Self::get_offset(self.onscreen_time, present, self.events[n].last_time);
            self.events[n].note_head_loc += down_movement;
            self.events[n].last_time = present;

            if self.button_offset <= self.events[n].note_head_loc {
                //equivalent of <EVE

                state.control_flags.set_tick_world(true);
                state.control_flags.set_interactions_disabled(true);
                //state.textscript_vm.executor_player = id;
                state.textscript_vm.start_script(self.events[n].event_no as u16);

                //remove note
                self.events.remove(n);
            }

        }
    }

    //move the notes down their rows and checks for hits from the buttons
    fn handle_notes(&mut self, state: &mut SharedGameState, ctx: &mut Context)
    {

        let mut hit_minus = 0;
        let mut hit_plus = 0;

        //for each note strip, move notes down
        //n is 0-3
        for n in 0..self.notes.len()
        {
            let n_strip = &mut self.notes[n];


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


                //note out of range, disqualify it for hitting and penalize
                if self.button_offset < n_strip[i].note_head_loc - n_strip[i].note_length_decimal - self.hit_leniency
                && !n_strip[i].was_hit
                && !n_strip[i].out_of_range
                {
                    n_strip[i].out_of_range = true;
                    hit_minus += 1;
                    continue;
                }

            }

            //iterate again, but forwards, so the furthest notes down get press-checked first
            for i in 0..n_strip.len() //(n_strip.len() - 1)..=0
            {
                if n_strip[i].out_of_range
                {
                    continue;
                }

                //check for intersection with button
                if self.hit_trigger[n]
                && self.button_offset < n_strip[i].note_head_loc + self.hit_leniency
                && self.button_offset > n_strip[i].note_head_loc - n_strip[i].note_length_decimal - self.hit_leniency
                {
                    if n_strip[i].was_hit != true
                    {
                        n_strip[i].was_hit = true;
                        hit_plus += 1;    
                    }
                    else {
                        //was hit, ignore this one
                        continue;
                    }
                    //breakout of for loop, each button can only press a single note at a time
                    break;
                }


            }

            
            //check for missing a note
            if self.hit_trigger[n]
            {
                //pressed and hit nothing
                if hit_plus < 1
                {
                    hit_minus += 1;
                }
            }

        }

        //reset streak
        if hit_minus > 0 
        {
            self.current_score.last_streak = 0;
        }
        else
        {
            self.current_score.last_streak += hit_plus;
        }

        //apply accuracies to score
        self.current_score.score += 50 * (hit_plus as i32 - hit_minus as i32) * (1 + self.current_score.last_streak / self.streak_per_level as u32) as i32 + 10 * hit_plus as i32;

        //update totals (TODO: fix missed_notes)
        self.current_score.correct_notes += hit_plus;
        self.current_score.incorrect_notes += hit_minus; //includes missed notes *and* missed strikes

        //update top streak
        if self.current_score.last_streak > self.current_score.longest_streak
        {
            self.current_score.longest_streak = self.current_score.last_streak;
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

    //shift rects so we only have to define a few
    fn shift_right(orig_rect: &Rect<u16>, shift: usize) -> Rect<u16>
    {
        let mut new_rect: Rect<u16> = orig_rect.clone();
        let rect_width = new_rect.right - new_rect.left;
        new_rect.left += rect_width * shift as u16;
        new_rect.right = rect_width + new_rect.left;
        new_rect
    }

    fn handle_corners(&mut self, state: &SharedGameState)
    {
        //for each corner
        for corner in self.draw_corners.iter_mut()
        {
            for (i, axis) in corner.point.iter_mut().enumerate()
            {
                
                //changed refrence points, update the last refrence to match this one
                if axis.corner_anchor != axis.last_corner_anchor
                {
                    let canvas_size = if i == 0 {state.canvas_size.0} else {state.canvas_size.1};


                    //start by making the last coordinate relative to the left/top wall
                    match axis.last_corner_anchor
                    {
                        Relativity::ZeroWall => {} //already realtive, do nothing

                        Relativity::CenterPoint => {
                            axis.last_draw_coord += canvas_size / 2.0;
                        }
                        Relativity::OutsideWall => {
                            axis.last_draw_coord += canvas_size;
                        }
                    }
                    //now make it relative to the current anchor
                    match axis.corner_anchor
                    {
                        Relativity::ZeroWall => {} //already realtive, do nothing

                        Relativity::CenterPoint => {
                            axis.last_draw_coord -= canvas_size / 2.0;
                        }
                        Relativity::OutsideWall => {
                            axis.last_draw_coord -= canvas_size;
                            //axis.last_draw_coord = canvas_size - axis.last_draw_coord;
                        }
                    }
                    //since this only happens when we change coordinates, we can assume these are equal
                    axis.draw_coord = axis.last_draw_coord;

                    //we are now on the same anchor
                    axis.last_corner_anchor = axis.corner_anchor;

                }

                let step_x = (axis.next_draw_coord - axis.last_draw_coord) / axis.corner_travel_time as f32;
                
                //overshot checking
                if step_x < 0.0 {
                    
                    if axis.draw_coord <= axis.next_draw_coord// - (axis.next_draw_coord * 0.1) //+- 10%
                    {
                        axis.draw_coord = axis.next_draw_coord;
                    }
                }
                else {
                    if axis.draw_coord >= axis.next_draw_coord// + (axis.next_draw_coord * 0.1) //+- 10%
                    {
                        axis.draw_coord = axis.next_draw_coord;
                    }
                }

                //move to
                if axis.draw_coord != axis.next_draw_coord
                {
                    axis.draw_coord += step_x;
                }
        
            }
        
        
        }
    }

    ///////////////////
    /// Control functions
    ///////////////////

    //starts a song X with tracker pattern Y (currently unused)
    // pub fn start_program(music: String, pattern: String)
    // {
    // }

    //feed this the time between when a note spawns and when it will cross the button row
    pub fn set_time_to_intersect(&mut self, seconds: f32)
    {
        //self.button_offset * self.onscreen_time = seconds;
        self.onscreen_time = seconds / self.button_offset;
    }

    pub fn set_hit_window(&mut self, percent: f32)
    {
        self.hit_leniency = percent;
    }


    //tell each player to delay so that they are synchronized
    //feed it milliseconds until the song+tracker makes the first note
    pub fn set_start_delay(&self, state: &mut SharedGameState, millis_song: u32, millis_tracker: u32, extra_millis: u32)
    {
        //factors in how long it takes to reach the buttons from playing a note
        let needed_tracker_time = millis_tracker as f32 + (self.button_offset * self.onscreen_time) * 1000.0;

        let time_difference = (millis_song as f32 - needed_tracker_time).abs();

        //if the song needs to start first
        let (dur_for_song, dur_for_track) = if needed_tracker_time < millis_song as f32
        {
            (
                Duration::from_millis(extra_millis as u64),
                Duration::from_millis(extra_millis as u64 + time_difference as u64),
            )
        }
        //if the tracker needs to start first
        else
        {
            (
                Duration::from_millis(extra_millis as u64 + time_difference as u64),
                Duration::from_millis(extra_millis as u64),
            )
        };

        //tell the players to halt for this ammount of time
        state.sound_manager.freeze_song_for(dur_for_song);
        state.sound_manager.freeze_tracker_for(dur_for_track);


    }

    //show or hide the tracker bar
    pub fn set_visibility(&mut self, state: bool)
    {
        self.visible = state;
    }
    pub fn get_visibility(&self) -> bool
    {
        self.visible
    }

    //saves the current stats to the map designated by stage number
    pub fn store_stats(&mut self, state: &mut SharedGameState, stage_no: usize)
    {
        if stage_no >= state.stages.len()
        {
            return;
        }
        state.stages[stage_no].score = self.current_score.clone();

    }

    //resets stats to default
    pub fn reset_stats(&mut self)
    {
        self.current_score = LevelScore::new();
    }


    pub fn get_current_score(&mut self) -> LevelScore
    {
        self.current_score.clone()
    }

    //set a corner to move to a spot
    pub fn set_corner(&mut self, axis: usize, index: CornerIndex, anchor: Relativity, offset: i32, time: u32)
    {

        

        let idx = match index
        {
            CornerIndex::TopLeft => {0},
            CornerIndex::TopRight => {1},
            CornerIndex::BottomLeft => {2},
            CornerIndex::BottomRight => {3},
        };

        //handle OOB
        let axis = if axis >= self.draw_corners.len() {
            self.draw_corners.len() - 1
        }
        else {axis};

        //update olds
        self.draw_corners[idx].point[axis].last_corner_anchor = self.draw_corners[idx].point[axis].corner_anchor;
        self.draw_corners[idx].point[axis].last_draw_coord = self.draw_corners[idx].point[axis].draw_coord;

        //set news
        self.draw_corners[idx].point[axis].corner_anchor = anchor;
        self.draw_corners[idx].point[axis].next_draw_coord = offset as f32;
        self.draw_corners[idx].point[axis].corner_travel_time = if time < 1 {1} else {time};



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

        //do not process if it can't be seen
        if !self.visible
        {
            return Ok(())
        }


        //get latest input state
        self.handle_buttons(state, ctx)?;

        //get the latest and greatest notes
        self.handle_tracker(state);

        //move existing notes down the chain
        self.handle_notes(state, ctx);

        //handle event timing
        self.handle_events(state);

        //move edges to new locations
        self.handle_corners(state);



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
           
        //where the buttons should be drawn vertically
        //let button_offset = notespace_rect.height() as f32 - 32.0; 
        let button_px_offset = (notespace_rect.height() as f32) * self.button_offset; 

        //notespace rect but cut off at the buttons, so 'hit' notes will not be drawn past there
        let mut hit_notespace_rect = notespace_rect.clone();
        hit_notespace_rect.bottom = hit_notespace_rect.top + button_px_offset;



        //rect of the intermediate surface that will be drawn to the screen after everything has been drawn to it
        let board_rect_2: Rect<f32> = Rect { left: 0.0, top: 0.0, right: self.ref_size.0 * state.scale, bottom: self.ref_size.1 * state.scale };


        //rect for user buttons
        let button_inactive = Rect { left: 176, top: 0, right: 192, bottom: 16 };
        let button_active = Rect { left: 176, top: 16, right: 192, bottom: 32 };
        //rect for note parts
        let note_head = Rect { left: 176, top: 48, right: 192, bottom: 64 };
        let note_body = Rect { left: 176, top: 40, right: 192, bottom: 48 };
        let note_tail = Rect { left: 176, top: 32, right: 192, bottom: 48 };

        let note_dead_head = Rect { left: 160, top: 80, right: 176, bottom: 96 };
        let note_dead_body = Rect { left: 160, top: 72, right: 176, bottom: 80 };
        let note_dead_tail = Rect { left: 160, top: 64, right: 176, bottom: 80 };

        let nrg_bar_frame: Rect<u16> = Rect { left: 144, top: 0, right: 152, bottom: 64 };
        let nrg_bar_fuel: Rect<u16> = Rect { left: 152, top: 0, right: 160, bottom: 64 };
        let nrg_bar_max: Rect<u16> = Rect { left: 160, top: 0, right: 168, bottom: 64 };
        let nrg_bar_flash: Rect<u16> = Rect { left: 168, top: 0, right: 176, bottom: 64 };


        //rects for HUD
        let points_rc = Rect { left: 128, top: 64, right: 160, bottom: 72 };
        let total_h_rc = Rect  { left: 128, top: 96, right: 176, bottom: 104 };
        //rect of '1', but can be shifted over for other numbers
        let mult_num_rc = Rect  { left: 128, top: 72, right: 136, bottom: 80 };
        let rect_x = Rect  { left: 128, top: 80, right: 136, bottom: 88 };

        let fill_bar_frame = Rect  { left: 176, top: 96, right: 240, bottom: 104 };
        let fill_bar_back = Rect  { left: 176, top: 104, right: 240, bottom: 112 };
        let fill_bar_filling = Rect  { left: 176, top: 112, right: 240, bottom: 120 };


        //push all shapes to the piano roll texture
        {
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
                    let button_rect = if self.hit_state[i]
                    {Self::shift_right(&button_active, i)}
                    else
                    {Self::shift_right(&button_inactive, i)};
                    
                    let button_width = (button_rect.width() / 2) as f32;
                    batch.add_rect((notespace_rect.left + note_offset - button_width + note_spacing * i as f32) as f32, notespace_rect.top as f32 + button_px_offset, &button_rect); 
                }
            }

            //draw notes
            {

                //for all note strips (4 of them)
                for i in 0..4
                {
                    //let (n_head, n_body, n_tail) = if 

                    //color changing head
                    let note_h_rect: Rect<u16> = Self::shift_right(&note_head, i);
                    //color changing body
                    let note_b_rect: Rect<u16> = Self::shift_right(&note_body, i);
                    //color changing tail
                    let note_t_rect: Rect<u16> = Self::shift_right(&note_tail, i);

                    let head_center = (note_h_rect.width() / 2) as f32;
                    let body_center = (note_b_rect.width() / 2) as f32;
                    let tail_center = (note_t_rect.width() / 2) as f32;
                   
                    //is notespace_rect.height(), but we also include the vertical offset so the note can despawn offscreen
                    let travel_px = notespace_rect.height() + note_h_rect.height() as f32;


                    //for all notes in the list
                    for j in 0..self.notes[i].len()
                    {
                        //check for dead notes and change the RECT again if needed
                        let (note_h_rect, note_b_rect, note_t_rect) = if self.notes[i][j].out_of_range
                        { (note_dead_head, note_dead_body, note_dead_tail) } else {(note_h_rect, note_b_rect, note_t_rect)} ;
                        

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

                                Self::crop_and_draw_rect(seg_x, seg_y, note_b_rect, 
                                    if self.notes[i][j].was_hit {hit_notespace_rect} else {notespace_rect},
                                    batch);
                            }
                        }

                        //cap with tail tip
                        let seg_x = notespace_rect.left + note_offset - tail_center as f32 + note_spacing * i as f32;
                        let seg_y = (self.notes[i][j].note_head_loc - self.notes[i][j].note_length_decimal) * travel_px - note_h_rect.height() as f32 + notespace_rect.top;
                        Self::crop_and_draw_rect(seg_x, seg_y, note_t_rect, 
                            if self.notes[i][j].was_hit {hit_notespace_rect} else {notespace_rect},
                            batch);

                        //draw head
                        let seg_x = notespace_rect.left + note_offset - head_center as f32 + note_spacing * i as f32;
                        let seg_y = self.notes[i][j].note_head_loc * travel_px - note_h_rect.height() as f32 + notespace_rect.top;
                        Self::crop_and_draw_rect(seg_x, seg_y, note_h_rect,
                            if self.notes[i][j].was_hit {hit_notespace_rect} else {notespace_rect},
                            batch);
                        

                    }

                }
            }

            //draw NRG bar (vetoed for now)
            // {
            //     let nrg_x = (board_rect.right - nrg_bar_frame.width()) as f32;
            //     let nrg_y = (board_rect.bottom - nrg_bar_frame.height() - 32) as f32;

            //     batch.add_rect(nrg_x, nrg_y, &nrg_bar_frame);
            //     batch.add_rect(nrg_x, nrg_y, &nrg_bar_fuel);
            //     //batch.add_rect(nrg_x, nrg_y, &nrg_bar_flash);
            //     batch.add_rect(nrg_x, nrg_y, &nrg_bar_max);

            // }

            //blit all shapes to intermediate texture
            batch.draw(ctx)?;


            //set target back to main surface
            graphics::set_render_target(ctx, None)?;


        }


        
        //draw texture onto the main screen
        if let Some(tex) = self.texture.borrow_mut().as_mut()
        {
            let mut final_corners = [(0.0, 0.0);4];

            for (i, corner) in self.draw_corners.iter().enumerate()
            {
                //have to do this explicitly; we're working with tuples
                final_corners[i].0 = match corner.point[0].corner_anchor
                {
                    Relativity::ZeroWall => corner.point[0].draw_coord,
                    Relativity::CenterPoint => corner.point[0].draw_coord + state.canvas_size.0/2.0, 
                    Relativity::OutsideWall => corner.point[0].draw_coord + state.canvas_size.0, 
                } * state.scale;
                final_corners[i].1 = match corner.point[1].corner_anchor
                {
                    Relativity::ZeroWall => corner.point[1].draw_coord,
                    Relativity::CenterPoint => corner.point[1].draw_coord + state.canvas_size.1/2.0, 
                    Relativity::OutsideWall => corner.point[1].draw_coord + state.canvas_size.1, 
                } * state.scale;

            }

            tex.clear();
            tex.add(SpriteBatchCommand::DrawRectSkewedTinted(
                board_rect_2, //src
                final_corners[0], final_corners[1],
                final_corners[2], final_corners[3],
                //top LR
                //(64.0 * state.scale, 0.0),
                //((64.0 + 80.0) * state.scale, 0.0),

                //bottom LR
                //(0.0, 144.0 * state.scale),
                //((64.0 + 80.0 + 64.0) * state.scale, (144.0) * state.scale),

                // (0.0 * state.scale, 0.0 * state.scale),
                // (self.ref_size.0 * state.scale, 0.0 * state.scale),
                // (0.0 * state.scale, self.ref_size.1 * state.scale),
                // (self.ref_size.0 * state.scale, self.ref_size.1 * state.scale),


                Color::from_rgb(0xFF, 0xFF, 0xFF),
            ));
            tex.draw()?;


        }

        //draw points counter and HUD (see hud.rs for a good example)
        {

            //things to draw:
            //points
            //score multiplier + level of multiplier
            //total gems hit

            //edge insets (used in HUD, what does it do?)
            let (left, top, right, bottom) = screen_insets_scaled(ctx, state.scale);

            //coordniates for drawing
            let x = 16.0 + left;
            let y = 16.0 + top;

            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "PianoRoll")?;



            batch.add_rect(x, y, &points_rc);
            batch.add_rect(x, y + 8.0, &total_h_rc);


            //draw fill bar
            {

                let shift_count = if self.current_score.last_streak as usize / self.streak_per_level > 3
                {3} else {self.current_score.last_streak as usize / self.streak_per_level };
                
                //get rect ammount for how full the bar should be
                let mut filling_amm = fill_bar_filling.clone();
                let chunk = 
                if shift_count == 3 {fill_bar_filling.width()}
                else if (self.current_score.last_streak % self.streak_per_level as u32) == 0
                {0}
                else { fill_bar_filling.width() / self.streak_per_level as u16 * (self.current_score.last_streak % self.streak_per_level as u32) as u16 };

                filling_amm.right = filling_amm.left +  chunk;


                //get color
                let tint_color = match shift_count
                {
                    0 =>{Color::from_rgb(0x00, 0xFF, 0xFF)}
                    1 =>{Color::from_rgb(0x00, 0xFF, 0x00)}
                    2 =>{Color::from_rgb(0xFF, 182, 0x00)}
                    3 =>{Color::from_rgb(234, 25, 62)}
                    _ =>{Color::from_rgb(0xFF, 0xFF, 0xFF)}
                };

                //multiplier bar
                batch.add_rect(x, y + 16.0, &fill_bar_back);
                batch.add_rect_tinted(x, y + 16.0, tint_color.to_rgba(), &fill_bar_back);
                batch.add_rect_tinted(x, y + 16.0, tint_color.to_rgba(), &filling_amm);

                let multiplier_rc = Self::shift_right(&mult_num_rc, shift_count );
                batch.add_rect(x + 32.0, y + 16.0, &multiplier_rc);
                batch.add_rect(x + 16.0, y + 16.0, &rect_x);


            }



            //draw to screen
            batch.draw(ctx)?;

            //points
            draw_number_int(x + total_h_rc.width() as f32, y, self.current_score.score as i32, Alignment::Left, state, ctx)?;
            
            //hits
            draw_number_int(x + total_h_rc.width() as f32, y + 8.0, self.current_score.correct_notes as i32, Alignment::Left, state, ctx)?;

        }
 
        Ok(())


    }


    ///////////////////
    /// Save/load functions
    ///////////////////


    //load guitar stats from the json into the stage table
    fn load(ctx: &Context) -> GameResult<GuitarScores> {

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
        let fresh_data = Self::load(ctx).unwrap();
        for stage in state.stages.iter_mut()
        {
            for data in fresh_data.stages.iter()
            {
                if stage.name == data.name
                {
                    stage.score = data.score.clone();
                }
            }
        }
        Ok(())
    }
    //saves the scores into the JSON
    pub fn put_saved_scores(state: &mut SharedGameState, ctx: &mut Context) -> GameResult
    {
        let mut prepped_data: Vec<LevelScoreWrapper> = Vec::with_capacity(state.stages.len());
        
        for stage in state.stages.iter_mut()
        {
            prepped_data.push(LevelScoreWrapper{name: stage.name.clone(), score: stage.score.clone()});
        }

        Self::save(ctx, &GuitarScores{stages: prepped_data})
    }




}


