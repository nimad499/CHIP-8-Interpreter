// ToDo: Replace magic numbers with const values
#![allow(dead_code)]
#![allow(clippy::needless_return)]
#![allow(clippy::upper_case_acronyms)]

use chip_8::{
    chip8::CHIP8,
    display::{GUIBackend, WindowSize},
};
use std::path::Path;

fn main() {
    let rom_path = Path::new("/home/nima/Downloads/1-chip8-logo.ch8");
    let rom_data = std::fs::read(rom_path).unwrap();

    let mut chip8 = CHIP8::new_custom_display_backend(GUIBackend::new(WindowSize {
        width: 1280,
        height: 640,
    }));

    chip8.load_rom(&rom_data).unwrap();

    chip8.start(false);
}
