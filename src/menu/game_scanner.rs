use std::fs;
use std::path::Path;
use crate::menu::GameInfo;

pub struct GameScanner;

impl GameScanner {
    pub fn scan_games(roms_directory: &str) -> Vec<GameInfo> {
        let mut games = Vec::new();
        
        // Scan the main roms directory
        if let Ok(entries) = fs::read_dir(roms_directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && Self::is_gameboy_rom(&path) {
                    if let Some(game_info) = Self::create_game_info(&path) {
                        games.push(game_info);
                    }
                }
            }
        }
        
        // Also scan game_roms subdirectory (but NOT test_roms)
        let subdirs = ["game_roms"];
        for subdir in &subdirs {
            let subdir_path = format!("{}/{}", roms_directory, subdir);
            if let Ok(entries) = fs::read_dir(&subdir_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && Self::is_gameboy_rom(&path) {
                        if let Some(game_info) = Self::create_game_info(&path) {
                            games.push(game_info);
                        }
                    }
                }
            }
        }
        
        // Sort games alphabetically
        games.sort_by(|a, b| a.name.cmp(&b.name));
        games
    }
    
    fn is_gameboy_rom(path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            ext == "gb" || ext == "gbc"
        } else {
            false
        }
    }
    
    fn create_game_info(path: &Path) -> Option<GameInfo> {
        let path_str = path.to_string_lossy().to_string();
        
        // Extract game name from filename
        let name = if let Some(filename) = path.file_stem() {
            let name_str = filename.to_string_lossy();
            Self::clean_rom_name(&name_str)
        } else {
            return None;
        };
        
        // Get file size
        let file_size = if let Ok(metadata) = path.metadata() {
            metadata.len()
        } else {
            0
        };
        
        // Determine if it's battery-backed by reading the cartridge header
        let is_battery_backed = Self::check_battery_backup(&path_str);
        
        Some(GameInfo {
            name,
            path: path_str,
            file_size,
            is_battery_backed,
        })
    }
    
    fn clean_rom_name(raw_name: &str) -> String {
        // Remove common ROM filename patterns and clean up the name
        let mut name = raw_name.to_string();
        
        // Remove file extension
        if let Some(pos) = name.rfind('.') {
            name = name[..pos].to_string();
        }
        
        // Remove region codes like (USA), (Europe), [!], etc.
        let patterns_to_remove = [
            r"\(USA\)",
            r"\(Europe\)", 
            r"\(Japan\)",
            r"\(World\)",
            r"\[!\]",
            r"\[T\+.*?\]",
            r"\[h.*?\]",
            r"\[o.*?\]",
            r"\[b.*?\]",
            r"\[f.*?\]",
            r"\[t.*?\]",
            r"\[T-.*?\]",
            r"\[T\+.*?\]",
            r"\[a.*?\]",
            r"\(Rev .*?\)",
            r"\(V.*?\)",
            r"Rev ",
            r"V\d+\.\d+",
        ];
        
        for pattern in &patterns_to_remove {
            // Simple string replacement (could use regex for more complex patterns)
            if pattern.contains("(USA)") {
                name = name.replace("(USA)", "");
            } else if pattern.contains("(Europe)") {
                name = name.replace("(Europe)", "");
            } else if pattern.contains("(Japan)") {
                name = name.replace("(Japan)", "");
            } else if pattern.contains("(World)") {
                name = name.replace("(World)", "");
            } else if pattern.contains("[!]") {
                name = name.replace("[!]", "");
            }
            // Add more replacements as needed
        }
        
        // Clean up extra spaces and dashes
        name = name.replace("  ", " ");
        name = name.replace(" - ", " ");
        name = name.replace(", The", "");
        name = name.trim().to_string();
        
        // Handle special cases
        if name.starts_with("Legend of Zelda") {
            name = name.replace("Legend of Zelda", "Zelda");
        }
        
        // Capitalize first letter of each word
        Self::title_case(&name)
    }
    
    fn title_case(s: &str) -> String {
        s.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    fn check_battery_backup(rom_path: &str) -> bool {
        // Read the cartridge type byte at offset 0x0147
        if let Ok(data) = fs::read(rom_path) {
            if data.len() > 0x0147 {
                let cart_type = data[0x0147];
                // Check for battery-backed cartridge types
                matches!(cart_type, 
                    0x03 | 0x06 | 0x09 | 0x0D | // MBC1+RAM+BATTERY, MBC2+BATTERY, ROM+RAM+BATTERY, MMM01+RAM+BATTERY
                    0x0F | 0x10 | 0x13 | // MBC3+TIMER+BATTERY, MBC3+TIMER+RAM+BATTERY, MBC3+RAM+BATTERY
                    0x1B | 0x1E | // MBC5+RAM+BATTERY, MBC5+RUMBLE+RAM+BATTERY
                    0x22 | 0xFF // MBC7+SENSOR+RUMBLE+RAM+BATTERY, HuC1+RAM+BATTERY
                )
            } else {
                false
            }
        } else {
            false
        }
    }
} 