#[derive(Debug, Clone, PartialEq)]
pub enum MenuState {
    MainMenu,
    Credits,
    GameSelection,
    InGame(String), // Contains the path to the currently running game
}

#[derive(Debug, Clone)]
pub struct GameInfo {
    pub name: String,
    pub path: String,
    pub file_size: u64,
    pub is_battery_backed: bool,
}

pub struct MenuContext {
    pub current_state: MenuState,
    pub selected_main_option: usize, // 0 = Start, 1 = Credits
    pub selected_game_index: usize,
    pub games: Vec<GameInfo>,
    pub scroll_offset: usize,
    pub max_visible_games: usize,
    pub credits_scroll: f32,
    pub animation_time: f32,
    pub debug: bool,
}

impl MenuContext {
    pub fn new_with_debug(debug: bool) -> Self {
        MenuContext {
            current_state: MenuState::MainMenu,
            selected_main_option: 0,
            selected_game_index: 0,
            games: Vec::new(),
            scroll_offset: 0,
            max_visible_games: 12, // Increased from 8 to 12 for more vertical space
            credits_scroll: 0.0,
            animation_time: 0.0,
            debug,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.animation_time += delta_time;
        
        // No auto-scroll for credits anymore - they are now static
    }

    pub fn navigate_up(&mut self) {
        match self.current_state {
            MenuState::MainMenu => {
                if self.selected_main_option > 0 {
                    self.selected_main_option -= 1;
                }
            }
            MenuState::GameSelection => {
                if self.selected_game_index > 0 {
                    self.selected_game_index -= 1;
                    // Adjust scroll if needed
                    if self.selected_game_index < self.scroll_offset {
                        self.scroll_offset = self.selected_game_index;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn navigate_down(&mut self) {
        match self.current_state {
            MenuState::MainMenu => {
                if self.selected_main_option < 1 { // 0-1 (Start, Credits)
                    self.selected_main_option += 1;
                }
            }
            MenuState::GameSelection => {
                if self.selected_game_index < self.games.len().saturating_sub(1) {
                    self.selected_game_index += 1;
                    // Adjust scroll if needed
                    if self.selected_game_index >= self.scroll_offset + self.max_visible_games {
                        self.scroll_offset = self.selected_game_index + 1 - self.max_visible_games;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn select(&mut self) -> Option<String> {
        match self.current_state {
            MenuState::MainMenu => {
                match self.selected_main_option {
                    0 => { // Start
                        self.current_state = MenuState::GameSelection;
                        None
                    }
                    1 => { // Credits
                        self.current_state = MenuState::Credits;
                        self.credits_scroll = 0.0;
                        None
                    }
                    _ => None
                }
            }
            MenuState::GameSelection => {
                if let Some(game) = self.games.get(self.selected_game_index) {
                    let game_path = game.path.clone();
                    self.current_state = MenuState::InGame(game_path.clone());
                    Some(game_path)
                } else {
                    None
                }
            }
            _ => None
        }
    }

    pub fn back(&mut self) {
        match self.current_state {
            MenuState::Credits => {
                self.current_state = MenuState::MainMenu;
            }
            MenuState::GameSelection => {
                self.current_state = MenuState::MainMenu;
            }
            MenuState::InGame(_) => {
                self.current_state = MenuState::GameSelection;
            }
            _ => {}
        }
    }

    pub fn exit_game(&mut self) {
        if matches!(self.current_state, MenuState::InGame(_)) {
            self.current_state = MenuState::GameSelection;
        }
    }

    pub fn get_selected_game(&self) -> Option<&GameInfo> {
        self.games.get(self.selected_game_index)
    }

    pub fn get_visible_games(&self) -> Vec<(usize, &GameInfo)> {
        self.games
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(self.max_visible_games)
            .collect()
    }
} 