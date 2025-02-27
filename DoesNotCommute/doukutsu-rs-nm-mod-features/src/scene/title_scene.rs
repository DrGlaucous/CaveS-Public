use core::mem;

use crate::common::{Color, FadeState, Rect, VERSION_BANNER};
use crate::components::background::Background;
use crate::components::compact_jukebox::CompactJukebox;
use crate::components::nikumaru::NikumaruCounter;
use crate::entity::GameEntity;
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
//use crate::menu::save_select_menu::SaveSelectMenu;
use crate::menu::stage_select_menu::StageSelectMenu;
use crate::menu::settings_menu::SettingsMenu;
use crate::menu::{Menu, MenuEntry, MenuSelectionResult};
use crate::scene::jukebox_scene::JukeboxScene;
use crate::scene::Scene;
use super::game_scene::{ GameScene, GameMode };
use crate::game::scripting::tsc::text_script::TextScriptExecutionState;

#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
#[allow(unused)]
pub enum CurrentMenu {
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
    // Item0,
    // Item1,
    // Item2,
    // Item3,
    // Item4,
    // Item5,
    // Item6,
    // Item7,
    // Item8,
    // Item9,
    // Item10,
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
    pub current_menu: CurrentMenu,
    main_menu: Menu<MainMenuEntry>,
    //save_select_menu: SaveSelectMenu,
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

    stage_select_menu: StageSelectMenu,

    pub game_scene: Option<Box<GameScene>>,
}

impl TitleScene {
    pub fn new(state: &mut SharedGameState, ctx: &mut Context) -> Self {
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

        //prepare the background scene
        let start_stage_id = state.constants.game.title_stage as usize;
        let bk_stage = if state.stages.len() < start_stage_id {
            log::warn!("Intro scene out of bounds in stage table, not using live background");
            None            
        } else {
            if let Ok(mut next_scene) = GameScene::new(state, ctx, start_stage_id) {

                next_scene.player1.cond.set_hidden(true);
                let (pos_x, pos_y) = state.constants.game.title_player_pos;
                next_scene.player1.x = pos_x as i32 * next_scene.stage.map.tile_size.as_int() * 0x200;
                next_scene.player1.y = pos_y as i32 * next_scene.stage.map.tile_size.as_int() * 0x200;
                next_scene.mode = GameMode::Title;

                //TSC vm initialized in the 'init' section far below

                Some(Box::new(next_scene))
            } else {
                None
            }
        };

        Self {
            tick: 0,
            controller: CombinedMenuController::new(),
            current_menu: CurrentMenu::MainMenu,
            main_menu: Menu::new(0, 0, 100, 0),
            //save_select_menu: SaveSelectMenu::new(),
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

            stage_select_menu: StageSelectMenu::new(),
            game_scene: bk_stage
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
            song_id = 24;
        }

        if state.settings.soundtrack == "New" && Season::current() == Season::PixelBirthday {
            song_id = 43;
        }

        if self.compact_jukebox.is_shown() {
            self.compact_jukebox.change_song(song_id, state, ctx)?;
        } else {
            let cur_sid = state.sound_manager.current_song();
            if song_id != cur_sid.id
            || cur_sid.loaded_from_path == true {
                //we'll be using our own songs, thank you.
                //state.sound_manager.play_song(song_id, &state.constants, &state.settings, ctx, false)?;
            }
        }

        Ok(())
    }

    pub fn open_settings_menu(&mut self) -> GameResult {
        self.current_menu = CurrentMenu::OptionMenu;
        Ok(())
    }


