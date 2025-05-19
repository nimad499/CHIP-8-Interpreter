use std::time::{Duration, Instant};

pub struct Timer {
    value: u8,
    frequency: u8,
    start: Instant,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    pub fn new() -> Self {
        return Timer {
            value: 0,
            frequency: 60,
            start: Instant::now(),
        };
    }

    pub fn set_value(&mut self, value: u8) {
        self.value = value;

        self.start = Instant::now();
    }

    pub fn get_value(&mut self) -> u8 {
        let ticks = self.start.elapsed().as_micros() as u64 / (1000000 / self.frequency as u64);
        self.start += Duration::from_micros(1000000 * ticks);

        if ticks > u8::MAX as u64 {
            // ToDo: cold_path()
            self.value = 0;
        } else {
            let (value, overflow) = self.value.overflowing_sub(ticks as u8);
            self.value = value * !overflow as u8;
        }

        return self.value;
    }
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;

    use super::*;

    #[test]
    fn wait_zero_ticks() {
        let mut timer: Timer = Timer::new();
        timer.set_value(u8::MAX / 2);

        assert_eq!(timer.get_value(), u8::MAX / 2);
    }

    #[test]
    fn wait_one_tick() {
        let mut timer = Timer::new();
        timer.set_value(u8::MAX);

        sleep(Duration::from_millis(1000 / timer.frequency as u64 + 1));

        assert_eq!(timer.get_value(), u8::MAX - 1);
    }

    #[test]
    fn wait_hundred_ticks() {
        let mut timer = Timer::new();
        timer.set_value(u8::MAX);

        sleep(Duration::from_millis(100000 / timer.frequency as u64 + 1));

        assert_eq!(timer.get_value(), u8::MAX - 100);
    }

    #[test]
    fn wait_255_ticks() {
        let mut timer = Timer::new();
        timer.set_value(u8::MAX);

        sleep(Duration::from_millis(255000 / timer.frequency as u64 + 1));

        assert_eq!(timer.get_value(), 0);
    }

    #[test]
    fn wait_256_ticks() {
        let mut timer = Timer::new();
        timer.set_value(u8::MAX);

        sleep(Duration::from_millis(256000 / timer.frequency as u64 + 1));

        assert_eq!(timer.get_value(), 0);
    }
}
