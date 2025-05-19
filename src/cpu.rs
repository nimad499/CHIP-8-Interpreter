use crate::{
    constant::{
        cpu::GENERAL_PURPOSE_REGISTERS_COUNT,
        display::{CHIP8_DISPLAY_HEIGHT, CHIP8_DISPLAY_WIDTH},
        ram::{FONT_LOCATION, MEMORY_SIZE},
    },
    display::{Display, DisplayBackend},
    timer::Timer,
};
use core::fmt;
use std::fmt::Write;
use std::hint::unreachable_unchecked;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instruction {
    ClearScreen(),
    Return(),
    Jump(u16),
    CallSub(u16),
    SkipEq(u8, u8),
    SkipNEq(u8, u8),
    SkipRegEq(u8, u8),
    Set(u8, u8),
    Add(u8, u8),
    AluOperation { x: u8, y: u8, operation: AluOp },
    SkipRegNEq(u8, u8),
    SetIndex(u16),
    JumpWithOffset(u16),
    Random(u8, u8),
    Display { x: u8, y: u8, height: u8 },
    SkipIfPressed(u8),
    SkipIfNotPressed(u8),
    GetDelayTimer(u8),
    WaitForKey(u8),
    SetDelayTimer(u8),
    SetSoundTimer(u8),
    AddToIndex(u8),
    SetIndexToFontLocation(u8),
    BCDConversion(u8),
    Store(u8),
    Load(u8),
    Unknown(u16),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AluOp {
    LoadRegReg,
    Or,
    And,
    Xor,
    AddRegReg,
    Sub,
    ShiftRight,
    SubNeg,
    ShiftLeft,
}

pub struct CPU {
    pub pc: u16,
    i: u16,
    registers: [u8; GENERAL_PURPOSE_REGISTERS_COUNT],
    stack: Vec<u16>,
    delay_timer: Timer,
    sound_timer: Timer,
}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

impl CPU {
    pub fn new() -> Self {
        return CPU {
            pc: 0,
            i: 0,
            registers: [0; GENERAL_PURPOSE_REGISTERS_COUNT],
            stack: Vec::new(),
            delay_timer: Timer::new(),
            sound_timer: Timer::new(),
        };
    }

    pub fn fetch(&mut self, memory: [u8; MEMORY_SIZE]) -> u16 {
        let instruction =
            ((memory[self.pc as usize] as u16) << 8) | memory[(self.pc + 1) as usize] as u16;

        self.pc += 2;

        return instruction;
    }

    pub fn decode(instruction: u16) -> Instruction {
        let low_byte = instruction as u8;
        let high_byte = (instruction >> 8) as u8;

        let x = high_byte & 0x0F;
        let y = (low_byte & 0xF0) >> 4;
        let nn = low_byte;
        let nnn = instruction & 0x0FFF;

        let opcode = high_byte & 0xF0;

        let instruction = match opcode {
            0x00 => match low_byte {
                0xE0 => Instruction::ClearScreen(),
                0xEE => Instruction::Return(),
                _ => Instruction::Unknown(instruction),
            },
            0x10 => Instruction::Jump(nnn),
            0x20 => Instruction::CallSub(nnn),
            0x30 => Instruction::SkipEq(x, nn),
            0x40 => Instruction::SkipNEq(x, nn),
            0x50 => Instruction::SkipRegEq(x, y),
            0x60 => Instruction::Set(x, nn),
            0x70 => Instruction::Add(x, nn),
            0x80 => Instruction::AluOperation {
                x,
                y,
                operation: match low_byte & 0x0F {
                    0x0 => AluOp::LoadRegReg,
                    0x1 => AluOp::Or,
                    0x2 => AluOp::And,
                    0x3 => AluOp::Xor,
                    0x4 => AluOp::AddRegReg,
                    0x5 => AluOp::Sub,
                    0x6 => AluOp::ShiftRight,
                    0x7 => AluOp::SubNeg,
                    0xE => AluOp::ShiftLeft,
                    _ => return Instruction::Unknown(instruction),
                },
            },
            0x90 => Instruction::SkipRegNEq(x, y),
            0xA0 => Instruction::SetIndex(nnn),
            0xB0 => Instruction::JumpWithOffset(nnn),
            0xC0 => Instruction::Random(x, nn),
            0xD0 => Instruction::Display {
                x,
                y,
                height: low_byte & 0x0F,
            },
            0xE0 => match low_byte {
                0x9E => Instruction::SkipIfPressed(x),
                0xA1 => Instruction::SkipIfNotPressed(x),
                _ => Instruction::Unknown(instruction),
            },
            0xF0 => match low_byte {
                0x07 => Instruction::GetDelayTimer(x),
                0x0A => Instruction::WaitForKey(x),
                0x15 => Instruction::SetDelayTimer(x),
                0x18 => Instruction::SetSoundTimer(x),
                0x1E => Instruction::AddToIndex(x),
                0x29 => Instruction::SetIndexToFontLocation(x),
                0x33 => Instruction::BCDConversion(x),
                0x55 => Instruction::Store(x),
                0x65 => Instruction::Load(x),
                _ => Instruction::Unknown(instruction),
            },
            _ => unsafe { unreachable_unchecked() },
        };

        return instruction;
    }

    pub fn execute<B: DisplayBackend>(
        &mut self,
        instruction: Instruction,
        memory: &mut [u8; MEMORY_SIZE],
        display: &mut Display<B>,
    ) {
        match instruction {
            Instruction::ClearScreen() => {
                for row in display.pixels.iter_mut() {
                    for pixel in row.iter_mut() {
                        *pixel = false;
                    }
                }
            }
            Instruction::Return() => {
                self.pc = self.stack.pop().expect("Return while stack is empty.")
            }
            Instruction::Jump(nnn) => self.pc = nnn,
            Instruction::CallSub(nnn) => {
                self.stack.push(self.pc);
                self.pc = nnn;
            }
            Instruction::SkipEq(x, nn) => {
                if self.registers[x as usize] == nn {
                    self.pc += 2;
                }
            }
            Instruction::SkipNEq(x, nn) => {
                if self.registers[x as usize] != nn {
                    self.pc += 2;
                }
            }
            Instruction::SkipRegEq(x, y) => {
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.pc += 2;
                }
            }
            Instruction::Set(x, nn) => self.registers[x as usize] = nn,
            Instruction::Add(x, nn) => {
                self.registers[x as usize] = self.registers[x as usize].wrapping_add(nn)
            }
            Instruction::AluOperation { x, y, operation } => match operation {
                AluOp::LoadRegReg => self.registers[x as usize] = self.registers[y as usize],
                AluOp::Or => self.registers[x as usize] |= self.registers[y as usize],
                AluOp::And => self.registers[x as usize] &= self.registers[y as usize],
                AluOp::Xor => self.registers[x as usize] ^= self.registers[y as usize],
                AluOp::AddRegReg => {
                    let overflow;
                    (self.registers[x as usize], overflow) =
                        self.registers[x as usize].overflowing_add(self.registers[y as usize]);

                    self.registers[0xF] = overflow as u8;
                }
                AluOp::Sub => {
                    let overflow;
                    (self.registers[x as usize], overflow) =
                        self.registers[x as usize].overflowing_sub(self.registers[y as usize]);

                    self.registers[0xF] = !overflow as u8;
                }
                AluOp::ShiftRight => {
                    self.registers[y as usize] = ((self.registers[x as usize] & 0x01) == 1) as u8;
                    self.registers[x as usize] >>= 1;
                }
                AluOp::SubNeg => {
                    let overflow;
                    (self.registers[x as usize], overflow) =
                        self.registers[y as usize].overflowing_sub(self.registers[x as usize]);

                    self.registers[0xF] = !overflow as u8;
                }
                AluOp::ShiftLeft => {
                    self.registers[y as usize] =
                        ((self.registers[x as usize] & 0x80) == 0x80) as u8;
                    self.registers[x as usize] <<= 1;
                }
            },
            Instruction::SkipRegNEq(x, y) => {
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.pc += 2;
                }
            }
            Instruction::SetIndex(nnn) => self.i = nnn,
            Instruction::JumpWithOffset(nnn) => self.pc = nnn + self.registers[0x0] as u16,
            Instruction::Random(x, nn) => self.registers[x as usize] = fastrand::u8(..) & nn,
            Instruction::Display { x, y, height } => {
                let x_cord = (self.registers[x as usize] % CHIP8_DISPLAY_WIDTH as u8) as usize;
                let y_cord = (self.registers[y as usize] % CHIP8_DISPLAY_HEIGHT as u8) as usize;

                self.registers[0xF] = 0;

                for n in 0..height as usize {
                    if y_cord + n >= CHIP8_DISPLAY_HEIGHT {
                        break;
                    }

                    let row = memory[self.i as usize + n];

                    for m in 0..8 {
                        if x_cord + m >= CHIP8_DISPLAY_WIDTH {
                            break;
                        }

                        let bit = ((row >> (7 - m)) & 0x01) == 1;

                        if bit {
                            if display.pixels[y_cord + n][x_cord + m] {
                                self.registers[0xF] = 1;
                            }

                            display.pixels[y_cord + n][x_cord + m] =
                                !display.pixels[y_cord + n][x_cord + m];
                        }
                    }
                }

                display.render();
            }
            Instruction::SkipIfPressed(x) => {
                let pressed_keys = display.read_keys();
                if pressed_keys.contains(&self.registers[x as usize]) {
                    self.pc += 2;
                }
            }
            Instruction::SkipIfNotPressed(x) => {
                let pressed_keys = display.read_keys();
                if !pressed_keys.contains(&self.registers[x as usize]) {
                    self.pc += 2;
                }
            }
            Instruction::GetDelayTimer(x) => {
                self.registers[x as usize] = self.delay_timer.get_value()
            }
            Instruction::WaitForKey(x) => {
                let key = display.wait_for_key();
                self.registers[x as usize] = key;
            }
            Instruction::SetDelayTimer(x) => self.delay_timer.set_value(self.registers[x as usize]),
            Instruction::SetSoundTimer(x) => self.sound_timer.set_value(self.registers[x as usize]),
            Instruction::AddToIndex(x) => {
                let old_i = self.i;
                self.i += self.registers[x as usize] as u16;

                if self.i > 0xFFF && old_i <= 0xFFF {
                    self.registers[x as usize] = 1;
                }
            }
            Instruction::SetIndexToFontLocation(x) => self.i = x as u16 * 5 + FONT_LOCATION as u16,
            Instruction::BCDConversion(x) => {
                let vx = self.registers[x as usize];
                let i = self.i as usize;

                memory[i] = vx / 100;
                memory[i + i] = (vx / 10) % 10;
                memory[i + 2] = vx % 10;
            }
            Instruction::Store(x) => {
                let i: usize = self.i.into();

                memory[i..=(i + x as usize)].copy_from_slice(&self.registers[0..=x as usize]);
            }
            Instruction::Load(x) => {
                let i: usize = self.i.into();

                self.registers[0..=x as usize].copy_from_slice(&memory[i..=(i + x as usize)]);
            }
            Instruction::Unknown(instruction) => panic!("Unknown instruction: {:X}", instruction),
        }
    }
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ToDo: Replace this with a procedural macro
        let mut registers = String::new();
        for n in 0..self.registers.len() {
            writeln!(registers, "V{:X}: {:X}", n, self.registers[n]).unwrap();
        }

        return write!(
            f,
            "PC: 0x{:X}\nI: 0x{:X}\nStack: {:?}\n{}",
            self.pc, self.i, self.stack, registers
        );
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::ClearScreen() => write!(f, "CLS"),
            Instruction::Return() => write!(f, "RET"),
            Instruction::Jump(nnn) => write!(f, "JP 0x{nnn:X}"),
            Instruction::CallSub(nnn) => write!(f, "CALL 0x{nnn:X}"),
            Instruction::SkipEq(x, nn) => write!(f, "SE V{x:X}, 0x{nn:X}"),
            Instruction::SkipNEq(x, nn) => write!(f, "SNE V{x:X}, 0x{nn:X}"),
            Instruction::SkipRegEq(x, y) => write!(f, "SE V{x:X}, V{y:X}"),
            Instruction::Set(x, nn) => write!(f, "LD V{x:X}, 0x{nn:X}"),
            Instruction::Add(x, nn) => write!(f, "ADD V{x:X}, 0x{nn:X}"),
            Instruction::AluOperation { x, y, operation } => match operation {
                AluOp::LoadRegReg => write!(f, "LD V{x:X}, V{y:X}"),
                AluOp::Or => write!(f, "OR V{x:X}, V{y:X}"),
                AluOp::And => write!(f, "AND V{x:X}, V{y:X}"),
                AluOp::Xor => write!(f, "XOR V{x:X}, V{y:X}"),
                AluOp::AddRegReg => write!(f, "ADD V{x:X}, V{y:X}"),
                AluOp::Sub => write!(f, "SUB V{x:X}, V{y:X}"),
                AluOp::ShiftRight => write!(f, "SHR V{x:X} {{, V{y:X}}}"),
                AluOp::SubNeg => write!(f, "SUBN V{x:X}, V{y:X}"),
                AluOp::ShiftLeft => write!(f, "SHL V{x:X} {{, V{y:X}}}"),
            },
            Instruction::SkipRegNEq(x, y) => write!(f, "SNE V{x:X}, V{y:X}"),
            Instruction::SetIndex(nnn) => write!(f, "LD I, 0x{nnn:X}"),
            Instruction::JumpWithOffset(nnn) => write!(f, "JP V0, 0x{nnn:X}"),
            Instruction::Random(x, nn) => write!(f, "RND V{x:X}, 0x{nn:X}"),
            Instruction::Display { x, y, height } => write!(f, "DRW V{x:X}, V{y:X}, {height:X}"),
            Instruction::SkipIfPressed(x) => write!(f, "SKP V{x:X}"),
            Instruction::SkipIfNotPressed(x) => write!(f, "SKNP V{x:X}"),
            Instruction::GetDelayTimer(x) => write!(f, "LD V{x:X}, DT"),
            Instruction::WaitForKey(x) => write!(f, "LD V{x:X}, K"),
            Instruction::SetDelayTimer(x) => write!(f, "LD DT, V{x:X}"),
            Instruction::SetSoundTimer(x) => write!(f, "LD ST, V{x:X}"),
            Instruction::AddToIndex(x) => write!(f, "ADD I, V{x:X}"),
            Instruction::SetIndexToFontLocation(x) => write!(f, "LD F, V{x:X}"),
            Instruction::BCDConversion(x) => write!(f, "LD B, V{x:X}"),
            Instruction::Store(x) => write!(f, "LD [I], V{x:X}"),
            Instruction::Load(x) => write!(f, "LD V{x:X}, [I]"),
            Instruction::Unknown(instruction) => write!(f, ".dw 0x{instruction:X}"),
        }
    }
}

