pub trait DisplayBackend: Default {
    fn render(&mut self, pixels: &[[bool; 64]; 32]);
}

pub struct CLIBackend {
    pub pixel_character: char,
    buffer: String,
}

impl CLIBackend {
    fn new() -> Self {
        return CLIBackend {
            pixel_character: 'O',
            buffer: String::with_capacity(2080),
        };
    }

    fn clear() {
        if cfg!(unix) {
            print!("{esc}c", esc = 27 as char);
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
            self.buffer.push('\n');
        }

        Self::clear();
        print!("{}", self.buffer);
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
}
