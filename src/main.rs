// ToDo: Replace magic numbers with const values
#![allow(dead_code)]
#![allow(clippy::needless_return)]
#![allow(clippy::upper_case_acronyms)]

use chip8::CHIP8;
use std::path::Path;

mod chip8;
mod cpu;
mod display;
mod ram;

fn main() {
    let rom_path = Path::new("/home/nima/Downloads/1-chip8-logo.ch8");
    let rom_data = std::fs::read(rom_path).unwrap();

    let mut chip8 = CHIP8::new();
    chip8.load_rom(&rom_data).unwrap();

    chip8.start();
}
