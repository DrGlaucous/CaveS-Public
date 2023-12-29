//use crate::common::{Color, VERSION_BANNER};
//use crate::components::background::Background;
//use crate::components::compact_jukebox::CompactJukebox;
//use crate::components::nikumaru::NikumaruCounter;
//use crate::entity::GameEntity;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
//use crate::game::frame::Frame;
//use crate::game::map::Map;
use crate::game::shared_game_state::SharedGameState;
//use crate::game::shared_game_state::{
//    GameDifficulty, MenuCharacter, ReplayKind, ReplayState, Season, SharedGameState, TileSize, self,
//};
//use crate::game::stage::{BackgroundType, NpcType, Stage, StageData, StageTexturePaths, Tileset};
//use crate::graphics::font::Font;
use crate::input::combined_menu_controller::CombinedMenuController;
//use crate::input::touch_controls::TouchControlType;
//use crate::menu::coop_menu::PlayerCountMenu;
//use crate::menu::save_select_menu::SaveSelectMenu;
//use crate::menu::settings_menu::SettingsMenu;
use crate::menu::{Menu, MenuEntry, MenuSelectionResult};
//use crate::scene::jukebox_scene::JukeboxScene;
//use crate::scene::Scene;


//use crate::scene::game_scene::GameScene;
use crate::game::guitar::{Guitar, LevelScore};
use crate::scene::title_scene::ConfirmMenuEntry;
//use crate::menu::save_select_menu::MenuSaveInfo;





//holds settings relating to the main list
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MenuItemsStageSel
{
    Banner,
    Back,
    Level(usize),
}
impl Default for MenuItemsStageSel {
    fn default() -> Self {
        MenuItemsStageSel::Level(0)
    }
}

//main menu
pub struct StageSelectMenu {
    list_menu: Menu<MenuItemsStageSel>,
}

impl StageSelectMenu {

    //constructor
    pub fn new() -> StageSelectMenu
    {
        StageSelectMenu {
            list_menu: Menu::new(0, 0, 230, 0),
        }
    }

    //called whenever this menu is switched to
    pub fn init(&mut self, state: &mut SharedGameState, ctx: &Context) -> GameResult 
    {
        
        self.list_menu = Menu::new(0, 0, 230, 0);


        //refresh scores
        Guitar::get_saved_scores(state, ctx)?;

        //make title
        self.list_menu.push_entry(MenuItemsStageSel::Banner, MenuEntry::Disabled(String::from("Stages")) );

        //make back button
        self.list_menu.push_entry(MenuItemsStageSel::Back, MenuEntry::Active(state.loc.t("common.back").to_owned()));


        //push all maps in the stage table to the menu
        let mut i: usize = 0;
        for map in state.stages.iter()
        {
            self.list_menu.push_entry(MenuItemsStageSel::Level(i), MenuEntry::Active(map.name.clone() + " TODO: Score here") );
            self.list_menu.selected = MenuItemsStageSel::Level(i);
            i += 1;
        }

        self.update_sizes(state);

        Ok(())
    }

    //handle window scaling
    fn update_sizes(&mut self, state: &SharedGameState)
    {
        self.list_menu.update_width(state);
        self.list_menu.update_height(state);
        self.list_menu.x = ((state.canvas_size.0 - self.list_menu.width as f32) / 2.0).floor() as isize;
        self.list_menu.y = ((state.canvas_size.1 - self.list_menu.height as f32) / 2.0).floor() as isize;
    }

    //process user input and behavior
    pub fn tick(
        &mut self,
        exit_action: &mut dyn FnMut(),
        controller: &mut CombinedMenuController,
        state: &mut SharedGameState,
        ctx: &mut Context,
    ) -> GameResult {

        self.update_sizes(state);

        //run cases depending on user action
        match self.list_menu.tick(controller, state)
        {
            MenuSelectionResult::Canceled => exit_action(),
            MenuSelectionResult::Selected(sub_item, _) =>
            {
                match sub_item
                {
                    MenuItemsStageSel::Banner => {}
                    MenuItemsStageSel::Back => {exit_action()}
                    MenuItemsStageSel::Level(level) =>{
                        //load level based on selection with predefined event
                        state.reload_resources(ctx)?;
                        state.start_new_game_at(ctx, level, 200, (0,0))?;
                    }
                }
            },
            //idler case
            _ => (),
        }

        Ok(())
    }

