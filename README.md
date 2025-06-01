# RustedROM - A Complete Gameboy Emulator

A cycle-accurate Game Boy emulator written in Rust that faithfully recreates the original 1989 Nintendo Game Boy hardware, supporting the complete library of Game Boy games with accurate timing, graphics, audio, and save functionality.

## Demo
<div align="center">
    <img src="https://github.com/CFdefense/RustedROM/blob/master/docs/demo.gif" alt="Description of the GIF" width="750">
</div>

## Overview

This Game Boy emulator provides a complete hardware-level emulation of Nintendo's iconic handheld gaming console. Built with accuracy and performance in mind, it recreates every aspect of the original Game Boy hardware including the custom Z80-like CPU, Picture Processing Unit, Audio Processing Unit, and all supported Memory Bank Controllers.

The emulator supports the entire Game Boy library with features like battery-backed saves, real-time clock functionality, and debugging capabilities, making it suitable for both gaming and Game Boy development research.

## Key Features

- **Complete CPU Emulation**
  - Full Sharp SM83 (Game Boy CPU) instruction set implementation
  - Cycle-accurate timing with T-cycle precision
  - Complete register set and flag handling
  - Interrupt system with priority handling
  - Debug mode with instruction logging and step execution

- **Graphics System (PPU)**
  - Complete LCD controller emulation
  - Background and window layer rendering
  - Sprite system with priority handling
  - Hardware scrolling and window positioning
  - V-blank and LCD status interrupts
  - Accurate timing for all graphics operations

- **Audio System (APU)**
  - Four-channel audio synthesis
  - Square wave channels with frequency sweep
  - Wave pattern channel with custom waveforms
  - Noise channel with linear feedback shift register
  - Real-time audio output with SDL2

- **Memory Management**
  - Complete memory map implementation
  - Memory Bank Controller support (MBC1, MBC2, MBC3)
  - ROM banking up to 127 banks
  - External RAM banking with battery backup
  - Real-time clock support for MBC3 cartridges

- **Save System**
  - Battery-backed SRAM persistence
  - Automatic save detection and loading
  - Save state management
  - Real-time clock data preservation
  - Cross-session game progress retention

- **Input and Controls**
  - Complete Game Boy button mapping
  - Keyboard input handling
  - Responsive control system
  - Exit and menu functionality

- **ROM Support**
  - Comprehensive cartridge header parsing
  - Checksum validation
  - Publisher and game information display
  - Support for all standard Game Boy ROM formats
  - Automatic cartridge type detection

## Tech Stack

### Core
- Rust
- SDL2 for graphics and audio
- Python for debugging and test scripts

### Dependencies
- sdl2 - Graphics, audio, and input handling
- once_cell - Global state management
- lazy_static - Static data initialization

## Getting Started

### Prerequisites

- Rust (latest stable version)
- SDL2 development libraries
- Git

### Installation

1. Clone the repository
```bash
git clone https://github.com/CFdefense/GameBoy.git
cd GameBoy
```

2. Install SDL2 development libraries

**Ubuntu/Debian:**
```bash
sudo apt-get install libsdl2-dev libsdl2-ttf-dev
```

**macOS:**
```bash
brew install sdl2 sdl2_ttf
```

**Windows:**
- Download SDL2 development libraries from [libsdl.org](https://www.libsdl.org/download-2.0.php)
- Extract and follow SDL2 Rust setup instructions

3. Build the project
```bash
cargo build --release
```

4. Run the emulator
```bash
cargo run --release
```

5. Load a ROM file
- Place Game Boy ROM files (.gb) in the `roms/game_roms/` directory
- Select a game from the menu or specify the ROM path directly

### Usage Examples

**Run with specific ROM:**
```bash
cargo run --release path/to/your/game.gb
```

**Enable debug mode:**
```bash
cargo run --release -- --debug path/to/your/game.gb
```

**Set instruction limit for debugging:**
```bash
cargo run --release -- --debug-limit 10000 path/to/your/game.gb
```

## Controls

- **Arrow Keys**: D-pad
- **Z**: B button
- **X**: A button
- **Enter**: Start button
- **Tab**: Select button
- **Escape**: Exit game and return to menu

## Supported Cartridge Types

- ROM Only
- MBC1 (Memory Bank Controller 1)
- MBC2 (Memory Bank Controller 2) 
- MBC3 (Memory Bank Controller 3) with Real-Time Clock
- Battery-backed SRAM for all supported MBC types

## File Structure

```
GameBoy/
├── src/
│   ├── hdw/           # Hardware emulation modules
│   │   ├── cpu.rs     # CPU implementation
│   │   ├── ppu.rs     # Picture Processing Unit
│   │   ├── apu.rs     # Audio Processing Unit
│   │   ├── cart.rs    # Cartridge and MBC handling
│   │   ├── bus.rs     # Memory bus and I/O
│   │   └── ui.rs      # User interface
│   └── main.rs        # Entry point
├── roms/              # ROM file directory
├── saves/             # Save file directory
└── logs/              # Debug log directory
```

## Development Features

- Comprehensive debug logging
- CPU state inspection
- Memory dump capabilities
- Performance profiling
- Instruction count limiting
- Step-by-step execution mode

## License

MIT License
