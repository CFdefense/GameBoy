// ui.rs

use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};
use sdl2::VideoSubsystem;
use sdl2::EventPump;

pub const SCREEN_WIDTH: u32 = 1024;
pub const SCREEN_HEIGHT: u32 = 768;

pub struct UI {
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: VideoSubsystem,
    pub ttf_context: Sdl2TtfContext,
    pub canvas: WindowCanvas,
    pub texture_creator: TextureCreator<WindowContext>,
    pub event_pump: EventPump,
}

impl UI {
    pub fn new() -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let event_pump = sdl_context.event_pump()?;

        println!("SDL INIT");

        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        println!("TTF INIT");

        let window = video_subsystem
            .window("Emulator", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();

        Ok(UI {
            sdl_context,
            video_subsystem,
            ttf_context,
            canvas,
            texture_creator,
            event_pump,
        })
    }

    // Create a texture using the stored texture_creator
    pub fn create_texture(&self, width: u32, height: u32) -> Result<Texture, String> {
        self.texture_creator
            .create_texture_target(PixelFormatEnum::RGBA8888, width, height)
            .map_err(|e| e.to_string())
    }

    pub fn delay(&self, ms: u32) {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
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
                // Add other event handling as needed
                _ => {}
            }
        }
        
        // Clear the screen with a color
        self.canvas.set_draw_color(sdl2::pixels::Color::RGB(200, 200, 255));
        self.canvas.clear();
        
        // Draw a simple rectangle to represent the Game Boy screen
        self.canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        let screen_rect = sdl2::rect::Rect::new(
            (SCREEN_WIDTH as i32 - 160 * 2) / 2, 
            (SCREEN_HEIGHT as i32 - 144 * 2) / 2,
            160 * 2,  // Game Boy screen width * 2
            144 * 2   // Game Boy screen height * 2
        );
        self.canvas.fill_rect(screen_rect).unwrap();
        
        // Present the canvas
        self.canvas.present();
        
        true
    }
}

// Standalone functions for compatibility with C-style code
pub fn delay(ms: u32) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

pub fn ui_handle_events(cpu: &mut super::cpu::CPU) -> bool {
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} |
            Event::Window { win_event: sdl2::event::WindowEvent::Close, .. } => {
                return false;
            },
            _ => {}
        }
    }
    
    // TODO: Update window surfaces when implemented
    
    true
}
