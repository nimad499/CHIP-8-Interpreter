use crate::{
    constant::{chip8::CPU_INSTRUCTION_PER_SECOND, ram::ROM_START_LOCATION},
    cpu::CPU,
    display::{CLIBackend, Display, DisplayBackend},
    ram::{Ram, RomError},
};
use core::time;
use std::{thread::sleep, time::Instant};

pub struct CHIP8<B: DisplayBackend> {
    cpu: CPU,
    ram: Ram,
    display: Display<B>,
}

impl Default for CHIP8<CLIBackend> {
    fn default() -> Self {
        Self::new()
    }
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
        self.cpu.pc = ROM_START_LOCATION as u16;

        return self.ram.load_rom(rom_data);
    }

    fn tick(&mut self) {
        let instruction = self.cpu.fetch(self.ram.memory);
        let instruction = CPU::decode(instruction);

        self.cpu
            .execute(instruction, &mut self.ram.memory, &mut self.display);
    }

    fn debug_tick(&mut self) {
        let instruction = self.cpu.fetch(self.ram.memory);
        let instruction = CPU::decode(instruction);

        self.cpu
            .execute(instruction, &mut self.ram.memory, &mut self.display);

        self.display.log(format!("{}\n{}", instruction, self.cpu));
    }

    pub fn start(&mut self, debug: bool) {
        let tick = if debug { Self::debug_tick } else { Self::tick };

        loop {
            let start = Instant::now();

            tick(self);

            let elapsed = ((1000000000 / CPU_INSTRUCTION_PER_SECOND) as u128)
                .overflowing_sub(start.elapsed().as_nanos());
            let sleep_duration = (elapsed.0 * !elapsed.1 as u128) as u64;

            sleep(time::Duration::from_nanos(sleep_duration));
        }
    }
}
