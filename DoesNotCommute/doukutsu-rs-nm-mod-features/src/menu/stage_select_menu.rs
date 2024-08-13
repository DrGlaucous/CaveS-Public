use crate::common::Rect;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::game;
use crate::game::shared_game_state::{PlayerCount, SharedGameState};
use crate::game::scripting::tsc::text_script::{TextScriptExecutionState, ScriptMode, Scripts};
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::MenuEntry;
use crate::menu::{Menu, MenuSelectionResult};
use crate::game::settings::ControllerType;
use crate::scene::game_scene::GameScene;
use crate::graphics::font::{Font, Symbols};

#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
#[allow(unused)]
pub enum CurrentMenu {
    SelectMenu,
    ConfirmMenu,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ConfirmMenuEntry {
    Play,
    WatchReplay,
    Back,
}
impl Default for ConfirmMenuEntry {
    fn default() -> Self {
        ConfirmMenuEntry::Play
    }
}


pub struct StageSelectMenu {
    current_menu: CurrentMenu,
    load_confirm: Menu<ConfirmMenuEntry>,
    left_arrow_visible: bool,
    right_arrow_visible: bool,
    stage_unlocked: bool,
    stage_completed: bool,
}

impl StageSelectMenu {
    pub fn new() -> StageSelectMenu {
        StageSelectMenu {
            current_menu: CurrentMenu::SelectMenu,
            load_confirm: Menu::new(0, 0, 130, 0),
            left_arrow_visible: true,
            right_arrow_visible: true,
            stage_unlocked: false,
            stage_completed: false,
        }
    }

    pub fn init(&mut self, state: &mut SharedGameState, ctx: &Context) -> GameResult {
        self.load_confirm = Menu::new(0, 0, 130, 0);

        // let aaa = state.loc.t("menus.load_confirm_menu.play");
        // let bbb = state.loc.t("common.back");

        self.load_confirm
            .push_entry(ConfirmMenuEntry::Play, MenuEntry::Active(state.loc.t("menus.load_confirm_menu.play").to_owned()));
        
        //self.load_confirm
        //    .push_entry(ConfirmMenuEntry::WatchReplay, MenuEntry::Active(state.loc.t("menus.load_confirm_menu.watch_replay").to_owned()));
        
        //start with this menu entry hidden
        self.load_confirm.push_entry(ConfirmMenuEntry::WatchReplay, MenuEntry::Hidden);

        self.load_confirm
            .push_entry(ConfirmMenuEntry::Back, MenuEntry::Active(state.loc.t("common.back").to_owned()));

        self.update_sizes(state);

        Ok(())
    }

    fn update_sizes(&mut self, state: &SharedGameState) {
        self.load_confirm.update_width(state);
        self.load_confirm.update_height(state);
        self.load_confirm.x = ((state.canvas_size.0 - self.load_confirm.width as f32) / 2.0).floor() as isize;
        self.load_confirm.y = ((state.canvas_size.1 - self.load_confirm.height as f32) / 2.0).floor() as isize;
    }

    pub fn tick(
        &mut self,
        exit_action: &mut dyn FnMut(),
        controller: &mut CombinedMenuController,
        state: &mut SharedGameState,
        ctx: &mut Context,
        game_scene: &mut GameScene
    ) -> GameResult {

        self.update_sizes(state);


        //self.load_confirm.set_entry(ConfirmMenuEntry::WatchReplay, MenuEntry::Hidden);
        //self.load_confirm.set_entry(ConfirmMenuEntry::WatchReplay, MenuEntry::Active(state.loc.t("menus.load_confirm_menu.watch_replay").to_owned()));

        
        //TSC will try to load all NPC keylogs, and finally CNP a null to the camera switcher, an indicator that the level was completed
        self.stage_completed = state.control_flags.replay_mode(); //game_scene.npc_list.is_alive_by_type(374);

        //change replay visibility based on stage_completed var (but only if it's not already set to this) (we could also use "set_id"... would it have much impact on performance?)
        for a in &mut self.load_confirm.entries {
            match (self.stage_completed, &a) {
                (true, (ConfirmMenuEntry::WatchReplay, MenuEntry::Hidden)) => {
                    a.1 = MenuEntry::Active(state.loc.t("menus.load_confirm_menu.watch_replay").to_owned());
                }
                (false, (ConfirmMenuEntry::WatchReplay,  MenuEntry::Active(_))) => {
                    a.1 = MenuEntry::Hidden;
                }
                _ => {}
            }
        }

        match self.current_menu {
            CurrentMenu::SelectMenu => {

                //it's up to the TSC to load stuff. this just reacts to what's there

                self.left_arrow_visible = state.textscript_vm.scripts.borrow().script_num_exists(
                    ScriptMode::Map, 
                    state.constants.game.stage_left_event
                ); 

                self.right_arrow_visible = state.textscript_vm.scripts.borrow().script_num_exists(
                    ScriptMode::Map, 
                    state.constants.game.stage_right_event
                ); 

                self.stage_unlocked = game_scene.nikumaru.tick > 0;


                //lockout while keyed
                if !(state.control_flags.control_enabled()) {
                    return Ok(())
                }

                //confirm menu entry
                if controller.trigger_ok() {
                    if self.stage_unlocked {
                        state.sound_manager.play_sfx(18);
                        self.current_menu = CurrentMenu::ConfirmMenu;
                        //don't allow any of the other actions to happen this round
                        return Ok(())
                    } else {
                        state.sound_manager.play_sfx(37);
                    }
                }

                //return to main title
                if controller.trigger_back() {
                    state.sound_manager.play_sfx(5);
                    state.textscript_vm.state = TextScriptExecutionState::Running(state.constants.game.title_go_event, 0);
                    exit_action();
                }

                //TRA to the next map's preview and load the timer for it
                if controller.trigger_left() {

                    if self.left_arrow_visible {
                        state.textscript_vm.state = TextScriptExecutionState::Running(state.constants.game.stage_left_event, 0);
                        state.sound_manager.play_sfx(1);
                    } else {
                        state.sound_manager.play_sfx(37);
                    }

                } else if controller.trigger_right() {
                    if self.right_arrow_visible {
                        state.textscript_vm.state = TextScriptExecutionState::Running(state.constants.game.stage_right_event, 0);
                        state.sound_manager.play_sfx(1);
                    } else {
                        state.sound_manager.play_sfx(37);
                    }
                }

            },
            CurrentMenu::ConfirmMenu => {
                match self.load_confirm.tick(controller, state) {
                    MenuSelectionResult::Selected(ConfirmMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                        self.current_menu = CurrentMenu::SelectMenu;
                    }
                    MenuSelectionResult::Selected(ConfirmMenuEntry::Play, _) => {

                        state.reload_resources(ctx)?;
                        state.start_new_game_at(
                            ctx,
                            game_scene.stage_id as u16,
                            state.constants.game.stage_play_event,
                            0,
                            0,
                        )?;

                    }
                    MenuSelectionResult::Selected(ConfirmMenuEntry::WatchReplay, _) => {
                        state.reload_resources(ctx)?;
                        state.start_new_game_at(
                            ctx,
                            game_scene.stage_id as u16,
                            state.constants.game.stage_replay_event,
                            0,
                            0,
                        )?;
                    }
                    _ => (),
                }
            },
        }

        Ok(())
    
    }

