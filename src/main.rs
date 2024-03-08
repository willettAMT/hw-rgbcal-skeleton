#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use num_traits::float::FloatCore;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_sync::{
    mutex::Mutex,
    blocking_mutex::raw::ThreadModeRawMutex,
};
use embassy_futures::join;
use microbit_bsp::{
    embassy_nrf::{
        bind_interrupts,
        gpio::{AnyPin, Level, Output, OutputDrive},
        saadc,
    },
    Microbit,
    Button,
};

type Adc = saadc::Saadc<'static, 1>;

struct Knob(Adc);

impl Knob {
    async fn new(adc: Adc) -> Self {
        adc.calibrate().await;
        Self(adc)
    }

    async fn measure(&mut self) -> u32 {
        let mut buf = [0];
        self.0.sample(&mut buf).await;
        let raw = buf[0].clamp(0, 0x7fff) as u16;
        let scaled = raw as f32 / 10_000.0;
        let result = ((LEVELS + 2) as f32  * scaled - 2.0)
            .clamp(0.0, (LEVELS - 1) as f32)
            .floor();
        result as u32
    }
}

type RgbPins = [Output<'static, AnyPin>; 3];

static RGB_LEVELS: Mutex<ThreadModeRawMutex, [u32; 3]> = Mutex::new([0; 3]);
const LEVELS: u32 = 16;

async fn get_rgb_levels() -> [u32; 3] {
    let rgb_levels = RGB_LEVELS.lock().await;
    *rgb_levels
}

async fn set_rgb_levels<F>(setter: F)
    where F: FnOnce(&mut [u32; 3])
{
    let mut rgb_levels = RGB_LEVELS.lock().await;
    setter(&mut rgb_levels);
}

struct Rgb {
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

    fn new(rgb: RgbPins, frame_rate: u64) -> Self {
        let tick_time = Self::frame_tick_time(frame_rate);
        Self { rgb, levels: [0; 3], tick: 0, tick_time }
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

    async fn run(mut self) -> ! {
        loop {
            self.step().await;
        }
    }
}

struct UiState {
    levels: [u32; 3],
    frame_rate: u64,
}

impl UiState {
    fn show(&self) {
        let names = ["red", "green", "blue"];
        rprintln!();
        for (name, level) in names.iter().zip(self.levels.iter()) {
            rprintln!("{}: {}", name, level);
        }
        rprintln!("frame rate: {}", self.frame_rate);
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            levels: [LEVELS - 1, LEVELS - 1, LEVELS - 1],
            frame_rate: 100,
        }
    }
}

struct Ui {
    knob: Knob,
    _button_a: Button,
    _button_b: Button,
    state: UiState,
}

impl Ui {
    fn new(knob: Knob, _button_a: Button, _button_b: Button) -> Self {
        Self { knob, _button_a, _button_b, state: UiState::default() }
    }

    async fn run(&mut self) -> ! {
        self.state.levels[2] = self.knob.measure().await;
        set_rgb_levels(|rgb| {
            *rgb = self.state.levels;
        }).await;
        self.state.show();
        loop {
            let level = self.knob.measure().await;
            if level != self.state.levels[2] {
                self.state.levels[2] = level;
                self.state.show();
                set_rgb_levels(|rgb| {
                    *rgb = self.state.levels;
                }).await;
            }
            Timer::after_millis(50).await;
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    rtt_init_print!();
    let board = Microbit::default();

    bind_interrupts!(struct Irqs {
        SAADC => saadc::InterruptHandler;
    });

    let led_pin = |p| Output::new(p, Level::Low, OutputDrive::Standard);
    let red = led_pin(AnyPin::from(board.p9));
    let green = led_pin(AnyPin::from(board.p8));
    let blue = led_pin(AnyPin::from(board.p16));
    let rgb: Rgb = Rgb::new([red, green, blue], 100);

    let mut saadc_config = saadc::Config::default();
    saadc_config.resolution = saadc::Resolution::_14BIT;
    let saadc = saadc::Saadc::new(
        board.saadc,
        Irqs,
        saadc_config,
        [saadc::ChannelConfig::single_ended(board.p2)],
    );
    let knob = Knob::new(saadc).await;
    let mut ui = Ui::new(knob, board.btn_a, board.btn_b);

    join::join(
        rgb.run(),
        ui.run(),
    ).await;

    panic!("fell off end of main loop");
}
