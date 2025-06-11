//! # RGB LED Control Module
//!
//! This module provides software-based PWM (Pulse Width Modulation) control for RGB LEDs
//! with dynamic frame rate adjustment. It implements a time-sliced PWM system that can
//! adjust both individual LED intensities and overall refresh rate in real-time.
//!
//! ## PWM Implementation
//!
//! The module uses a software PWM approach where each LED is controlled individually:
//! - **Time Slicing**: Each frame is divided into multiple time slices per LED
//! - **Intensity Control**: LED on-time within each slice determines brightness (0-15)
//! - **Smooth Transitions**: Fine-grained timing provides smooth color blending
//!
//! ## Frame Rate System
//!
//! Frame rate determines how frequently the entire RGB cycle repeats:
//! - **Dynamic Adjustment**: Frame rate can be changed during runtime (10-160 FPS)
//! - **Real-time Updates**: Changes take effect immediately without restarting
//! - **Efficient Detection**: Only recalculates timing when frame rate actually changes
//!
//! ## Timing Calculation
//!
//! The PWM timing is calculated as:
//! ```text
//! tick_time = 1_000_000 / (3 * frame_rate * LEVELS)
//! ```
//! Where:
//! - `1_000_000`: Microseconds per second
//! - `3`: Number of LEDs (Red, Green, Blue)
//! - `frame_rate`: Target FPS (10-160)
//! - `LEVELS`: Intensity levels (16, giving 0-15 range)
//!
//! ## Hardware Integration
//!
//! - **LED Pins**: Direct GPIO control of RGB LED pins
//! - **Timing**: Microsecond-precision delays using Embassy timers
//! - **Shared State**: Reads RGB levels and frame rate from shared memory
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! let rgb_pins = [red_pin, green_pin, blue_pin];
//! let rgb = Rgb::new(rgb_pins, 60); // 60 FPS initial rate
//! rgb.run().await; // Start the RGB control loop
//! ```
use crate::*;

/// Type alias for the RGB LED pin array.
///
/// Represents the three GPIO output pins that control the RGB LED:
/// - Index 0: Red LED pin
/// - Index 1: Green LED pin  
/// - Index 2: Blue LED pin
///
/// Each pin is configured as a standard output with low initial state.
type RgbPins = [Output<'static, AnyPin>; 3];
/// RGB LED controller using software PWM.
///
/// Manages three LEDs with individual intensity control and configurable
/// frame rate. Reads RGB levels and frame rate from shared state.
pub struct Rgb {
    /// GPIO pins for RGB LEDs [red, green, blue].
    rgb: RgbPins,
    /// Cached RGB intensity levels (0 to [`LEVELS`]-1).
    levels: [u32; 3],
    /// PWM timing interval in microseconds.
    tick_time: u64,
    /// Current frame rate for change detection.
    current_frame_rate: u64,
}

impl Rgb {
    /// Calculates PWM timing for the given frame rate.
    ///
    /// # Formula
    /// ```rust no_run
    /// tick_time = 1_000_000 / (3 * frame_rate * LEVELS)
    /// ```
    ///
    /// # Arguments
    /// * `frame_rate` - Target refresh rate in FPS
    ///
    /// # Returns
    /// PWM tick time in microseconds
    fn frame_tick_time(frame_rate: u64) -> u64 {
        1_000_000 / (3 * frame_rate * LEVELS as u64)
    }
    /// Creates a new RGB controller.
    ///
    /// # Arguments
    /// * `rgb` - Array of GPIO output pins [red, green, blue]
    /// * `frame_rate` - Initial frame rate in FPS
    ///
    /// # Examples
    /// ```rust,no_run
    /// let rgb_pins = [red_pin, green_pin, blue_pin];
    /// let rgb = Rgb::new(rgb_pins, 60);
    /// ```
    pub fn new(rgb: RgbPins, frame_rate: u64) -> Self {
        let tick_time = Self::frame_tick_time(frame_rate);
        Self {
            rgb,
            levels: [0; 3],
            tick_time,
            current_frame_rate: frame_rate,
        }
    }
    /// Executes one PWM cycle for a single LED.
    ///
    /// This is the core PWM implementation that controls LED brightness through
    /// time-based on/off control. The LED is turned on for a duration proportional
    /// to the desired intensity, then turned off for the remaining time.
    ///
    /// # PWM Algorithm
    ///
    /// 1. **On Phase**: Turn LED on for `(intensity * tick_time)` microseconds
    /// 2. **Off Phase**: Turn LED off for `((LEVELS - intensity) * tick_time)` microseconds
    ///
    /// # Arguments
    ///
    /// * `led` - LED index (0=Red, 1=Green, 2=Blue)
    ///
    /// # Timing Behavior
    ///
    /// - **Intensity 0**: LED stays off for full cycle
    /// - **Intensity 15**: LED stays on for full cycle  
    /// - **Intensity 8**: LED on for 50% of cycle time
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// // For intensity level 10 out of 15:
    /// // ON time:  10 * tick_time microseconds
    /// // OFF time: 5 * tick_time microseconds  
    /// self.step(0).await; // Execute PWM cycle for red LED
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Uses async timers for precise microsecond timing
    /// - Skips timing delays when intensity is 0 or max for efficiency
    /// - Each call completes one full PWM cycle for the specified LED
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
    /// Main RGB control loop.
    ///
    /// Continuously updates RGB levels and frame rate from shared state,
    /// then executes PWM cycles for all three LEDs.
    ///
    /// # Operation
    /// 1. Read current RGB levels from shared state
    /// 2. Check for frame rate changes and update timing if needed
    /// 3. Execute PWM cycle for each LED in sequence
    /// 4. Repeat
    ///
    /// This function never returns under normal operation.
    ///
    /// # Never Returns
    ///
    /// This function runs indefinitely under normal operation. It will only
    /// exit if the hardware fails or the system panics.
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
