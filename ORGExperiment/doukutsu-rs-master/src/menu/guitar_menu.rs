use crate::common::{Color, VERSION_BANNER};
use crate::components::background::Background;
use crate::components::compact_jukebox::CompactJukebox;
use crate::components::nikumaru::NikumaruCounter;
use crate::entity::GameEntity;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::game::frame::Frame;
use crate::game::map::Map;
use crate::game::shared_game_state::{
    GameDifficulty, MenuCharacter, ReplayKind, ReplayState, Season, SharedGameState, TileSize, self,
};
use crate::game::stage::{BackgroundType, NpcType, Stage, StageData, StageTexturePaths, Tileset};
use crate::graphics::font::Font;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::input::touch_controls::TouchControlType;
use crate::menu::coop_menu::PlayerCountMenu;
use crate::menu::save_select_menu::SaveSelectMenu;
use crate::menu::settings_menu::SettingsMenu;
use crate::menu::{Menu, MenuEntry, MenuSelectionResult};
use crate::scene::jukebox_scene::JukeboxScene;
use crate::scene::Scene;


use crate::game::guitar::Guitar;
use crate::menu::save_select_menu::MenuSaveInfo;





//holds settings relating to the main list
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MenuItems
{
    Banner,
    Back,
    Level(usize),
}
impl Default for MenuItems {
    fn default() -> Self {
        MenuItems::Level(0)
    }
}

//main menu
pub struct StageSelectMenu {
    list_menu: Menu<MenuItems>,
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
        self.list_menu.push_entry(MenuItems::Banner, MenuEntry::Disabled(String::from("Stages")) );

        //make back button


        //push all maps in the stage table to the menu
        let mut i: usize = 0;
        for map in state.stages.iter()
        {
            self.list_menu.push_entry(MenuItems::Level(i), MenuEntry::Active(map.name.clone() + " TODO: Score here") );
            self.list_menu.selected = MenuItems::Level(i);
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
                    MenuItems::Banner => {}
                    MenuItems::Back => {exit_action()}
                    MenuItems::Level(level) =>{
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


































