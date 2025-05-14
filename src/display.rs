use std::{
    collections::HashSet,
    io::{self, Read, Write, stdin},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, poll},
    terminal,
};

type KeyMap = [char; 16];

pub trait DisplayBackend: Default {
    fn render(&mut self, pixels: &[[bool; 64]; 32]);
    fn read_keys(&self) -> Vec<u8>;
    fn wait_for_key(&self) -> u8;
    fn log(&self, message: String);
}

pub struct CLIBackend {
    pub pixel_character: char,
    buffer: String,
    key_map: KeyMap,
}

impl Drop for CLIBackend {
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
    }
}

impl CLIBackend {
    fn new() -> Self {
        terminal::enable_raw_mode().unwrap();

        return CLIBackend {
            pixel_character: 'O',
            // ToDo: Replace this with array
            buffer: String::with_capacity(2112),
            // ToDo: Check the performance of HashMap for the key_map
            key_map: [
                // 49, 50, 51, 52, 113, 119, 101, 114, 97, 115, 100, 102, 122, 120, 99, 118,
                '1', '2', '3', '4', 'q', 'w', 'e', 'r', 'a', 's', 'd', 'f', 'z', 'x', 'c', 'v',
            ],
        };
    }

    fn clear() {
        if cfg!(unix) {
            print!("{esc}c", esc = 27 as char);
            io::stdout().flush().unwrap();
        } else if cfg!(windows) {
            todo!();
        }
    }
}

impl Default for CLIBackend {
    fn default() -> Self {
        return Self::new();
    }
}

impl DisplayBackend for CLIBackend {
    fn render(&mut self, pixels: &[[bool; 64]; 32]) {
        self.buffer.clear();

        for row in pixels.iter() {
            for &pixel in row {
                self.buffer
                    .push(if pixel { self.pixel_character } else { ' ' });
            }
            self.buffer.push('\r');
            self.buffer.push('\n');
        }

        Self::clear();
        print!("{}", self.buffer);
        io::stdout().flush().unwrap();
    }

    fn read_keys(&self) -> Vec<u8> {
        let mut pressed_keys = HashSet::<u8>::new();

        let start = Instant::now();
        let time_window = Duration::from_micros(10);
        let single_polling_time = Duration::from_micros(1);

        while start.elapsed() < time_window {
            if poll(single_polling_time).unwrap() {
                if let Event::Key(event) = event::read().unwrap() {
                    if event.is_press() {
                        match self
                            .key_map
                            .iter()
                            .position(|key_code| *key_code == event.code.as_char().unwrap())
                        {
                            Some(key_code) => {
                                pressed_keys.insert(key_code as u8);
                            }
                            None => (),
                        };
                    }
                }
            }
        }

        let pressed_keys = Vec::from_iter(pressed_keys);

        return pressed_keys;
    }

    fn wait_for_key(&self) -> u8 {
        loop {
            for b in stdin().bytes() {
                let b = b.unwrap();

                if self.key_map.contains(&(b as char)) {
                    return b;
                }
            }
        }
    }

    fn log(&self, message: String) {
        // ToDo: Get rid of redrawing
        Self::clear();

        print!("{}", self.buffer);

        let message = message.replace("\n", "\r\n");
        print!("{}\r", message);

        io::stdout().flush().unwrap();
    }
}

pub struct Display<B: DisplayBackend> {
    pub pixels: [[bool; 64]; 32],
    pub backend: B,
}

impl<B: DisplayBackend> Display<B> {
    pub fn new(backend: B) -> Self {
        return Display {
            pixels: [[false; 64]; 32],
            backend,
        };
    }

    pub fn render(&mut self) {
        self.backend.render(&self.pixels);
    }

    pub fn read_keys(&self) -> Vec<u8> {
        return self.backend.read_keys();
    }

    pub fn wait_for_key(&self) -> u8 {
        return self.backend.wait_for_key();
    }

    pub fn log(&self, message: String) {
        self.backend.log(message);
    }
}
