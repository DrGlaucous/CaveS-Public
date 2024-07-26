use crate::common::Rect;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::game::shared_game_state::{PlayerCount, SharedGameState};
use crate::game::scripting::tsc::text_script::TextScriptExecutionState;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::MenuEntry;
use crate::menu::{Menu, MenuSelectionResult};
use crate::scene::game_scene::GameScene;
pub struct StageSelectMenu {
    left_arrow_visible: bool,
    right_arrow_visible: bool,
    stage_locked: bool,
}

impl StageSelectMenu {
    pub fn new() -> StageSelectMenu {
        StageSelectMenu {
            left_arrow_visible: true,
            right_arrow_visible: true,
            stage_locked: false,
        }
    }


    pub fn tick(
        &mut self,
        exit_action: &mut dyn FnMut(),
        controller: &mut CombinedMenuController,
        state: &mut SharedGameState,
        ctx: &mut Context,
        game_scene: &mut GameScene
    ) -> GameResult {

        //its up to the TSC to load stuff. this just reacts to what's there


        //return to main title
        if controller.trigger_back() {
            state.sound_manager.play_sfx(5);
            exit_action();
        }

        //TRA to the next map's preview and load the timer for it
        if controller.trigger_left() {

            state.textscript_vm.state = TextScriptExecutionState::Running(state.constants.game.stage_left_event, 0);
            state.sound_manager.play_sfx(1);

            //todo: hide arrow if TSC is not found

        } else if controller.trigger_right() {
            state.textscript_vm.state = TextScriptExecutionState::Running(state.constants.game.stage_right_event, 0);
            state.sound_manager.play_sfx(1);
        }




        Ok(())
    }

    pub fn draw(
        &self, 
        state: &mut SharedGameState, 
        ctx: &mut Context, 
        custom_height_margin: Option<(f32, f32)>,
        bounding_rect: Rect<f32>, //minx, miny, maxx, maxy
    ) -> GameResult {
        
        let play = Rect::new(56, 152, 80, 176);
        let lock = Rect::new(80, 152, 104, 176);
        let camera = Rect::new(104, 144, 136, 160);
        let l_arrow = Rect::new(104, 160, 112, 168);
        let r_arrow = Rect::new(112, 160, 120, 168);

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;

        //draw play icon
        let icon_rc = if self.stage_locked {lock} else {play};
        batch.add_rect(
            ((bounding_rect.width() - icon_rc.width() as f32) / 2.0).floor() + bounding_rect.left,
            ((bounding_rect.height() - icon_rc.height() as f32) / 2.0).floor() + bounding_rect.top,
            &icon_rc,
        );

        //draw left arrow
        if self.left_arrow_visible {
            batch.add_rect(
                ((bounding_rect.width() - l_arrow.width() as f32) / 2.0).floor() + bounding_rect.left - 30.0,
                ((bounding_rect.height() - l_arrow.height() as f32) / 2.0).floor() + bounding_rect.top,
                &l_arrow,
            );
        }
        //draw right arrow
        if self.right_arrow_visible {
            batch.add_rect(
                ((bounding_rect.width() - r_arrow.width() as f32) / 2.0).floor() + bounding_rect.left + 30.0,
                ((bounding_rect.height() - r_arrow.height() as f32) / 2.0).floor() + bounding_rect.top,
                &r_arrow,
            );
        }

        batch.draw(ctx)?;


        Ok(())
    }

    fn start_game(&mut self, player_count: PlayerCount, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.player_count = player_count;
        state.reload_resources(ctx)?;
        state.load_or_start_game(ctx)?;
        Ok(())
    }
}
