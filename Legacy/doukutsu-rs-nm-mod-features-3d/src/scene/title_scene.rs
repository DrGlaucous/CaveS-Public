use crate::common::{Color, VERSION_BANNER};
use crate::components::background::Background;
use crate::components::compact_jukebox::CompactJukebox;
use crate::components::nikumaru::NikumaruCounter;
use crate::entity::GameEntity;
use crate::framework::graphics;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::game::frame::Frame;
use crate::game::map::Map;
use crate::game::shared_game_state::{
    GameDifficulty, MenuCharacter, ReplayKind, ReplayState, Season, SharedGameState, TileSize,
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

#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
#[allow(unused)]
enum CurrentMenu {
    MainMenu,
    OptionMenu,
    SaveSelectMenu,
    ChallengesMenu,
    ChallengeConfirmMenu,
    PlayerCountMenu,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MainMenuEntry {
    Start,
    Challenges,
    Options,
    Editor,
    Jukebox,
    Quit,
}

impl Default for MainMenuEntry {
    fn default() -> Self {
        MainMenuEntry::Start
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ChallengesMenuEntry {
    Back,
    Challenge(usize),
}

impl Default for ChallengesMenuEntry {
    fn default() -> Self {
        ChallengesMenuEntry::Back
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ConfirmMenuEntry {
    Title,
    StartChallenge,
    Replay(ReplayKind),
    DeleteReplay,
    Back,
}

impl Default for ConfirmMenuEntry {
    fn default() -> Self {
        ConfirmMenuEntry::StartChallenge
    }
}

pub struct TitleScene {
    tick: usize,
    controller: CombinedMenuController,
    current_menu: CurrentMenu,
    main_menu: Menu<MainMenuEntry>,
    save_select_menu: SaveSelectMenu,
    challenges_menu: Menu<ChallengesMenuEntry>,
    confirm_menu: Menu<ConfirmMenuEntry>,
    coop_menu: PlayerCountMenu,
    settings_menu: SettingsMenu,
    background: Background,
    frame: Frame,
    nikumaru_rec: NikumaruCounter,
    compact_jukebox: CompactJukebox,
    stage: Stage,
    textures: StageTexturePaths,
    test_var: i32,
}

impl TitleScene {
    pub fn new() -> Self {
        let fake_stage = Stage {
            map: Map { width: 0, height: 0, tiles: vec![], attrib: vec![], tile_size: TileSize::Tile16x16, animation_config: None },
            data: StageData {
                name: String::new(),
                name_jp: String::new(),
                map: String::new(),
                boss_no: 0,
                tileset: Tileset { name: "0".to_string() },
                pxpack_data: None,
                background: crate::game::stage::Background::new("bkMoon"),
                background_type: BackgroundType::Outside,
                background_color: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
                npc1: NpcType::new("0"),
                npc2: NpcType::new("0"),
            },
        };
        let mut textures = StageTexturePaths::new();
        textures.update(&fake_stage);

        let mut settings_menu = SettingsMenu::new();
        settings_menu.on_title = true;

        Self {
            tick: 0,
            controller: CombinedMenuController::new(),
            current_menu: CurrentMenu::MainMenu,
            main_menu: Menu::new(0, 0, 100, 0),
            save_select_menu: SaveSelectMenu::new(),
            challenges_menu: Menu::new(0, 0, 150, 0),
            confirm_menu: Menu::new(0, 0, 150, 0),
            coop_menu: PlayerCountMenu::new(),
            settings_menu,
            background: Background::new(),
            frame: Frame::new(),
            nikumaru_rec: NikumaruCounter::new(),
            compact_jukebox: CompactJukebox::new(),
            stage: fake_stage,
            textures,

            test_var: 0,
        }
    }

    fn draw_text_centered(&self, text: &str, y: f32, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.font.builder().center(state.canvas_size.0).y(y).shadow(true).draw(
            text,
            ctx,
            &state.constants,
            &mut state.texture_set,
        )?;

        Ok(())
    }

    fn draw_three_d(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        
        
        //update backend stuff
        //this pre-divides sub-pixels by 0x200 for us, so these are raw pixel coordinates
        let (mut frame_x, mut frame_y) = (self.frame.x as f32, self.frame.y as f32); //self.frame.xy_interpolated(state.frame_time);
        //convert to meters
        frame_x /= 16.0;
        frame_y /= -16.0; //invert y as well
        graphics::update_frame_location(ctx, frame_x, frame_y)?;

        //state.frame_time is a number between 0.0 and 1.0, showing how far along a frame we are.
        let offset_time = (state.frame_time * (1.0/50.0)) as f32;
        graphics::increment_animation_time(ctx, 0.0, offset_time)?;

        //set target to the 3d surface's 2D plane
        //we're not drawing anything here right now, so just clear it
        graphics::set_render_target(ctx, state.char_plane_canvas.as_ref())?;
        graphics::clear(ctx, Color::from_rgba(0, 0, 0, 0));


        //put 2d plane stuff here
        //self.background.draw(state, ctx, &self.frame, &self.textures, &self.stage, false)?;



        //draw the 3d stuff

        graphics::set_render_target(ctx, None)?;
        //bind char plane to the char plane canvas
        graphics::set_3d_char_plane(ctx, state.char_plane_canvas.as_ref().unwrap())?;
        //draw the 3D layer
        graphics::draw_3d(ctx, None)?; //(ctx, state.three_d_canvas.as_ref())?;

        
        Ok(())
    }

    pub fn update_menu_cursor(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let minutes = self.nikumaru_rec.tick / (60 * state.settings.timing_mode.get_tps());
        let mut song_id: usize;

        if self.nikumaru_rec.shown && minutes < 3 {
            state.menu_character = MenuCharacter::Sue;

            song_id = 2;

            if state.constants.is_cs_plus && !state.constants.is_switch {
                self.compact_jukebox.show();
            }
        } else if self.nikumaru_rec.shown && minutes < 4 {
            state.menu_character = MenuCharacter::King;
            song_id = 41;
        } else if self.nikumaru_rec.shown && minutes < 5 {
            state.menu_character = MenuCharacter::Toroko;
            song_id = 40;
        } else if self.nikumaru_rec.shown && minutes < 6 {
            state.menu_character = MenuCharacter::Curly;
            song_id = 36;
        } else {
            state.menu_character = MenuCharacter::Quote;
            song_id = 1;
        }

        song_id = 0;


        // if state.settings.soundtrack == "New" && Season::current() == Season::PixelBirthday {
        //     song_id = 43;
        // }

        if self.compact_jukebox.is_shown() {
            self.compact_jukebox.change_song(song_id, state, ctx)?;
        } else {
            if song_id != state.sound_manager.current_song() {
                state.sound_manager.play_song(song_id, &state.constants, &state.settings, ctx, false)?;
            }
        }

        Ok(())
    }

    pub fn open_settings_menu(&mut self) -> GameResult {
        self.current_menu = CurrentMenu::OptionMenu;
        Ok(())
    }
}

static CREATOR_CREDIT: &str = "2024.10  Dr. G";
// Freeware
//static COPYRIGHT_NICALIS: &str = "@2022 NICALIS INC."; // Nicalis font uses @ for copyright

impl Scene for TitleScene {
    fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if !state.mod_path.is_none() {
            state.mod_path = None;
            state.reload_resources(ctx)?;
        }

        self.controller.add(state.settings.create_player1_controller());
        self.controller.add(state.settings.create_player2_controller());

        self.main_menu
            .push_entry(MainMenuEntry::Start, MenuEntry::Active(state.loc.t("menus.main_menu.start").to_owned()));

        if !state.mod_list.mods.is_empty() {
            self.main_menu.push_entry(
                MainMenuEntry::Challenges,
                MenuEntry::Active(state.loc.t("menus.main_menu.challenges").to_owned()),
            );
        }

        self.main_menu
            .push_entry(MainMenuEntry::Options, MenuEntry::Active(state.loc.t("menus.main_menu.options").to_owned()));

        if cfg!(feature = "editor") {
            self.main_menu
                .push_entry(MainMenuEntry::Editor, MenuEntry::Active(state.loc.t("menus.main_menu.editor").to_owned()));
        }

        if state.constants.is_switch {
            self.main_menu.push_entry(
                MainMenuEntry::Jukebox,
                MenuEntry::Active(state.loc.t("menus.main_menu.jukebox").to_owned()),
            );
        }

        self.main_menu
            .push_entry(MainMenuEntry::Quit, MenuEntry::Active(state.loc.t("menus.main_menu.quit").to_owned()));

        self.settings_menu.init(state, ctx)?;

        self.save_select_menu.init(state, ctx)?;

        self.coop_menu.on_title = true;
        self.coop_menu.init(state)?;

        let mut selected = ChallengesMenuEntry::Back;
        let mut mutate_selection = true;

        for (idx, mod_info) in state.mod_list.mods.iter().enumerate() {
            if !mod_info.valid {
                self.challenges_menu
                    .push_entry(ChallengesMenuEntry::Challenge(idx), MenuEntry::Disabled(mod_info.path.clone()));
                continue;
            }
            if mod_info.satisfies_requirement(&state.mod_requirements) {
                self.challenges_menu
                    .push_entry(ChallengesMenuEntry::Challenge(idx), MenuEntry::Active(mod_info.name.clone()));

                if mutate_selection {
                    selected = ChallengesMenuEntry::Challenge(idx);
                    mutate_selection = false;
                }
            } else {
                self.challenges_menu
                    .push_entry(ChallengesMenuEntry::Challenge(idx), MenuEntry::Disabled("???".to_owned()));
            }
        }
        self.challenges_menu
            .push_entry(ChallengesMenuEntry::Back, MenuEntry::Active(state.loc.t("common.back").to_owned()));
        self.challenges_menu.selected = selected;

        self.confirm_menu.push_entry(ConfirmMenuEntry::Title, MenuEntry::Disabled(String::new()));
        self.confirm_menu.push_entry(
            ConfirmMenuEntry::StartChallenge,
            MenuEntry::Active(state.loc.t("menus.challenge_menu.start").to_owned()),
        );
        self.confirm_menu.push_entry(ConfirmMenuEntry::Replay(ReplayKind::Best), MenuEntry::Hidden);
        self.confirm_menu.push_entry(ConfirmMenuEntry::Replay(ReplayKind::Last), MenuEntry::Hidden);
        self.confirm_menu.push_entry(ConfirmMenuEntry::DeleteReplay, MenuEntry::Hidden);
        self.confirm_menu.push_entry(ConfirmMenuEntry::Back, MenuEntry::Active(state.loc.t("common.back").to_owned()));
        self.confirm_menu.selected = ConfirmMenuEntry::StartChallenge;

        self.controller.update(state, ctx)?;
        self.controller.update_trigger();

        self.nikumaru_rec.load_counter(state, ctx)?;
        self.update_menu_cursor(state, ctx)?;

        state.replay_state = ReplayState::None;
        state.textscript_vm.flags.set_cutscene_skip(false);
        state.difficulty = GameDifficulty::Normal;

        #[cfg(feature = "discord-rpc")]
        state.discord_rpc.set_idling()?;


        //clear all loaded meshes
        graphics::clear_gltf(ctx)?;


        //load skybox note: debug mode lags with this, so we'll use it later
        //state.load_skybox(ctx, &format!("Mesh/marslike_big_ds.jpg"), false)?;

        //try to load in the space station mesh
        state.load_gltf(ctx, &format!("Mesh/City9/City9.glb"), true, 0)?;
        state.set_ambient_attributes(ctx, Some(Color::from_rgb(0, 0, 0)), Some(0.0))?;
        state.set_model_animation_attributes(ctx, 0, Some("Action"), Some(0.0), Some(true), Some(-1.0))?;



        Ok(())
    }

    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.touch_controls.control_type = TouchControlType::None;


        self.frame.x = -(ctx.screen_size.0 / (2.0 * state.scale)) as i32;
        self.frame.y = -(ctx.screen_size.1 / (2.0 * state.scale)) as i32;
        state.increment_animation_time(ctx, 1.0 / 50.0, 0.0)?; //run by-tick animation

        self.background.tick(state, &self.stage, &self.frame)?;

        self.controller.update(state, ctx)?;
        self.controller.update_trigger();

        //TEST
        if self.controller.trigger_up() || self.controller.trigger_down() || self.controller.trigger_left() || self.controller.trigger_right() {

            let mut c1: i8 = (self.test_var % 128) as i8;
            let mut c2: i8 = ((self.test_var / 128) % 128) as i8;
            if self.controller.trigger_up() {
                c1 += 1;
            } else if self.controller.trigger_down() {
                c1 -= 1;
            }

            if self.controller.trigger_right() {
                c2 += 1;
            } else if self.controller.trigger_left() {
                c2 -= 1;
            }
            c1 = c1.clamp(0, 23);
            c2 = c2.clamp(0, 15);

            self.test_var = (c2 as i32 * 128) + c1 as i32;

            state.load_gltf(ctx, &format!(""), true, 0)?;
            state.load_gltf_test(ctx, &format!("Mesh/City9/City9.glb"), true, 0, self.test_var as usize)?;

        }

        self.main_menu.update_width(state);
        self.main_menu.update_height(state);
        self.main_menu.x = ((state.canvas_size.0 - self.main_menu.width as f32) / 2.0).floor() as isize;
        self.main_menu.y = ((state.canvas_size.1 + 70.0 - self.main_menu.height as f32) / 2.0).floor() as isize;

        self.challenges_menu.update_width(state);
        self.challenges_menu.update_height(state);
        self.challenges_menu.x = ((state.canvas_size.0 - self.challenges_menu.width as f32) / 2.0).floor() as isize;
        self.challenges_menu.y =
            ((state.canvas_size.1 + 30.0 - self.challenges_menu.height as f32) / 2.0).floor() as isize;

        if self.controller.trigger_left()
            && self.compact_jukebox.is_shown()
            && self.current_menu == CurrentMenu::MainMenu
        {
            self.compact_jukebox.prev_song(state, ctx)?;
        }

        if self.controller.trigger_right()
            && self.compact_jukebox.is_shown()
            && self.current_menu == CurrentMenu::MainMenu
        {
            self.compact_jukebox.next_song(state, ctx)?;
        }

        match self.current_menu {
            CurrentMenu::MainMenu => match self.main_menu.tick(&mut self.controller, state) {
                MenuSelectionResult::Selected(MainMenuEntry::Start, _) => {
                    state.mod_path = None;
                    self.save_select_menu.init(state, ctx)?;
                    self.save_select_menu.set_skip_difficulty_menu(!state.constants.has_difficulty_menu);
                    self.current_menu = CurrentMenu::SaveSelectMenu;
                }
                MenuSelectionResult::Selected(MainMenuEntry::Challenges, _) => {
                    self.current_menu = CurrentMenu::ChallengesMenu;
                }
                MenuSelectionResult::Selected(MainMenuEntry::Options, _) => {
                    self.current_menu = CurrentMenu::OptionMenu;
                }
                MenuSelectionResult::Selected(MainMenuEntry::Editor, _) => {
                    // this comment is just there because rustfmt removes parenthesis around the match case and breaks compilation
                    #[cfg(feature = "editor")]
                    {
                        use crate::scene::editor_scene::EditorScene;
                        state.next_scene = Some(Box::new(EditorScene::new()));
                    }
                }
                MenuSelectionResult::Selected(MainMenuEntry::Jukebox, _) => {
                    state.next_scene = Some(Box::new(JukeboxScene::new()));
                }
                MenuSelectionResult::Selected(MainMenuEntry::Quit, _) => {
                    state.shutdown();
                }
                _ => {}
            },
            CurrentMenu::OptionMenu => {
                let timing_mode = state.settings.timing_mode;
                let cm = &mut self.current_menu;
                self.settings_menu.tick(
                    &mut || {
                        *cm = CurrentMenu::MainMenu;
                    },
                    &mut self.controller,
                    state,
                    ctx,
                )?;
                if timing_mode != state.settings.timing_mode {
                    self.update_menu_cursor(state, ctx)?;
                }
            }
            CurrentMenu::SaveSelectMenu => {
                let cm = &mut self.current_menu;
                let rm = if state.mod_path.is_none() { CurrentMenu::MainMenu } else { CurrentMenu::ChallengesMenu };
                self.save_select_menu.tick(
                    &mut || {
                        *cm = rm;
                    },
                    &mut self.controller,
                    state,
                    ctx,
                )?;
            }
            CurrentMenu::ChallengesMenu => match self.challenges_menu.tick(&mut self.controller, state) {
                MenuSelectionResult::Selected(ChallengesMenuEntry::Challenge(idx), _) => {
                    if let Some(mod_info) = state.mod_list.mods.get(idx) {
                        state.mod_path = Some(mod_info.path.clone());
                        if mod_info.save_slot >= 0 {
                            self.save_select_menu.init(state, ctx)?;
                            self.save_select_menu.set_skip_difficulty_menu(true);
                            self.nikumaru_rec.load_counter(state, ctx)?;
                            self.current_menu = CurrentMenu::SaveSelectMenu;
                        } else {
                            let mod_name = mod_info.name.clone();
                            self.confirm_menu.width =
                                (state.font.builder().compute_width(&mod_name).max(50.0) + 32.0) as u16;

                            self.confirm_menu.set_entry(ConfirmMenuEntry::Title, MenuEntry::Disabled(mod_name));

                            if state.has_replay_data(ctx, ReplayKind::Best) {
                                self.confirm_menu.set_entry(
                                    ConfirmMenuEntry::Replay(ReplayKind::Best),
                                    MenuEntry::Active(state.loc.t("menus.challenge_menu.replay_best").to_owned()),
                                );
                                self.confirm_menu.set_entry(
                                    ConfirmMenuEntry::DeleteReplay,
                                    MenuEntry::Active(state.loc.t("menus.challenge_menu.delete_replay").to_owned()),
                                );
                            } else {
                                self.confirm_menu
                                    .set_entry(ConfirmMenuEntry::Replay(ReplayKind::Best), MenuEntry::Hidden);
                                self.confirm_menu.set_entry(ConfirmMenuEntry::DeleteReplay, MenuEntry::Hidden);
                            }

                            if state.has_replay_data(ctx, ReplayKind::Last) {
                                self.confirm_menu.set_entry(
                                    ConfirmMenuEntry::Replay(ReplayKind::Last),
                                    MenuEntry::Active(state.loc.t("menus.challenge_menu.replay_last").to_owned()),
                                );
                            } else {
                                self.confirm_menu
                                    .set_entry(ConfirmMenuEntry::Replay(ReplayKind::Last), MenuEntry::Hidden);
                            }

                            self.nikumaru_rec.load_counter(state, ctx)?;
                            self.current_menu = CurrentMenu::ChallengeConfirmMenu;
                        }
                    }
                }
                MenuSelectionResult::Selected(ChallengesMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                    state.mod_path = None;
                    self.nikumaru_rec.load_counter(state, ctx)?;
                    self.current_menu = CurrentMenu::MainMenu;
                }
                _ => (),
            },
            CurrentMenu::ChallengeConfirmMenu => match self.confirm_menu.tick(&mut self.controller, state) {
                MenuSelectionResult::Selected(ConfirmMenuEntry::StartChallenge, _) => {
                    state.difficulty = GameDifficulty::Normal;
                    state.replay_state = ReplayState::Recording;
                    self.current_menu = CurrentMenu::PlayerCountMenu;
                }
                MenuSelectionResult::Selected(ConfirmMenuEntry::Replay(kind), _) => {
                    state.difficulty = GameDifficulty::Normal;
                    state.replay_state = ReplayState::Playback(kind);
                    state.reload_resources(ctx)?;
                    state.start_new_game(ctx)?;
                }
                MenuSelectionResult::Selected(ConfirmMenuEntry::DeleteReplay, _) => {
                    state.delete_replay_data(ctx, ReplayKind::Best)?;
                    self.current_menu = CurrentMenu::ChallengesMenu;
                }
                MenuSelectionResult::Selected(ConfirmMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                    self.current_menu = CurrentMenu::ChallengesMenu;
                }
                _ => (),
            },
            CurrentMenu::PlayerCountMenu => {
                let cm = &mut self.current_menu;
                let rm = CurrentMenu::ChallengeConfirmMenu;
                self.coop_menu.tick(
                    &mut || {
                        *cm = rm;
                    },
                    &mut self.controller,
                    state,
                    ctx,
                )?;
            }
        }

        self.confirm_menu.update_width(state);
        self.confirm_menu.update_height(state);
        self.confirm_menu.x = ((state.canvas_size.0 - self.confirm_menu.width as f32) / 2.0).floor() as isize;
        self.confirm_menu.y = ((state.canvas_size.1 + 30.0 - self.confirm_menu.height as f32) / 2.0).floor() as isize;

        self.tick += 1;

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {

        self.draw_three_d(state, ctx)?;

        // if self.current_menu == CurrentMenu::MainMenu {
        //     let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Title")?;
        //     let logo_x_offset =
        //         if state.settings.original_textures && state.constants.supports_og_textures { 20.0 } else { 0.0 };
        //     batch.add_rect(
        //         ((state.canvas_size.0 - state.constants.title.logo_rect.width() as f32) / 2.0).floor() + logo_x_offset,
        //         40.0,
        //         &state.constants.title.logo_rect,
        //     );
        //     batch.add_rect(
        //         ((state.canvas_size.0 - state.constants.title.logo_splash_rect.width() as f32) / 2.0).floor() + 72.0,
        //         88.0,
        //         &state.constants.title.logo_splash_rect,
        //     );
        //     batch.draw(ctx)?;
        // } else

        if self.current_menu != CurrentMenu::MainMenu {
            let window_title = match self.current_menu {
                CurrentMenu::ChallengesMenu => state.loc.t("menus.main_menu.challenges"),
                CurrentMenu::ChallengeConfirmMenu | CurrentMenu::SaveSelectMenu => state.loc.t("menus.main_menu.start"),
                CurrentMenu::OptionMenu => state.loc.t("menus.main_menu.options"),
                CurrentMenu::MainMenu => unreachable!(),
                CurrentMenu::PlayerCountMenu => state.loc.t("menus.main_menu.start"),
            };
            state
                .font
                .builder()
                .shadow(true)
                .position(0.0, state.font.line_height())
                .center(state.canvas_size.0)
                .draw(&window_title, ctx, &state.constants, &mut state.texture_set)?;
        }

        if self.current_menu == CurrentMenu::MainMenu {
            self.draw_text_centered(&VERSION_BANNER, state.canvas_size.1 - 15.0, state, ctx)?;

            self.draw_text_centered(CREATOR_CREDIT, state.canvas_size.1 - 30.0, state, ctx)?;

            self.compact_jukebox.draw(state, ctx, &self.frame)?;
        }

        self.nikumaru_rec.draw(state, ctx, &self.frame)?;

        match self.current_menu {
            CurrentMenu::MainMenu => self.main_menu.draw(state, ctx)?,
            CurrentMenu::ChallengesMenu => self.challenges_menu.draw(state, ctx)?,
            CurrentMenu::ChallengeConfirmMenu => self.confirm_menu.draw(state, ctx)?,
            CurrentMenu::OptionMenu => self.settings_menu.draw(state, ctx)?,
            CurrentMenu::SaveSelectMenu => self.save_select_menu.draw(state, ctx)?,
            CurrentMenu::PlayerCountMenu => self.coop_menu.draw(state, ctx)?,
        }

        Ok(())
    }
}
