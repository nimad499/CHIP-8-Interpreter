#[derive(Debug)]
pub enum RomError {
    InvalidRomSize(usize),
}

pub struct Ram {
    pub memory: [u8; 4096],
}

impl Ram {
    pub fn new() -> Self {
        return Ram { memory: [0; 4096] };
    }

    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<(), RomError> {
        if self.memory.len() - rom_data.len() < 0x200 {
            return Err(RomError::InvalidRomSize(rom_data.len()));
        }

        self.memory[0x200..(0x200 + rom_data.len())].copy_from_slice(rom_data);

        return Ok(());
    }
}
