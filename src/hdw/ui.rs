// UI module for Game Boy emulator
// Handles SDL2-based graphical user interface, including main game display and debug tile viewer

use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{WindowContext};
use sdl2::VideoSubsystem;
use sdl2::EventPump;
use sdl2::surface::Surface;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::keyboard::Keycode;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::hdw::cpu::CPU;

// Main emulator window dimensions - provides plenty of space for the scaled Game Boy display
pub const SCREEN_WIDTH: u32 = 1024;
pub const SCREEN_HEIGHT: u32 = 768;

// Game Boy screen resolution - the actual LCD dimensions
pub const XRES: u32 = 160;
pub const YRES: u32 = 144;

// Scale factor for pixel upscaling - makes the 160x144 display visible on modern screens
const SCALE: u32 = 4;

// Debug window shows VRAM tile data in a 16x24 grid (384 tiles total)
// Each tile is 8x8 pixels, scaled up by the scale factor
pub const DEBUG_WINDOW_WIDTH: u32 = 16 * 8 * SCALE;
pub const DEBUG_WINDOW_HEIGHT: u32 = 32 * 8 * SCALE;

// Debug surface is slightly larger to accommodate padding and additional info
pub const DEBUG_SURFACE_WIDTH: u32 = (16 * 8 * SCALE) + (16 * SCALE);
pub const DEBUG_SURFACE_HEIGHT: u32 = (32 * 8 * SCALE) + (64 * SCALE);

// Color palette for tile display in debug viewer
// Represents the 4 possible Game Boy colors from white to black
const TILE_COLORS: [u32; 4] = [0xFFFFFFFF, 0xFFAAAAAA, 0xFF555555, 0xFF000000];

pub struct UI {
    // Core SDL2 components
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: VideoSubsystem,
    pub ttf_context: Sdl2TtfContext,
    
    // Rendering contexts for main game window and debug tile viewer
    pub main_canvas: WindowCanvas,
    pub debug_canvas: WindowCanvas,
    
    // Texture creators for efficient rendering
    pub main_texture_creator: TextureCreator<WindowContext>,
    pub debug_texture_creator: TextureCreator<WindowContext>,
    
    // Event handling for user input
    pub event_pump: EventPump,
    
    // Frame buffers - surfaces hold pixel data before rendering to screen
    pub screen_surface: Surface<'static>,
    pub debug_surface: Surface<'static>,
}