    fn draw_tv_perimeter(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {

        //state.canvas_size.0;
        let corner_rc: [Rect<u16>; 4] = [
            Rect::new(0, 48, 48, 96), //LT
            Rect::new(64, 48, 112, 96), //RT
            Rect::new(0, 112, 48, 176), //LB
            Rect::new(64, 112, 112, 176), //RB
        ];

        let edge_rc: [Rect<u16>; 4] = [
            Rect::new(0, 96, 40, 112), //L
            Rect::new(48, 48, 64, 88), //T
            Rect::new(72, 96, 112, 112), //R
            Rect::new(48, 120, 64, 176), //B
        ];

        let speaker_rc: [Rect<u16>; 4] = [
            Rect::new(112, 104, 128, 152), //L
            Rect::new(128, 104, 144, 152), //C
            Rect::new(144, 104, 160, 152), //R
            Rect::new(160, 104, 176, 136), //Fabric
        ];
        

        let logo_rc: Rect<u16> = Rect::new(144, 32, 296, 56);

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Title")?;

        //draw edges
        {
            //left and right edge
            let vert_edge_count = (state.canvas_size.1 / edge_rc[0].height() as f32).floor() as u16;
            for i in 0..vert_edge_count {
                //draw left edge
                batch.add_rect(
                    0.0,
                    (edge_rc[0].height() * i) as f32,
                    &edge_rc[0],
                );
                //draw right edge
                batch.add_rect(
                    state.canvas_size.0 - edge_rc[2].width() as f32,
                    (edge_rc[2].height() * i) as f32,
                    &edge_rc[2],
                );
            }
        
            //top and bottom edge
            let horiz_edge_count = (state.canvas_size.0 / edge_rc[1].width() as f32).floor() as u16;
            for i in 0..horiz_edge_count {
                //draw top edge
                batch.add_rect(
                    (edge_rc[1].width() * i) as f32,
                    0.0,
                    &edge_rc[1],
                );
                //draw bottom edge
                batch.add_rect(
                    (edge_rc[3].width() * i) as f32,
                    state.canvas_size.1 - edge_rc[3].height() as f32,
                    &edge_rc[3],
                );
            }
        }

        //draw corners
        {
            batch.add_rect(
                0.0,
                0.0,
                &corner_rc[0], //LT
            );
            batch.add_rect(
                state.canvas_size.0 - corner_rc[1].width() as f32,
                0.0,
                &corner_rc[1], //RT
            );
            batch.add_rect(
                0.0,
                state.canvas_size.1 - corner_rc[2].height() as f32,
                &corner_rc[2], //LB
            );
            batch.add_rect(
                state.canvas_size.0 - corner_rc[3].width() as f32,
                state.canvas_size.1 - corner_rc[3].height() as f32,
                &corner_rc[3], //RB
            );
        }

        //draw logo
        {
            batch.add_rect(
                ((state.canvas_size.0 - logo_rc.width() as f32) / 2.0).floor(),
                8.0,
                &logo_rc,
            );
        }


        //draw speaker grille
        {

            let edge_margin = 32.0;
            let inner_width = state.canvas_size.0 - (edge_margin * 2.0);

            let inner_repeat_ct = inner_width as u16 / speaker_rc[1].width();

            for i in 0..inner_repeat_ct {
                batch.add_rect(
                    edge_margin + (i * speaker_rc[1].width()) as f32 + (inner_width - (inner_repeat_ct * speaker_rc[1].width()) as f32) / 2.0,
                    state.canvas_size.1 - speaker_rc[1].height() as f32,
                    if i == 0 {&speaker_rc[0]}
                    else if i == (inner_repeat_ct - 1) {&speaker_rc[2]}
                    else {&speaker_rc[1]},
                );
            }
        }



        batch.draw(ctx)?;

        Ok(())

    }


}

static COPYRIGHT_PIXEL: &str = "2024.8  Dr. G";
// Freeware
static COPYRIGHT_NICALIS: &str = "2024.8  Dr. G"; // Nicalis font uses @ for copyright

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


        // //test: fill menu with dummy entries
        // self.main_menu.push_entry(MainMenuEntry::Item0, MenuEntry::Active(format!("Item 0")));
        // self.main_menu.push_entry(MainMenuEntry::Item1, MenuEntry::Active(format!("Item 1")));
        // self.main_menu.push_entry(MainMenuEntry::Item2, MenuEntry::Active(format!("Item 2")));
        // self.main_menu.push_entry(MainMenuEntry::Item3, MenuEntry::Active(format!("Item 3")));
        // self.main_menu.push_entry(MainMenuEntry::Item4, MenuEntry::Active(format!("Item 4")));
        // self.main_menu.push_entry(MainMenuEntry::Item5, MenuEntry::Active(format!("Item 5")));
        // self.main_menu.push_entry(MainMenuEntry::Item6, MenuEntry::Active(format!("Item 6")));
        // self.main_menu.push_entry(MainMenuEntry::Item7, MenuEntry::Active(format!("Item 7")));
        // self.main_menu.push_entry(MainMenuEntry::Item8, MenuEntry::Active(format!("Item 8")));
        // self.main_menu.push_entry(MainMenuEntry::Item9, MenuEntry::Active(format!("Item 9")));
        // self.main_menu.push_entry(MainMenuEntry::Item10, MenuEntry::Active(format!("Item 10")));


        self.settings_menu.init(state, ctx)?;

        self.stage_select_menu.init(state, ctx)?;

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

        if let Some(subscene) = &mut self.game_scene {
            subscene.init(state, ctx)?;

            //we need to init the TSC here because doing it in new() will result in it being run before its ready
            state.reset_map_flags();
            state.fade_state = FadeState::Hidden;
            state.textscript_vm.state = TextScriptExecutionState::Running(state.constants.game.title_event, 0);

        }


