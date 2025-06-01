#[derive(Debug, Clone, PartialEq)]
pub enum MenuState {
    MainMenu,
    Credits,
    GameSelection,
    PaletteSelection,
    InGame(String), // Contains the path to the currently running game
}

#[derive(Debug, Clone)]
pub struct GameInfo {
    pub name: String,
    pub path: String,
    pub file_size: u64,
    pub is_battery_backed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorPalette {
    ClassicGreen,
    Grayscale,
    PurpleShades,
    BlueShades,
    Sepia,
    RedShades,
    CyberpunkGreen,
    Ocean,
}

impl ColorPalette {
    pub fn get_colors(&self) -> [u32; 4] {
        match self {
            ColorPalette::ClassicGreen => [
                0xFF9BBB0F,  // Light green
                0xFF8BAC0F,  // Medium green
                0xFF306230,  // Dark green
                0xFF0F380F,  // Very dark green
            ],
            ColorPalette::Grayscale => [
                0xFFFFFFFF,  // White
                0xFFAAAAAA,  // Light gray
                0xFF555555,  // Dark gray
                0xFF000000,  // Black
            ],
            ColorPalette::PurpleShades => [
                0xFFE6E6FA,  // Lavender
                0xFFDDA0DD,  // Plum
                0xFF9370DB,  // Medium slate blue
                0xFF4B0082,  // Indigo
            ],
            ColorPalette::BlueShades => [
                0xFFE0F6FF,  // Alice blue
                0xFF87CEEB,  // Sky blue
                0xFF4682B4,  // Steel blue
                0xFF191970,  // Midnight blue
            ],
            ColorPalette::Sepia => [
                0xFFFFF8DC,  // Cornsilk
                0xFFDEB887,  // Burlywood
                0xFFCD853F,  // Peru
                0xFF8B4513,  // Saddle brown
            ],
            ColorPalette::RedShades => [
                0xFFFFE4E1,  // Misty rose
                0xFFFF6B6B,  // Light red
                0xFFDC143C,  // Crimson
                0xFF8B0000,  // Dark red
            ],
            ColorPalette::CyberpunkGreen => [
                0xFF00FF41,  // Bright neon green
                0xFF00CC33,  // Medium neon green
                0xFF008F11,  // Dark neon green
                0xFF003300,  // Very dark green
            ],
            ColorPalette::Ocean => [
                0xFFF0F8FF,  // Alice blue
                0xFF00CED1,  // Dark turquoise
                0xFF008B8B,  // Dark cyan
                0xFF2F4F4F,  // Dark slate gray
            ],
        }
    }
    
    pub fn get_name(&self) -> &'static str {
        match self {
            ColorPalette::ClassicGreen => "Classic Game Boy",
            ColorPalette::Grayscale => "Grayscale",
            ColorPalette::PurpleShades => "Purple Dreams",
            ColorPalette::BlueShades => "Ocean Blue",
            ColorPalette::Sepia => "Vintage Sepia",
            ColorPalette::RedShades => "Ruby Red",
            ColorPalette::CyberpunkGreen => "Cyberpunk",
            ColorPalette::Ocean => "Deep Ocean",
        }
    }
    
    pub fn all_palettes() -> Vec<ColorPalette> {
        vec![
            ColorPalette::ClassicGreen,
            ColorPalette::Grayscale,
            ColorPalette::PurpleShades,
            ColorPalette::BlueShades,
            ColorPalette::Sepia,
            ColorPalette::RedShades,
            ColorPalette::CyberpunkGreen,
            ColorPalette::Ocean,
        ]
    }
}

pub struct MenuContext {
    pub current_state: MenuState,
    pub selected_main_option: usize, // 0 = Start, 1 = Palette, 2 = Credits
    pub selected_game_index: usize,
    pub selected_palette_index: usize,
    pub current_palette: ColorPalette,
    pub available_palettes: Vec<ColorPalette>,
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
            selected_palette_index: 0,
            current_palette: ColorPalette::ClassicGreen,
            available_palettes: ColorPalette::all_palettes(),
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
            MenuState::PaletteSelection => {
                if self.selected_palette_index > 0 {
                    self.selected_palette_index -= 1;
                }
            }
            _ => {}
        }
    }

    pub fn navigate_down(&mut self) {
        match self.current_state {
            MenuState::MainMenu => {
                if self.selected_main_option < 2 { // 0-2 (Start, Palette, Credits)
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
            MenuState::PaletteSelection => {
                if self.selected_palette_index < self.available_palettes.len().saturating_sub(1) {
                    self.selected_palette_index += 1;
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
                    1 => { // Palette
                        self.current_state = MenuState::PaletteSelection;
                        None
                    }
                    2 => { // Credits
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
            MenuState::PaletteSelection => {
                if let Some(palette) = self.available_palettes.get(self.selected_palette_index) {
                    self.current_palette = palette.clone();
                    println!("Selected palette: {}", palette.get_name());
                }
                None
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
            MenuState::PaletteSelection => {
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
    
    pub fn get_current_palette(&self) -> &ColorPalette {
        &self.current_palette
    }
    
    pub fn get_selected_palette(&self) -> Option<&ColorPalette> {
        self.available_palettes.get(self.selected_palette_index)
    }
} 