    //look pretty
    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult 
    {
        self.list_menu.draw(state, ctx)?;

        Ok(())
    }

}




//Turns out I don't need anything below. I can do it better by adding the behavior to the TSC engine in a simmilar manner to a YNJ
//well, that was time down the hole...
/* 
//save confimration menu
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SaveConfirmationItems
{
    Banner,
    Yes,
    No,
}
//default value when variable is initialzied
impl Default for SaveConfirmationItems {
    fn default() -> Self {SaveConfirmationItems::Banner}
}

pub struct SaveConfirmationMenu {
    //the internal menu items, using the enums defined above to define menu objects
    sc_menu: Menu<SaveConfirmationItems>,
    
}
impl SaveConfirmationMenu
{
    //constructor
    pub fn new() -> SaveConfirmationMenu
    {
        SaveConfirmationMenu {
            sc_menu: Menu::new(0, 0, 160, 0),
        }
    }

    //called whenever this menu is switched to
    pub fn init(&mut self, state: &mut SharedGameState, ctx: &Context) -> GameResult 
    {

        //recreate the menu on init (why do we do this in new() as well?)
        self.sc_menu = Menu::new(0, 0, 160, 0);

        //make title
        self.sc_menu.push_entry(SaveConfirmationItems::Banner, MenuEntry::Disabled(String::from("Save?")) );

        //make yes button
        self.sc_menu.push_entry(SaveConfirmationItems::Yes, MenuEntry::Active(state.loc.t("common.yes").to_owned()));

        //make no button
        self.sc_menu.push_entry(SaveConfirmationItems::No, MenuEntry::Active(state.loc.t("common.no").to_owned()));

        self.update_sizes(state);

        Ok(())
    }
    
    fn update_sizes(&mut self, state: &SharedGameState) {
        self.sc_menu.update_width(state);
        self.sc_menu.update_height(state);
        self.sc_menu.x = ((state.canvas_size.0 - self.sc_menu.width as f32) / 2.0).floor() as isize;
        self.sc_menu.y = ((state.canvas_size.1 - self.sc_menu.height as f32) / 2.0).floor() as isize;

        self.sc_menu.update_width(state);
        self.sc_menu.update_height(state);
        self.sc_menu.x = ((state.canvas_size.0 - self.sc_menu.width as f32) / 2.0).floor() as isize;
        self.sc_menu.y = ((state.canvas_size.1 - self.sc_menu.height as f32) / 2.0).floor() as isize;
    }

    //process user input and behavior
    pub fn tick(
        &mut self,
        //exit_action: &mut dyn FnMut(),
        controller: &mut CombinedMenuController,
        state: &mut SharedGameState,
        ctx: &mut Context,
        guitar_manager: &mut Guitar,
    ) -> GameResult {

        self.update_sizes(state);

        //run cases depending on user action, determined by the controller fed into the function
        match self.sc_menu.tick(controller, state)
        {
            MenuSelectionResult::Canceled => {}, //exit_action(),
            MenuSelectionResult::Selected(sub_item, _) =>
            {
                match sub_item
                {
                    SaveConfirmationItems::Banner => {},
                    SaveConfirmationItems::Yes => {
                        //jump TSC
                    },
                    SaveConfirmationItems::No =>{
                        //do not jump TSC
                    },
                }

            },
            //idler case
            _ => 
            {

            },
        }

        Ok(())
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult 
    {
        self.sc_menu.draw(state, ctx)?;

        Ok(())
    }



}



enum MenuMode
{
    Overview,
    ConfirmSave,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HiScoreDispItems
{
    Banner,
    Continue,
}
impl Default for HiScoreDispItems {
    fn default() -> Self {
        HiScoreDispItems::Continue
    }
}

pub struct HiScoreMenu {
    //the internal menu items, using the enums defined above to define menu objects
    hs_menu: Menu<HiScoreDispItems>,
    latest_score: LevelScore,
    controller: CombinedMenuController,
    save_confirm_menu: SaveConfirmationMenu,
    pub is_visible: bool,
    menu_mode: MenuMode,

    
}

impl HiScoreMenu {

    //constructor
    pub fn new() -> HiScoreMenu
    {
        HiScoreMenu {
            hs_menu: Menu::new(0, 0, 160, 0),
            latest_score: LevelScore::new(),
            controller: CombinedMenuController::new(),
            save_confirm_menu: SaveConfirmationMenu::new(),
            is_visible: false,
            menu_mode: MenuMode::Overview,
        }
    }

    //called whenever this menu is switched to
    pub fn init(&mut self, state: &mut SharedGameState, ctx: &Context) -> GameResult 
    {
        //controllers
        self.controller.add(state.settings.create_player1_controller());
        self.controller.add(state.settings.create_player2_controller());

        //recreate the menu on init (why do we do this in new() as well?)
        self.hs_menu = Menu::new(0, 0, 160, 0);

        //make title
        self.hs_menu.push_entry(HiScoreDispItems::Banner, MenuEntry::Disabled(String::from("Results")) );


        //make continue button
        self.hs_menu.push_entry(HiScoreDispItems::Continue, MenuEntry::Active(state.loc.t("common.continue").to_owned()));

        self.update_sizes(state);

        self.save_confirm_menu.init(state, ctx)?;

        Ok(())
    }

    //handle window scaling
    fn update_sizes(&mut self, state: &SharedGameState)
    {
        self.hs_menu.update_width(state);
        self.hs_menu.update_height(state);
        self.hs_menu.x = ((state.canvas_size.0 - self.hs_menu.width as f32) / 2.0).floor() as isize;
        self.hs_menu.y = ((state.canvas_size.1 - self.hs_menu.height as f32) / 2.0).floor() as isize;
    }

    //process user input and behavior
    pub fn tick(
        &mut self,
        //exit_action: &mut dyn FnMut(),
        //controller: &mut CombinedMenuController,
        state: &mut SharedGameState,
        ctx: &mut Context,
    ) -> GameResult {

        //get the current score from the guitar manager
        //let latest_score = game_scene.guitar_manager.get_current_score();
        //let last_score = state.stages[stage_no].clone();


        self.controller.update(state, ctx)?;
        self.controller.update_trigger();

        //switch menu mode
        match self.menu_mode
        {
            MenuMode::Overview => {

                self.update_sizes(state);

                //run cases depending on user action, determined by the controller fed into the function
                match self.hs_menu.tick(&mut self.controller, state)
                {
                    //MenuSelectionResult::Canceled => {}, //exit_action(),
                    MenuSelectionResult::Selected(sub_item, _) =>
                    {
                        match sub_item
                        {
                            HiScoreDispItems::Banner => {}
                            HiScoreDispItems::Continue => {self.menu_mode = MenuMode::ConfirmSave}, //{exit_action()}
                        }
                    },
                    //idler case
                    _ => 
                    {
                        
                    },
                }
            },
            MenuMode::ConfirmSave => {
                self.save_confirm_menu.tick(&mut self.controller, state, ctx, &mut Guitar::new())?;
            }
        }


        Ok(())
    }

    //look pretty
    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult 
    {
        //switch menu mode
        match self.menu_mode
        {
            MenuMode::Overview => {
                self.hs_menu.draw(state, ctx)?;
            }
            MenuMode::ConfirmSave => {
                self.save_confirm_menu.draw(state, ctx)?;
            }
        }

        Ok(())
    }

}


*/