impl UI {
    pub fn new() -> Result<Self, String> {
        // Initialize SDL2 video subsystem
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let event_pump = sdl_context.event_pump()?;

        println!("SDL INIT");

        // Initialize SDL2 TTF for text rendering (though not currently used)
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        println!("TTF INIT");

        // Create main emulator window - centered on screen
        let main_window = video_subsystem
            .window("GameBoy", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        // Get main window position to place debug window adjacent to it
        let (x, y) = main_window.position();

        // Create debug tile viewer window - positioned to the right of main window
        let debug_window = video_subsystem
            .window("Debug Viewer", DEBUG_WINDOW_WIDTH, DEBUG_WINDOW_HEIGHT)
            .position(x + SCREEN_WIDTH as i32 + 10, y)
            .build()
            .map_err(|e| e.to_string())?;

        // Convert windows to canvas objects for 2D rendering
        let main_canvas = main_window.into_canvas().build().map_err(|e| e.to_string())?;
        let debug_canvas = debug_window.into_canvas().build().map_err(|e| e.to_string())?;
        
        // Create texture creators for efficient GPU-accelerated rendering
        let main_texture_creator = main_canvas.texture_creator();
        let debug_texture_creator = debug_canvas.texture_creator();

        // Create RGB surface for main display - ARGB8888 format for full color support
        let screen_surface = Surface::new(SCREEN_WIDTH, SCREEN_HEIGHT, PixelFormatEnum::ARGB8888)
            .map_err(|e| e.to_string())?;

        // Create surface for debug tile display with extra space for padding
        let debug_surface = Surface::new(DEBUG_SURFACE_WIDTH, DEBUG_SURFACE_HEIGHT, PixelFormatEnum::ARGB8888)
            .map_err(|e| e.to_string())?;

        Ok(UI {
            sdl_context,
            video_subsystem,
            ttf_context,
            main_canvas,
            debug_canvas,
            main_texture_creator,
            debug_texture_creator,
            event_pump,
            screen_surface,
            debug_surface,
        })
    }

    /// Renders a single 8x8 tile from VRAM to the debug surface
    /// Each tile consists of 16 bytes (2 bytes per 8-pixel row)
    /// The two bytes form bit planes that combine to create 2-bit color values (0-3)
    fn display_tile(&mut self, start_location: u16, tile_num: u16, x: i32, y: i32, cpu: &mut super::cpu::CPU) {
        // Process each row of the tile (8 rows total, 2 bytes per row)
        for tile_y in (0..16).step_by(2) {
            // Calculate addresses for the two bit planes of this row
            let addr1 = start_location + (tile_num * 16) + tile_y as u16;
            let addr2 = start_location + (tile_num * 16) + tile_y as u16 + 1;
            
            // Ensure we're reading from valid VRAM range to prevent crashes
            if addr1 >= 0x8000 && addr1 <= 0x9FFF && addr2 >= 0x8000 && addr2 <= 0x9FFF {
                // Read the two bit planes for this row
                let b1 = cpu.bus.read_byte(None, addr1);
                let b2 = cpu.bus.read_byte(None, addr2);

                // Process each pixel in the row (8 pixels, from bit 7 down to bit 0)
                for bit in (0..=7).rev() {
                    // Extract bit from each plane and combine to form 2-bit color index
                    let hi = ((b1 & (1 << bit)) != 0) as u8 * 2;  // High bit contributes 2 to value
                    let lo = ((b2 & (1 << bit)) != 0) as u8;       // Low bit contributes 1 to value
                    let color = hi | lo;  // Combine to get color index (0-3)

                    // Calculate pixel position on screen with scaling
                    let rect = Rect::new(
                        x + ((7 - bit) * SCALE as i32),        // X position (left to right)
                        y + ((tile_y / 2) * SCALE as i32),     // Y position (top to bottom)
                        SCALE,                                  // Width of scaled pixel
                        SCALE                                   // Height of scaled pixel
                    );

                    // Fill the scaled pixel rectangle with the appropriate color
                    if (color as usize) < TILE_COLORS.len() {
                        let color_value = TILE_COLORS[color as usize];
                        self.debug_surface.fill_rect(rect, Color::RGBA(
                            ((color_value >> 16) & 0xFF) as u8,  // Red component
                            ((color_value >> 8) & 0xFF) as u8,   // Green component
                            (color_value & 0xFF) as u8,          // Blue component
                            ((color_value >> 24) & 0xFF) as u8,  // Alpha component
                        )).unwrap();
                    }
                }
            }
        }
    }

    /// Updates the debug window showing all tiles in VRAM
    /// Displays 384 tiles in a 16x24 grid layout
    pub fn update_dbg_window(&mut self, cpu: &mut super::cpu::CPU) {
        let mut x_draw = 0;
        let mut y_draw = 0;
        let mut tile_num = 0;

        // Clear debug surface with dark gray background
        self.debug_surface.fill_rect(None, Color::RGBA(0x11, 0x11, 0x11, 0xFF)).unwrap();

        // Start from VRAM tile data area
        let addr = 0x8000;

        // Render all 384 tiles in a 16x24 grid
        for y in 0..24 {
            for x in 0..16 {
                // Render individual tile at calculated position
                self.display_tile(addr, tile_num, x_draw + (x * SCALE as i32), y_draw + (y * SCALE as i32), cpu);
                // Move to next horizontal tile position
                x_draw += (8 * SCALE) as i32;
                // Move to next tile number
                tile_num += 1;
            }
            // Move to next row of tiles
            y_draw += (8 * SCALE) as i32;
            // Reset horizontal position for new row
            x_draw = 0;
        }

        // Create texture from surface and render to debug window
        let debug_texture = self.debug_texture_creator
            .create_texture_from_surface(&self.debug_surface)
            .expect("Failed to create debug texture");
        
        self.debug_canvas.clear();
        self.debug_canvas.copy(&debug_texture, None, None).unwrap();
        self.debug_canvas.present();
    }

    /// Updates the main game display window
    /// Renders the PPU's video buffer to screen with pixel scaling
    pub fn ui_update(&mut self, cpu: &mut super::cpu::CPU) {
        // Update debug window first to avoid borrow conflicts
        self.update_dbg_window(cpu);

        // Render each pixel from the Game Boy's video buffer to the main display
        for line_num in 0..YRES {
            for x in 0..XRES {
                // Calculate scaled pixel rectangle
                let rect = Rect::new(
                    (x * SCALE) as i32,         // Scaled X position
                    (line_num * SCALE) as i32,  // Scaled Y position
                    SCALE,                      // Scaled width
                    SCALE                       // Scaled height
                );

                // Get pixel color from PPU video buffer
                let buffer_index = (x + (line_num * XRES)) as usize;
                if buffer_index < cpu.bus.ppu.video_buffer.len() {
                    let pixel_color = cpu.bus.ppu.video_buffer[buffer_index];
                    // Draw scaled pixel with the color from video buffer
                    self.screen_surface.fill_rect(rect, Color::RGBA(
                        ((pixel_color >> 16) & 0xFF) as u8,  // Red component
                        ((pixel_color >> 8) & 0xFF) as u8,   // Green component
                        (pixel_color & 0xFF) as u8,          // Blue component
                        ((pixel_color >> 24) & 0xFF) as u8,  // Alpha component
                    )).unwrap();
                }
            }
        }

        // Create texture from surface and render to main window
        let main_texture = self.main_texture_creator
            .create_texture_from_surface(&self.screen_surface)
            .expect("Failed to create main texture");
        
        self.main_canvas.clear();
        self.main_canvas.copy(&main_texture, None, None).unwrap();
        self.main_canvas.present();
    }

    /// Maps keyboard input to gamepad buttons
    /// Returns true if the key is mapped to a gamepad button
    fn ui_on_key(cpu: &mut CPU, down: bool, key_code: Keycode) -> bool {
        match key_code {
            Keycode::Z => {
                cpu.bus.gamepad.state.b = down;
                true
            },
            Keycode::X => {
                cpu.bus.gamepad.state.a = down;
                true
            },
            Keycode::Return => {
                cpu.bus.gamepad.state.start = down;
                true
            },
            Keycode::Tab => {
                cpu.bus.gamepad.state.select = down;
                true
            },
            Keycode::Up => {
                cpu.bus.gamepad.state.up = down;
                true
            },
            Keycode::Down => {
                cpu.bus.gamepad.state.down = down;
                true
            },
            Keycode::Left => {
                cpu.bus.gamepad.state.left = down;
                true
            },
            Keycode::Right => {
                cpu.bus.gamepad.state.right = down;
                true
            },
            _ => false
        }
    }

    /// Handles SDL events and updates the display
    /// Returns false if the application should quit, true otherwise
    pub fn handle_events(&mut self, cpu: &mut CPU) -> bool {
        // Process all pending SDL events
        for event in self.event_pump.poll_iter() {
            match event {
                // Handle quit events (X button, Alt+F4, etc.)
                Event::Quit {..} => {
                    return false;
                },
                // Handle window close events
                Event::Window { win_event: sdl2::event::WindowEvent::Close, .. } => {
                    return false;
                },
                // Handle key down events
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    Self::ui_on_key(cpu, true, keycode);
                },
                // Handle key up events
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    Self::ui_on_key(cpu, false, keycode);
                },
                // Ignore other events for now
                _ => {}
            }
        }
        
        // Update the display after processing events
        self.ui_update(cpu);
        
        // Continue running
        true
    }
}

/// Cross-platform delay function using standard library sleep
pub fn delay(ms: u32) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

/// Get current time in milliseconds since Unix epoch
/// Used for frame timing and FPS calculations
pub fn get_ticks() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
