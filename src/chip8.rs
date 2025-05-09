use core::time;
use std::{thread::sleep, time::Instant};

use crate::{
    cpu::{CPU, Instruction},
    display::{CLIBackend, Display, DisplayBackend},
    ram::{Ram, RomError},
};

pub struct CHIP8<B: DisplayBackend> {
    cpu: CPU,
    ram: Ram,
    display: Display<B>,
}

impl CHIP8<CLIBackend> {
    pub fn new() -> Self {
        return CHIP8 {
            cpu: CPU::new(),
            ram: Ram::new(),
            display: Display::<CLIBackend>::new(CLIBackend::default()),
        };
    }
}

impl<B: DisplayBackend> CHIP8<B> {
    pub fn new_custom_display_backend(display_backend: B) -> Self {
        return CHIP8 {
            cpu: CPU::new(),
            ram: Ram::new(),
            display: Display::<B>::new(display_backend),
        };
    }

    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<(), RomError> {
        self.cpu.pc = 0x200;

        return self.ram.load_rom(rom_data);
    }

    fn tick(&mut self) {
        let instruction = self.cpu.fetch(self.ram.memory);
        let instruction = CPU::decode(instruction);

        let pressed_keys = if matches!(
            instruction,
            Instruction::SkipIfPressed(..) | Instruction::SkipIfNotPressed(..)
        ) {
            self.display.read_keys()
        } else {
            Vec::new()
        };

        self.cpu.execute(
            instruction,
            &mut self.ram.memory,
            &mut self.display.pixels,
            pressed_keys,
        );

        if matches!(instruction, Instruction::Display { .. }) {
            self.display.render();
        }
    }

    pub fn start(&mut self) {
        loop {
            let start = Instant::now();

            self.tick();

            let elapsed = ((1000000000 / 700) as u128).overflowing_sub(start.elapsed().as_nanos());
            let sleep_duration = (elapsed.0 * !elapsed.1 as u128) as u64;

            sleep(time::Duration::from_nanos(sleep_duration));
        }
    }
}
