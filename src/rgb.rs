use crate::*;

type RgbPins = [Output<'static, AnyPin>; 3];

pub struct Rgb {
    rgb: RgbPins,
    // Shadow array to minimize lock contention.
    levels: [u32; 3],
    tick: u32,
    tick_time: Duration,
}

impl Rgb {
    fn frame_tick_time(frame_rate: u64) -> Duration {
        Duration::from_micros(1_000_000 / (3 * frame_rate * LEVELS as u64))
    }

    pub fn new(rgb: RgbPins, frame_rate: u64) -> Self {
        let tick_time = Self::frame_tick_time(frame_rate);
        Self {
            rgb,
            levels: [0; 3],
            tick: 0,
            tick_time,
        }
    }

    async fn step(&mut self) {
        let led = self.tick / LEVELS;
        let level = self.tick % LEVELS;
        if level == 0 {
            if led == 0 {
                self.levels = get_rgb_levels().await;
            }

            let prev = (led + 2) % 3;
            if self.rgb[prev as usize].is_set_high() {
                self.rgb[prev as usize].set_low();
            }
        }
        if level < self.levels[led as usize] {
            self.rgb[led as usize].set_high();
        } else if self.rgb[led as usize].is_set_high() {
            self.rgb[led as usize].set_low();
        }
        self.tick = (self.tick + 1) % (3 * LEVELS);
        Timer::after(self.tick_time).await;
    }

    pub async fn run(mut self) -> ! {
        loop {
            self.step().await;
        }
    }
}
