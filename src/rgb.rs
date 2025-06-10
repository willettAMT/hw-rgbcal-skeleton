use crate::*;

type RgbPins = [Output<'static, AnyPin>; 3];

pub struct Rgb {
    rgb: RgbPins,
    // Shadow variables to minimize lock contention.
    levels: [u32; 3],
    tick_time: u64,
    current_frame_rate: u64,
}

impl Rgb {
    fn frame_tick_time(frame_rate: u64) -> u64 {
        1_000_000 / (3 * frame_rate * LEVELS as u64)
    }

    pub fn new(rgb: RgbPins, frame_rate: u64) -> Self {
        let tick_time = Self::frame_tick_time(frame_rate);
        Self {
            rgb,
            levels: [0; 3],
            tick_time,
            current_frame_rate: frame_rate,
        }
    }

    async fn step(&mut self, led: usize) {
        let level = self.levels[led];
        if level > 0 {
            self.rgb[led].set_high();
            let on_time = level as u64 * self.tick_time;
            Timer::after_micros(on_time).await;
            self.rgb[led].set_low();
        }
        let level = LEVELS - level;
        if level > 0 {
            let off_time = level as u64 * self.tick_time;
            Timer::after_micros(off_time).await;
        }
    }

    pub async fn run(mut self) -> ! {
        loop {
            self.levels = get_rgb_levels().await;

            let new_frame_rate = get_frame_rate().await;
            if new_frame_rate != self.current_frame_rate {
                self.current_frame_rate = new_frame_rate;
                self.tick_time = Self::frame_tick_time(new_frame_rate);
                rprintln!("RGB: Frame rate updated to {} fps", new_frame_rate);
            }
            for led in 0..3 {
                self.step(led).await;
            }
        }
    }
}
