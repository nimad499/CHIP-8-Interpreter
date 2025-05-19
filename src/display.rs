use crate::constant::display::{
    CHIP8_DISPLAY_HEIGHT, CHIP8_DISPLAY_WIDTH, CLI_BACKEND_BUFFER_SIZE,
};
use crossterm::{
    event::{self, Event, poll},
    terminal,
};
use minifb::{Key, Window, WindowOptions};
use std::{
    collections::HashSet,
    io::{self, Read, Write, stdin},
    time::{Duration, Instant},
};

pub trait DisplayBackend: Default {
    fn render(&mut self, pixels: &[[bool; CHIP8_DISPLAY_WIDTH]; CHIP8_DISPLAY_HEIGHT]);
    fn read_keys(&mut self) -> Vec<u8>;
    fn wait_for_key(&mut self) -> u8;
    fn log(&self, message: String);
}

pub struct CLIBackend {
    pub pixel_character: char,
    buffer: String,
    key_map: [char; 16],
}

impl Drop for CLIBackend {
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
    }
}

impl CLIBackend {
    pub fn new() -> Self {
        terminal::enable_raw_mode().unwrap();

        return CLIBackend {
            pixel_character: 'O',
            // ToDo: Replace this with array
            buffer: String::with_capacity(CLI_BACKEND_BUFFER_SIZE),
            // ToDo: Check the performance of enum for the key_map
            key_map: [
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
    fn render(&mut self, pixels: &[[bool; CHIP8_DISPLAY_WIDTH]; CHIP8_DISPLAY_HEIGHT]) {
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

    fn read_keys(&mut self) -> Vec<u8> {
        let mut pressed_keys = HashSet::<u8>::new();

        let start = Instant::now();
        let time_window = Duration::from_micros(10);
        let single_polling_time = Duration::from_micros(1);

        while start.elapsed() < time_window {
            if poll(single_polling_time).unwrap() {
                if let Event::Key(event) = event::read().unwrap() {
                    if event.is_press() {
                        if let Some(key_code) = self
                            .key_map
                            .iter()
                            .position(|key_code| *key_code == event.code.as_char().unwrap())
                        {
                            pressed_keys.insert(key_code as u8);
                        };
                    }
                }
            }
        }

        let pressed_keys = Vec::from_iter(pressed_keys);

        return pressed_keys;
    }

    fn wait_for_key(&mut self) -> u8 {
        loop {
            for b in stdin().bytes() {
                let b = b.unwrap() as char;

                if let Some(i) = self.key_map.iter().position(|k| b == *k) {
                    return i as u8;
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

pub struct WindowSize {
    pub width: usize,
    pub height: usize,
}

pub struct GUIBackend {
    window: Window,
    buffer: Vec<u32>,
    key_map: [Key; 16],
}

impl GUIBackend {
    pub fn new(window_size: WindowSize) -> Self {
        let mut window = Window::new(
            "CHIP8",
            window_size.width,
            window_size.height,
            WindowOptions::default(),
        )
        .unwrap();

        window.set_target_fps(60);

        let buffer = vec![0; window_size.width * window_size.height];

        return GUIBackend {
            window,
            buffer,
            key_map: [
                Key::Key1,
                Key::Key2,
                Key::Key3,
                Key::Key4,
                Key::Q,
                Key::W,
                Key::E,
                Key::R,
                Key::A,
                Key::S,
                Key::D,
                Key::F,
                Key::Z,
                Key::X,
                Key::C,
                Key::V,
            ],
        };
    }
}

impl Default for GUIBackend {
    fn default() -> Self {
        return Self::new(WindowSize {
            width: CHIP8_DISPLAY_WIDTH * 10,
            height: CHIP8_DISPLAY_HEIGHT * 10,
        });
    }
}

impl DisplayBackend for GUIBackend {
    fn render(&mut self, pixels: &[[bool; CHIP8_DISPLAY_WIDTH]; CHIP8_DISPLAY_HEIGHT]) {
        let (width, height) = self.window.get_size();

        let height_multiplier = height / CHIP8_DISPLAY_HEIGHT;
        let width_multiplier = width / CHIP8_DISPLAY_WIDTH;
        for (i, row) in pixels.iter().enumerate() {
            for (j, &pixel) in row.iter().enumerate() {
                let value = pixel as u32 * 0x00FFFFFF;

                for x in i * height_multiplier..i * height_multiplier + height_multiplier {
                    for y in j * width_multiplier..j * width_multiplier + width_multiplier {
                        self.buffer[x * width + y] = value;
                    }
                }
            }
        }

        self.window
            .update_with_buffer(&self.buffer, width, height)
            .unwrap();
    }

    fn read_keys(&mut self) -> Vec<u8> {
        self.window.update();

        return self
            .window
            .get_keys()
            .iter()
            .filter_map(|pressed_key| {
                if let Some(i) = self.key_map.iter().position(|k| pressed_key == k) {
                    return Some(i as u8);
                }
                return None;
            })
            .collect();
    }

    fn wait_for_key(&mut self) -> u8 {
        loop {
            self.window.update();

            for pressed_key in self.window.get_keys() {
                if let Some(i) = self.key_map.iter().position(|k| pressed_key == *k) {
                    return i as u8;
                }
            }
        }
    }

    fn log(&self, message: String) {
        println!("{message}");
    }
}

pub struct Display<B: DisplayBackend> {
    pub pixels: [[bool; CHIP8_DISPLAY_WIDTH]; CHIP8_DISPLAY_HEIGHT],
    pub backend: B,
}

impl<B: DisplayBackend> Display<B> {
    pub fn new(backend: B) -> Self {
        return Display {
            pixels: [[false; CHIP8_DISPLAY_WIDTH]; CHIP8_DISPLAY_HEIGHT],
            backend,
        };
    }

    pub fn render(&mut self) {
        self.backend.render(&self.pixels);
    }

    pub fn read_keys(&mut self) -> Vec<u8> {
        return self.backend.read_keys();
    }

    pub fn wait_for_key(&mut self) -> u8 {
        return self.backend.wait_for_key();
    }

    pub fn log(&self, message: String) {
        self.backend.log(message);
    }
}
