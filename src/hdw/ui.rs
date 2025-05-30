// ui.rs

use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};
use sdl2::VideoSubsystem;
use sdl2::EventPump;
use sdl2::surface::Surface;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use std::time::{SystemTime, UNIX_EPOCH};

pub const SCREEN_WIDTH: u32 = 1024;
pub const SCREEN_HEIGHT: u32 = 768;
// Tile viewer constants to match C code
pub const SCALE: u32 = 4;
pub const TILE_SIZE: u32 = 8;
pub const TILES_PER_ROW: u32 = 16;    // 16 tiles per row
pub const TILE_ROWS: u32 = 24;        // 24 rows
pub const TILE_PADDING: u32 = 16;     // Padding between tiles
// Window size calculation matching C code exactly
pub const TILE_VIEWER_WIDTH: u32 = (TILES_PER_ROW * TILE_SIZE * SCALE) + (TILES_PER_ROW * SCALE); // 16 * 8 * 4 + 16 * 4
pub const TILE_VIEWER_HEIGHT: u32 = (TILE_ROWS * TILE_SIZE * SCALE) + (TILE_ROWS * SCALE);        // 24 * 8 * 4 + 24 * 4

// Tile colors matching C code
const TILE_COLORS: [Color; 4] = [
    Color::RGBA(255, 255, 255, 255), // White
    Color::RGBA(170, 170, 170, 255), // Light gray
    Color::RGBA(85, 85, 85, 255),    // Dark gray
    Color::RGBA(0, 0, 0, 255),       // Black
];

pub struct UI {
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: VideoSubsystem,
    pub ttf_context: Sdl2TtfContext,
    pub main_canvas: WindowCanvas,
    pub tile_canvas: WindowCanvas,
    pub main_texture_creator: TextureCreator<WindowContext>,
    pub tile_texture_creator: TextureCreator<WindowContext>,
    pub event_pump: EventPump,
    pub main_surface: Surface<'static>,
    pub tile_surface: Surface<'static>,
}

impl UI {
    pub fn new() -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let event_pump = sdl_context.event_pump()?;

        println!("SDL INIT");

        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        println!("TTF INIT");

        // Main window
        let main_window = video_subsystem
            .window("GameBoy", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        // Get main window position for positioning tile viewer
        let (x, y) = main_window.position();

        // Tile viewer window
        let tile_window = video_subsystem
            .window("Tile Viewer", TILE_VIEWER_WIDTH, TILE_VIEWER_HEIGHT)
            .position(x + SCREEN_WIDTH as i32 + 10, y)
            .build()
            .map_err(|e| e.to_string())?;

        let main_canvas = main_window.into_canvas().build().map_err(|e| e.to_string())?;
        let tile_canvas = tile_window.into_canvas().build().map_err(|e| e.to_string())?;
        
        let main_texture_creator = main_canvas.texture_creator();
        let tile_texture_creator = tile_canvas.texture_creator();

        // Create surfaces
        let main_surface = Surface::new(SCREEN_WIDTH, SCREEN_HEIGHT, PixelFormatEnum::RGBA8888)
            .map_err(|e| e.to_string())?;
        let tile_surface = Surface::new(TILE_VIEWER_WIDTH, TILE_VIEWER_HEIGHT, PixelFormatEnum::RGBA8888)
            .map_err(|e| e.to_string())?;

        Ok(UI {
            sdl_context,
            video_subsystem,
            ttf_context,
            main_canvas,
            tile_canvas,
            main_texture_creator,
            tile_texture_creator,
            event_pump,
            main_surface,
            tile_surface,
        })
    }

    fn display_tile(&mut self, start_addr: u16, tile_num: u16, x: i32, y: i32, cpu: &mut super::cpu::CPU) {
        for tile_y in (0..16).step_by(2) {
            let b1 = cpu.bus.read_byte(Some(cpu), start_addr + (tile_num * 16) + tile_y as u16);
            let b2 = cpu.bus.read_byte(Some(cpu), start_addr + (tile_num * 16) + tile_y as u16 + 1);

            for bit in (0..=7).rev() {
                let hi = ((b1 & (1 << bit)) != 0) as u8 * 2;
                let lo = ((b2 & (1 << bit)) != 0) as u8;
                let color = hi | lo;

                let rect = Rect::new(
                    x + ((7 - bit) * SCALE as i32),
                    y + ((tile_y / 2) * SCALE as i32),
                    SCALE,
                    SCALE
                );

                self.tile_surface.fill_rect(rect, TILE_COLORS[color as usize]).unwrap();
            }
        }
    }

    pub fn update_debug_window(&mut self, cpu: &mut super::cpu::CPU) {
        // Clear debug screen with dark gray background
        self.tile_surface.fill_rect(None, Color::RGB(17, 17, 17)).unwrap();

        let mut tile_num = 0;
        let vram_start = 0x8000;

        // Draw 384 tiles (24 rows x 16 columns)
        for y in 0..TILE_ROWS {
            for x in 0..TILES_PER_ROW {
                let x_draw = (x * TILE_SIZE * SCALE + x * SCALE) as i32;
                let y_draw = (y * TILE_SIZE * SCALE + y * SCALE) as i32;
                
                self.display_tile(vram_start, tile_num, x_draw, y_draw, cpu);
                tile_num += 1;
            }
        }

        // Create texture from surface and render using tile texture creator
        let tile_texture = self.tile_texture_creator
            .create_texture_from_surface(&self.tile_surface)
            .expect("Failed to create tile texture");

        self.tile_canvas.clear();
        self.tile_canvas.copy(&tile_texture, None, None).unwrap();
        self.tile_canvas.present();
    }

    pub fn handle_events(&mut self, cpu: &mut super::cpu::CPU) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    return false;
                },
                Event::Window { win_event: sdl2::event::WindowEvent::Close, .. } => {
                    return false;
                },
                _ => {}
            }
        }
        
        // Update main window - set background to black
        self.main_canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.main_canvas.clear();
        
        // Draw Game Boy screen (same black color)
        let screen_rect = Rect::new(
            (SCREEN_WIDTH as i32 - 160 * 2) / 2, 
            (SCREEN_HEIGHT as i32 - 144 * 2) / 2,
            160 * 2,
            144 * 2
        );
        self.main_canvas.fill_rect(screen_rect).unwrap();
        
        // Update debug window with tiles
        self.update_debug_window(cpu);
        
        // Present main window
        self.main_canvas.present();
        
        true
    }
}

// Standalone functions
pub fn delay(ms: u32) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

pub fn get_ticks() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn ui_handle_events(cpu: &mut super::cpu::CPU) -> bool {
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} |
            Event::Window { win_event: sdl2::event::WindowEvent::Close, .. } => {
                if let Some(ctx_arc) = crate::hdw::emu::EMU_CONTEXT.get() {
                    ctx_arc.lock().unwrap().die = true;
                }
                return false;
            },
            _ => {}
        }
    }
    
    true
}