pub fn disassemble(rom_data: &[u8]) -> String {
    let mut result = String::new();

    rom_data.chunks_exact(2).for_each(|instruction| {
        let instruction = ((instruction[0] as u16) << 8) | instruction[1] as u16;
        let instruction = CPU::decode(instruction);

        writeln!(result, "{instruction}").unwrap();
    });

    if rom_data.len() % 2 == 1 {
        writeln!(result, ".db 0x{}", rom_data.last().unwrap()).unwrap();
    }

    return result;
}

#[cfg(test)]
mod tests {
    use crate::{
        constant::{
            display::{CHIP8_DISPLAY_HEIGHT, CHIP8_DISPLAY_WIDTH},
            ram::FONT_LOCATION,
        },
        cpu::Instruction::*,
        display::{CLIBackend, Display},
        ram::Ram,
    };

    use super::CPU;

    #[test]
    fn cpu_execution() {
        let mut ram = Ram::new();
        let mut display = Display::new(CLIBackend::default());
        let mut cpu = CPU::new();

        macro_rules! execute {
            ($instruction:expr) => {
                cpu.execute($instruction, &mut ram.memory, &mut display);
            };
        }

        execute!(Jump(10));
        assert_eq!(cpu.pc, 10);
        execute!(Jump(0));
        assert_eq!(cpu.pc, 0);

        execute!(CallSub(20));
        assert_eq!(cpu.pc, 20);

        execute!(Return());
        assert_eq!(cpu.pc, 0);

        execute!(Set(0, 2));
        assert_eq!(cpu.registers[0], 2);

        execute!(Add(0, 2));
        assert_eq!(cpu.registers[0], 4);

        execute!(Set(1, 4));
        assert_eq!(cpu.registers[1], 4);

        execute!(Set(2, 2));
        assert_eq!(cpu.registers[2], 2);

        execute!(SkipEq(0, 4));
        assert_eq!(cpu.pc, 2);
        execute!(SkipEq(0, 0));
        assert_eq!(cpu.pc, 2);

        execute!(SkipNEq(0, 0));
        assert_eq!(cpu.pc, 4);
        execute!(SkipNEq(0, 4));
        assert_eq!(cpu.pc, 4);

        execute!(SkipRegEq(0, 1));
        assert_eq!(cpu.pc, 6);
        execute!(SkipRegEq(0, 2));
        assert_eq!(cpu.pc, 6);

        for i in 0..=0xF {
            execute!(SetIndexToFontLocation(i as u8));
            assert_eq!(cpu.i, FONT_LOCATION as u16 + i * 0x5);
        }

        for i in 0..=0xF {
            execute!(Set(i, i));
            assert_eq!(cpu.registers[i as usize], i);
        }
        execute!(SetIndex(0));
        assert_eq!(cpu.i, 0);
        execute!(Store(0xF));
        assert_eq!(ram.memory[0..=0xF], (0..=0xF).collect::<Vec<u8>>());

        for i in 0..=0xF {
            execute!(Set(i, 0));
            assert_eq!(cpu.registers[i as usize], 0);
        }
        execute!(Load(0xF));
        assert_eq!(cpu.registers[0..=0xF], (0..=0xF).collect::<Vec<u8>>());

        execute!(AddToIndex(1));
        assert_eq!(cpu.i, 1);

        execute!(Set(0, 12));
        assert_eq!(cpu.registers[0], 12);
        execute!(BCDConversion(0));
        for i in 1..=3 {
            assert_eq!(ram.memory[i], i as u8 - 1);
        }

        execute!(Set(0, 1));
        assert_eq!(cpu.registers[0], 1);
        execute!(JumpWithOffset(0x5));
        assert_eq!(cpu.pc, 0x6);

        for i in 0..=0xF {
            execute!(Random(i, 0));
            assert_eq!(cpu.registers[i as usize], 0);
        }
        for i in 0..=0xF {
            execute!(Random(i, 0x0F));
            assert!(cpu.registers[i as usize] <= 0x0F);
        }

        for i in (0..display.pixels.as_flattened().len()).step_by(3) {
            display.pixels.as_flattened_mut()[i] = true;
        }
        execute!(ClearScreen());
        assert_eq!(
            display.pixels,
            [[false; CHIP8_DISPLAY_WIDTH]; CHIP8_DISPLAY_HEIGHT]
        );
    }
}
