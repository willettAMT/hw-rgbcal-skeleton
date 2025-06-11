//! # RGB LED Controller for micro:bit
//!
//! This is an embedded Rust application for the BBC micro:bit that provides
//! real-time RGB LED control using a potentiometer knob and button interface.
//!
//! ## Features
//!
//! - **RGB LED Control**: Independent control of red, green, and blue LED channels
//! - **Analog Input**: Potentiometer-based value adjustment via analog pin P2
//! - **Button Interface**: Two-button UI using micro:bit's built-in buttons A and B
//! - **Configurable Frame Rate**: Adjustable PWM refresh rate for smooth LED transitions
//! - **Async Architecture**: Built on Embassy framework for efficient embedded async execution
//!
//! ## Hardware Setup
//!
//! - **Red LED**: Connected to pin P9
//! - **Green LED**: Connected to pin P8  
//! - **Blue LED**: Connected to pin P16
//! - **Potentiometer**: Connected to analog pin P2
//! - **Buttons**: Uses micro:bit's built-in buttons A and B
//!
//! ## Architecture
//!
//! The application uses a modular design with three main components:
//! - [`knob`] module: Handles analog input from potentiometer
//! - [`rgb`] module: Manages RGB LED PWM control
//! - [`ui`] module: Processes button inputs and user interface logic
//!
//! Shared state is managed through async-safe mutexes for thread-safe access
//! across the concurrent tasks.

#![no_std]
#![no_main]

mod knob;
mod rgb;
mod ui;
pub use knob::*;
pub use rgb::*;
pub use ui::*;

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use embassy_executor::Spawner;
use embassy_futures::join;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::Timer;
use microbit_bsp::{
    embassy_nrf::{
        bind_interrupts,
        gpio::{AnyPin, Level, Output, OutputDrive},
        saadc,
    },
    Button, Microbit,
};
use num_traits::float::FloatCore;

/// Global RGB LED intensity levels shared across all tasks.
///
/// This mutex-protected array contains the current intensity values for each LED channel:
/// - Index 0: Red channel intensity (0 to [`LEVELS`]-1)
/// - Index 1: Green channel intensity (0 to [`LEVELS`]-1)  
/// - Index 2: Blue channel intensity (0 to [`LEVELS`]-1)
///
/// The values are used by the RGB module for PWM control and modified by the UI module
/// based on user input from the knob and buttons.
pub static RGB_LEVELS: Mutex<ThreadModeRawMutex, [u32; 3]> = Mutex::new([0; 3]);
/// Global frame rate setting for RGB LED refresh rate.
///
/// This mutex-protected value controls how frequently the RGB LEDs are updated,
/// measured in frames per second (Hz). Higher values provide smoother transitions
/// but increase CPU usage. The frame rate can be adjusted through the UI.
///
/// Default value: 100 Hz
pub static FRAME_RATE: Mutex<ThreadModeRawMutex, u64> = Mutex::new(100);
/// Maximum intensity levels for each RGB channel.
///
/// This constant defines the number of discrete intensity steps available
/// for each LED channel, providing 16 levels from 0 (off) to 15 (maximum brightness).
/// The actual PWM duty cycle is calculated as `level / LEVELS`.
pub const LEVELS: u32 = 16;
/// Retrieves the current RGB LED intensity levels.
///
/// This is a convenience function that safely accesses the shared [`RGB_LEVELS`] state.
///
/// # Returns
///
/// An array of three `u32` values representing the current intensity levels
/// for red, green, and blue channels respectively.
///
/// # Examples
///
/// ```rust,no_run
/// let [red, green, blue] = get_rgb_levels().await;
/// println!("Current RGB: R={}, G={}, B={}", red, green, blue);
/// ```
async fn get_rgb_levels() -> [u32; 3] {
    let rgb_levels = RGB_LEVELS.lock().await;
    *rgb_levels
}

/// Updates the RGB LED intensity levels using a closure.
///
/// This function provides safe, atomic access to modify the shared [`RGB_LEVELS`] state.
/// The provided closure receives a mutable reference to the RGB levels array.
///
/// # Parameters
///
/// * `setter` - A closure that receives `&mut [u32; 3]` to modify the RGB levels
///
/// # Examples
///
/// ```rust,no_run
/// // Set red to maximum, others to zero
/// set_rgb_levels(|levels| {
///     levels[0] = LEVELS - 1;  // Red
///     levels[1] = 0;           // Green  
///     levels[2] = 0;           // Blue
/// }).await;
///
/// // Increment blue channel (with bounds checking)
/// set_rgb_levels(|levels| {
///     if levels[2] < LEVELS - 1 {
///         levels[2] += 1;
///     }
/// }).await;
/// ```
async fn set_rgb_levels<F>(setter: F)
where
    F: FnOnce(&mut [u32; 3]),
{
    let mut rgb_levels = RGB_LEVELS.lock().await;
    setter(&mut rgb_levels);
}
///
/// This is a convenience function that safely accesses the shared [`FRAME_RATE`] state.
///
/// # Returns
///
/// The current frame rate in Hz as a `u64` value.
///
/// # Examples
///
/// ```rust,no_run
/// let current_fps = get_frame_rate().await;
/// println!("Running at {} FPS", current_fps);
/// ```
async fn get_frame_rate() -> u64 {
    let frame_rate = FRAME_RATE.lock().await;
    *frame_rate
}
/// Updates the frame rate setting using a closure.
///
/// This function provides safe, atomic access to modify the shared [`FRAME_RATE`] state.
/// The provided closure receives a mutable reference to the frame rate value.
///
/// # Parameters
///
/// * `setter` - A closure that receives `&mut u64` to modify the frame rate
///
/// # Examples
///
/// ```rust,no_run
/// // Set frame rate to 60 Hz
/// set_frame_rate(|fps| *fps = 60).await;
///
/// // Double the current frame rate
/// set_frame_rate(|fps| *fps *= 2).await;
/// ```
async fn set_frame_rate<F>(setter: F)
where
    F: FnOnce(&mut u64),
{
    let mut frame_rate = FRAME_RATE.lock().await;
    setter(&mut frame_rate);
}
/// Main application entry point.
///
/// Initializes all hardware peripherals and spawns the main application tasks:
///
/// 1. **Hardware Initialization**:
///    - Sets up RTT for debug printing
///    - Configures GPIO pins for RGB LEDs (P9=Red, P8=Green, P16=Blue)
///    - Initializes 14-bit SAADC for analog input on P2
///    - Configures buttons A and B for user input
///
/// 2. **Task Execution**:
///    - Creates and runs the RGB LED control task
///    - Creates and runs the UI input processing task
///    - Both tasks run concurrently using `embassy_futures::join`
///
/// The function runs indefinitely, and if both tasks somehow complete,
/// it will panic with an error message.
///
/// # Parameters
///
/// * `_spawner` - Embassy spawner for creating additional tasks (unused in this implementation)
///
/// # Panics
///
/// - Panics if both the RGB and UI tasks complete unexpectedly
/// - May panic during hardware initialization if peripherals are unavailable
///
/// # Hardware Dependencies
///
/// - BBC micro:bit v2 (or compatible board with nRF52833)
/// - RGB LEDs connected to specified GPIO pins
/// - Potentiometer connected to analog pin P2
/// - Built-in buttons A and B functional
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
    let initial_frame_rate = get_frame_rate().await;
    let rgb: Rgb = Rgb::new([red, green, blue], initial_frame_rate);

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

    join::join(rgb.run(), ui.run()).await;

    panic!("fell off end of main loop");
}
