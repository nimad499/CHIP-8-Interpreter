use std::hint::unreachable_unchecked;

use crate::{
    display::{Display, DisplayBackend},
    timer::Timer,
};

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
    registers: [u8; 16],
    stack: Vec<u16>,
    delay_timer: Timer,
    sound_timer: Timer,
}

impl CPU {
    pub fn new() -> Self {
        return CPU {
            pc: 0,
            i: 0,
            registers: [0; 16],
            stack: Vec::new(),
            delay_timer: Timer::new(),
            sound_timer: Timer::new(),
        };
    }

    pub fn fetch(&mut self, memory: [u8; 4096]) -> u16 {
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
                _ => panic!("Unknown instruction: {:X}", instruction),
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
                    _ => panic!("Invalid ALU operation"),
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
                _ => panic!("Unknown instruction: {:X}", instruction),
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
                _ => panic!("Unknown instruction: {:X}", instruction),
            },
            _ => unsafe { unreachable_unchecked() },
        };

        return instruction;
    }

    pub fn execute<B: DisplayBackend>(
        &mut self,
        instruction: Instruction,
        memory: &mut [u8; 4096],
        display: &mut Display<B>,
    ) {
        match instruction {
            Instruction::ClearScreen() => display.pixels = [[false; 64]; 32],
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
                let x_cord = (self.registers[x as usize] % 64) as usize;
                let y_cord = (self.registers[y as usize] % 32) as usize;

                self.registers[0xF] = 0;

                for n in 0..height as usize {
                    if y_cord + n >= 32 {
                        break;
                    }

                    let row = memory[self.i as usize + n];

                    for m in 0..8 {
                        if x_cord + m >= 64 {
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
            Instruction::SetIndexToFontLocation(x) => self.i = x as u16 * 5 + 0x50,
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
        }
    }
}
