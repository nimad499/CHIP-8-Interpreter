pub mod display {
    pub const CLI_BACKEND_BUFFER_SIZE: usize = 2112;
    pub const CHIP8_DISPLAY_HEIGHT: usize = 32;
    pub const CHIP8_DISPLAY_WIDTH: usize = 64;
}

pub mod cpu {
    pub const GENERAL_PURPOSE_REGISTERS_COUNT: usize = 16;
}

pub mod ram {
    pub const MEMORY_SIZE: usize = 4096;
    pub const FONT_LOCATION: usize = 0x50;
    pub const ROM_START_LOCATION: usize = 0x200;
}

pub mod chip8 {
    pub const CPU_INSTRUCTION_PER_SECOND: usize = 700;
}
