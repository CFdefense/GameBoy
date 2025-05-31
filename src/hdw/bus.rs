/*

    Gameboy Memory Bus

    Most of the function here is to reroute referencing of data
    0x0000 - 0x3FFF : ROM Bank 0
    0x4000 - 0x7FFF : ROM Bank 1 - Switchable
    0x8000 - 0x97FF : CHR RAM
    0x9800 - 0x9BFF : BG Map 1
    0x9C00 - 0x9FFF : BG Map 2
    0xA000 - 0xBFFF : Cartridge RAM
    0xC000 - 0xCFFF : RAM Bank 0
    0xD000 - 0xDFFF : RAM Bank 1-7 - switchable - Color only
    0xE000 - 0xFDFF : Reserved - Echo RAM
    0xFE00 - 0xFE9F : Object Attribute Memory
    0xFEA0 - 0xFEFF : Reserved - Unusable
    0xFF00 - 0xFF7F : I/O Registers
    0xFF80 - 0xFFFE : Zero Page

*/

use super::cart::Cartridge;
use crate::hdw::cpu::CPU;
use crate::hdw::ram::RAM;
use crate::hdw::ppu::PPU;
use crate::hdw::dma::DMA;
use crate::hdw::interrupts::InterruptController;
use crate::hdw::gamepad::GamePad;
use crate::hdw::apu::AudioSystem;
use crate::hdw::io::{io_read,io_write};

pub struct BUS {
    pub cart: Cartridge,
    pub ram: RAM,
    pub ppu: PPU,
    pub apu: AudioSystem,
    pub gamepad: GamePad,
    pub interrupt_controller: InterruptController,
    pub dma: DMA,
}

impl BUS {
    // Constructor
    pub fn new() -> Self {
        BUS {
            cart: Cartridge::new(),
            ram: RAM::new(),
            ppu: PPU::new(),
            apu: AudioSystem::new(),
            gamepad: GamePad::new(),
            interrupt_controller: InterruptController::new(),
            dma: DMA::new(),
        }
    }

    // Function to return a byte at an address
    pub fn read_byte(&mut self, cpu: Option<&CPU>, address: u16) -> u8 {
        let value = match address {
            0x0000..=0x7FFF => self.cart.read_byte(address),  // Cartridge ROM
            0x8000..=0x9FFF => self.ppu.ppu_vram_read(address), // Video RAM
            0xA000..=0xBFFF => self.cart.read_byte(address),  // Cartridge RAM
            0xC000..=0xDFFF => self.ram.wram_read(address),   // WRAM
            0xE000..=0xFDFF => self.ram.wram_read(address),   // Work RAM (echo)
            0xFE00..=0xFE9F => {
                if self.dma.dma_transferring() {
                    0xFF
                } else {
                    self.ppu.ppu_oam_read(address)
                }
            }, // OAM
            0xFEA0..=0xFEFF => 0x00, // Unusable memory
            0xFF00..=0xFF7F => {
                io_read(cpu, address, &self.interrupt_controller, &self.ppu, &self.gamepad, &self.apu)
            }, // I/O registers
            0xFF80..=0xFFFE => self.ram.hram_read(address), // High RAM
            0xFFFF => self.interrupt_controller.get_ie_register(), // Interrupt Enable
        };
        
        value
    }

    // Function to write byte to correct place
    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => self.cart.write_byte(address, value),  // ROM Banks
            0x8000..=0x9FFF => {  // Char/Map Data
                self.ppu.ppu_vram_write(address, value)
            },
            0xA000..=0xBFFF => self.cart.write_byte(address, value),  // Cartridge RAM
            0xC000..=0xDFFF => self.ram.wram_write(address, value),   // WRAM
            0xE000..=0xFDFF => (),  // Reserved Echo RAM
            0xFE00..=0xFE9F => {    // OAM RAM
                if self.dma.dma_transferring() {
                    return;
                } else {
                    self.ppu.ppu_oam_write(address, value)
                }
            },
            0xFEA0..=0xFEFF => (),  // Reserved Unusable
            0xFF00..=0xFF7F => {    // IO Registers
                io_write(address, value, &mut self.dma, &mut self.interrupt_controller, &mut self.ppu, &mut self.gamepad, &mut self.apu);
            },
            0xFFFF => self.interrupt_controller.set_ie_register(value),    // Interrupt Enable Register
            _ => self.ram.hram_write(address, value)  // HRAM
        }
    }

    pub fn tick_dma(&mut self) {
        let mut dma = std::mem::take(&mut self.dma);
        if dma.dma_transferring() {
            dma.dma_tick(self);
        }
        self.dma = dma;
    }
}