        Ok(())
    }

    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {   
        state.touch_controls.control_type = TouchControlType::None;

        //self.background.tick(state, &self.stage, &self.frame)?;
        if let Some(subscene) = &mut self.game_scene {
            subscene.tick(state, ctx)?;

            //swap scenes if we hit a TRA
            if let Some(_) = &state.next_title_subscene {
                self.game_scene = mem::take(&mut state.next_title_subscene);
                self.game_scene.as_mut().unwrap().mode = GameMode::Title;
                self.game_scene.as_mut().unwrap().init(state, ctx).unwrap();
                
            }
        }

        self.controller.update(state, ctx)?;
        self.controller.update_trigger();

        self.main_menu.update_width(state);
        self.main_menu.update_height(state);
        self.main_menu.x = ((state.canvas_size.0 - self.main_menu.width as f32) / 2.0).floor() as isize;
        self.main_menu.y = ((state.canvas_size.1 - self.main_menu.height as f32) / 2.0).floor() as isize;

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

                    //only change if we aren't locked in a <KEY (mid-background transition)
                    if state.control_flags.control_enabled() {
                        state.textscript_vm.state = TextScriptExecutionState::Running(state.constants.game.title_go_event, 0);
                        self.current_menu = CurrentMenu::SaveSelectMenu;
                    }
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
                // self.save_select_menu.tick(
                //     &mut || {
                //         *cm = rm;
                //     },
                //     &mut self.controller,
                //     state,
                //     ctx,
                // )?;

                if let Some(game_scene) = &mut self.game_scene {
                    self.stage_select_menu.tick(
                        &mut || {
                            *cm = rm;
                        },
                        &mut self.controller,
                        state,
                        ctx,
                        game_scene,
                    )?;
                }

            }
            CurrentMenu::ChallengesMenu => match self.challenges_menu.tick(&mut self.controller, state) {
                MenuSelectionResult::Selected(ChallengesMenuEntry::Challenge(idx), _) => {
                    if let Some(mod_info) = state.mod_list.mods.get(idx) {
                        state.mod_path = Some(mod_info.path.clone());
                        if mod_info.save_slot >= 0 {
                            //self.save_select_menu.init(state, ctx)?;
                            //self.save_select_menu.set_skip_difficulty_menu(true);
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
        //self.background.draw(state, ctx, &self.frame, &self.textures, &self.stage, false)?;
        if let Some(subscene) = &self.game_scene {
            subscene.draw(state, ctx)?;
        }


        if self.current_menu == CurrentMenu::MainMenu {
            // let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Title")?;
            // let logo_x_offset =
            //     if state.settings.original_textures && state.constants.supports_og_textures { 20.0 } else { 0.0 };

            // batch.add_rect(
            //     ((state.canvas_size.0 - state.constants.title.logo_rect.width() as f32) / 2.0).floor() + logo_x_offset,
            //     40.0,
            //     &state.constants.title.logo_rect,
            // );
            // batch.add_rect(
            //     ((state.canvas_size.0 - state.constants.title.logo_splash_rect.width() as f32) / 2.0).floor() + 72.0,
            //     88.0,
            //     &state.constants.title.logo_splash_rect,
            // );
            // batch.draw(ctx)?;
        } else {
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
                .position(0.0, state.font.line_height() + 48.0)
                .center(state.canvas_size.0)
                .draw(&window_title, ctx, &state.constants, &mut state.texture_set)?;
        }


        if self.current_menu == CurrentMenu::MainMenu {
            self.draw_text_centered(&VERSION_BANNER, state.canvas_size.1 - 85.0, state, ctx)?;

            if state.constants.is_cs_plus {
                self.draw_text_centered(COPYRIGHT_NICALIS, state.canvas_size.1 - 70.0, state, ctx)?;
            } else {
                self.draw_text_centered(COPYRIGHT_PIXEL, state.canvas_size.1 - 70.0, state, ctx)?;
            }

            self.compact_jukebox.draw(state, ctx, &self.frame)?;
        }

        //self.nikumaru_rec.draw(state, ctx, &self.frame)?;


        //40 px bezel,
        //56 + 40 px top and bottom
        let custom_height_margin = Some(
            (44.0, //40
            state.canvas_size.1 - 60.0) //56
        );

        match self.current_menu {
            CurrentMenu::MainMenu => self.main_menu.draw(state, ctx, custom_height_margin)?,
            CurrentMenu::ChallengesMenu => self.challenges_menu.draw(state, ctx, custom_height_margin)?,
            CurrentMenu::ChallengeConfirmMenu => self.confirm_menu.draw(state, ctx, custom_height_margin)?,
            CurrentMenu::OptionMenu => self.settings_menu.draw(state, ctx, custom_height_margin)?,
            CurrentMenu::PlayerCountMenu => self.coop_menu.draw(state, ctx, custom_height_margin)?,

            CurrentMenu::SaveSelectMenu => self.stage_select_menu.draw(
                state, 
                ctx, 
                custom_height_margin, 
                Rect::new(
                    40.0, 
                    40.0, 
                    state.canvas_size.0 - 40.0, 
                    state.canvas_size.1 - 56.0),
                    self.game_scene.as_ref().unwrap(),
            )?,

        }


        self.draw_tv_perimeter(state, ctx)?;

        Ok(())
    }
}
