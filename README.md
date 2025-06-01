# Game Boy Emulator

## Overview

This project involves building a Game Boy emulator using Rust.

The Game Boy, launched by Nintendo in 1989, is a pioneering handheld gaming console known for its compact design and extensive library of over 1,000 games. 
It features a 2.6-inch monochrome LCD screen and is powered by a custom 8-bit Z80-like CPU running at 4.19 MHz. This project will emulate all aspects of this timeless system.

## Project Goals

- **Skill Development**: Enhance my knowledge of low-level programming concepts and emulator architecture, building on the experience gained from developing a Chip-8 interpreter.
- **Learn Rust**: Familiarize myself with Rust’s syntax, features, and ecosystem, emphasizing memory safety and performance.

## Resources

To guide the development process, I will utilize the following resources:

- [Game Boy Technical Reference](https://gekkio.fi/files/gb-docs/gbctr.pdf#page=20&zoom=100,76,62): A comprehensive document detailing the Game Boy architecture, including the CPU, memory mapping, and I/O operations.
- [Game Boy CPU Instruction Set](https://gbdev.io/pandocs/CPU_Instruction_Set.html): This document outlines the instruction set for the Game Boy CPU, providing essential information for implementing the instruction decoding and execution phases.

## Features

- **CPU Emulation**: Implement the Game Boy’s CPU, including the instruction set and registers.
- **Memory Management**: Handle memory mapping and interactions with the Game Boy’s memory architecture.
- **Input Handling**: Process user inputs from a keyboard or controller to simulate Game Boy controls.
- **Rendering**: Implement basic graphics rendering for displaying Game Boy graphics on the screen.

## Getting Started

### Prerequisites

- **Rust**: Ensure you have the latest version of Rust installed. You can download it from the official [Rust website](https://www.rust-lang.org/).

### Installation

1. Clone the repository:

```bash
git clone https://github.com/CFdefense/GameBoy.git
cd gameboy-emulator
```

2. Build the project
```bash
cargo build --release
```
3. Run the project
```bash
cargo run --release
```