    pub fn draw(
        &self, 
        state: &mut SharedGameState, 
        ctx: &mut Context, 
        custom_height_margin: Option<(f32, f32)>,
        bounding_rect: Rect<f32>, //minx, miny, maxx, maxy
        game_scene: &GameScene,
    ) -> GameResult {
        
        match self.current_menu {
            CurrentMenu::SelectMenu => {

                let play = Rect::new(56, 152, 80, 176);
                let lock = Rect::new(80, 152, 104, 176);
                let camera = Rect::new(104, 144, 136, 160);
                let l_arrow = Rect::new(104, 160, 112, 168);
                let r_arrow = Rect::new(112, 160, 120, 168);
        
                let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;
        
                //draw play icon
                let icon_rc = if self.stage_unlocked {play} else {lock};
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
        
                //draw camera switcher
                if self.stage_completed
                {
        
                    let x = bounding_rect.width() / 2.0 + bounding_rect.left;
                    let y = bounding_rect.top;
        
                    //for now, we have hardlined the camera icon at 32 pixels wide * 3 (32 for left key, 32 for cam, 32 for right)
                    let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;
                    batch.add_rect(
                        x - camera.width() as f32 / 2.0,
                        y,
                        &camera,
                    );
                    batch.draw(ctx)?;
        
        
                    match state.settings.player1_controller_type {
                        ControllerType::Keyboard => {
        
                            //get scancode and print its name
                            let sc_string = format!("{:?}", state.settings.player1_key_map.prev_weapon);
                            let value = sc_string.as_str();
        
                            //left
                            let text_width = state.font.builder().compute_width(value);
                            state
                                .font
                                .builder()
                                .position(
                                    x - camera.width() as f32 / 2.0 - text_width,
                                    y
                                )
                                .draw(value, ctx, &state.constants, &mut state.texture_set)?;
        
        
                            //right
                            let sc_string = format!("{:?}", state.settings.player1_key_map.next_weapon);
                            let value = sc_string.as_str();
                            state
                                .font
                                .builder()
                                .position(
                                    x + camera.width() as f32 / 2.0,
                                    y
                                )
                                .draw(value, ctx, &state.constants, &mut state.texture_set)?;
        
                        },
                        ControllerType::Gamepad(index) => {
                            let gamepad_sprite_offset = ctx.gamepad_context.get_gamepad_sprite_offset(index as usize);
                            
                            //left switch
                            let rc = state.settings.player1_controller_button_map.prev_weapon.get_rect(gamepad_sprite_offset, &state.constants);
                            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "buttons")?;
                            batch.add_rect(
                                x - camera.width() as f32 / 2.0 - rc.width() as f32, 
                                y, 
                                &rc);
        
                            //right switch
                            let rc = state.settings.player1_controller_button_map.next_weapon.get_rect(gamepad_sprite_offset, &state.constants);
                            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "buttons")?;
                            batch.add_rect(
                                x + camera.width() as f32 / 2.0, 
                                y, 
                                &rc);
        
        
                            batch.draw(ctx)?;
        
                        },
                    };
        
        
        
                
                }
        
        
            }
            CurrentMenu::ConfirmMenu => {
                self.load_confirm.draw(state, ctx, custom_height_margin)?;
            }
        }

        //draw starting time
        if self.stage_unlocked {
            game_scene.nikumaru.draw_at(state, ctx, bounding_rect.left + 4.0, bounding_rect.top + 4.0)?;
        }

        Ok(())
    }

    fn start_game(&mut self, player_count: PlayerCount, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.player_count = player_count;
        state.reload_resources(ctx)?;
        state.load_or_start_game(ctx)?;
        Ok(())
    }
}
