//! # User Interface Module
//!
//! This module handles user input processing using a knob (potentiometer) and
//! two buttons to control RGB LED parameters and frame rate.
//!
//! ## Control Scheme
//!
//! - **No buttons**: Knob controls frame rate (10-160 FPS)
//! - **Button A**: Knob controls blue LED intensity (0-15)
//! - **Button B**: Knob controls green LED intensity (0-15)  
//! - **Both buttons**: Knob controls red LED intensity (0-15)
use crate::*;

/// Represents which parameter the knob is currently controlling.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ControlParameter {
    /// Frame rate control (no buttons pressed)
    FrameRate,
    /// Blue LED intensity (button A pressed)
    Blue,
    /// Green LED intensity (button B pressed)
    Green,
    /// Red LED intensity (both buttons pressed)
    Red,
}

/// Internal state for th e UI control system.
///
/// This struct maintains the current values for all controllable parameters.
/// It serves as a local cache to minimize shared state access and provides
/// the source of truth for UI display.
///
/// # Fields
///
/// -'levels': RGB intensity values [red, green, blue] ranging from 0-15
/// -'frame_rate': Display refresh rate in FPS, ranging from 10-160
///
/// # Examples
///
/// ```rust,no_run
/// let state = UiState {
///     levels: [10, 8, 12],    // Red=10, Green=8, Blue=12
///     frame_rate: 60,         // 60 FPS
/// };
/// ```
struct UiState {
    /// RGB intensity levels [red, green, blue] with values from 0-15.
    ///
    /// Each element corresponds to the intensity of the repsective color channel:
    /// - Index 0: Red intensity (0 = off, 15 = maximum)
    /// - Index 1: Green intensity (0 = off, 15 = maximum)
    /// - Index 2: Blue intensity (0 = off, 15 = maximum)
    levels: [u32; 3],
    /// Display refresh rate in frames per second (10-160 FPS).
    ///
    /// Controls how frequently the RGB LEDs are update. Higher values
    /// provide smoother visual transitions but increase power consumption.
    frame_rate: u64,
}

impl UiState {
    /// Displays the current UI state to the debug console.
    ///
    /// Outputs a formatted display of all current parameter values including
    /// RGB levels and frame rate. This provides real-time feedback about
    /// the system state for debugging and user confirmation.
    ///
    /// # Output Format
    ///
    /// ```text
    /// === RGB Calibration ===
    /// red: 10
    /// green: 8  
    /// blue: 12
    /// frame rate: 60
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let state = UiState { levels: [10, 8, 12], frame_rate: 60 };
    /// state.show(); // Prints current values to console
    /// ```
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
/// User interface controller that processes knob and button inputs.
///
/// Manages the mapping between button states and controllable parameters,
/// reads knob values, and updates shared state for the RGB controller.
pub struct Ui {
    knob: Knob,
    button_a: Button,
    button_b: Button,
    state: UiState,
    current_parameter: ControlParameter,
}

impl Ui {
    /// User interface controller that processes knob and button inputs.
    ///
    /// Manages the mapping between button states and controllable parameters,
    /// reads knob values, and updates shared state for the RGB controller.
    pub fn new(knob: Knob, button_a: Button, button_b: Button) -> Self {
        Self {
            knob,
            button_a,
            button_b,
            state: UiState::default(),
            current_parameter: ControlParameter::FrameRate,
        }
    }
    /// Reads button state and determines which parameter to control.
    ///
    /// # Returns
    /// The active control parameter based on button combination:
    /// - No buttons: Frame rate
    /// - A only: Blue LED
    /// - B only: Green LED  
    /// - A + B: Red LED
    fn read_button_state(&mut self) -> ControlParameter {
        let a_pressed = self.button_a.is_low();
        let b_pressed = self.button_b.is_low();

        match (a_pressed, b_pressed) {
            (false, false) => ControlParameter::FrameRate, // No buttons
            (true, false) => ControlParameter::Blue,       // A button
            (false, true) => ControlParameter::Green,      // B button
            (true, true) => ControlParameter::Red,         // Both A+B buttons
        }
    }
    /// Maps knob value (0-15) to appropriate parameter range.
    ///
    /// # Arguments
    /// * `knob_value` - Raw knob reading (0-15)
    /// * `parameter` - Target parameter to map to
    ///
    /// # Returns
    /// Mapped value in the appropriate range:
    /// - Frame rate: 10-160 FPS
    /// - RGB: 0-15 (unchanged)
    fn map_knob_value(&self, knob_value: u32, parameter: ControlParameter) -> u32 {
        match parameter {
            ControlParameter::FrameRate => 10 + (knob_value * 10),
            ControlParameter::Blue | ControlParameter::Green | ControlParameter::Red => knob_value,
        }
    }
    /// Main UI control loop that handles input processing and state management.
    ///
    /// This is the primary entry point for the UI system. It runs continuously,
    /// processing button and knob inputs, managing parameter selection, and
    /// synchronizing state with the RGB display system.

    /// # Value Ranges
    ///
    /// - **RGB Parameters**: 0-15 (mapped from knob input)
    /// - **Frame Rate**: 10-160 FPS (mapped from knob input)
    ///
    /// # Performance Considerations
    ///
    /// - Uses change detection to minimize shared state updates
    /// - Local state caching reduces lock contention
    /// - 50ms loop delay balances responsiveness with CPU usage
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let mut ui = Ui::new(knob, btn_a, btn_b);
    /// ui.run().await; // Starts the UI control loop (never returns)
    /// ```
    ///
    /// # Panics
    ///
    /// This function never returns under normal operation. It will only
    /// exit if the hardware fails or the system panics.
    pub async fn run(&mut self) -> ! {
        self.state.levels[2] = self.knob.measure().await;
        set_rgb_levels(|rgb| {
            *rgb = self.state.levels;
        })
        .await;
        self.state.show();
        loop {
            let parameter = self.read_button_state();

            if parameter != self.current_parameter {
                self.current_parameter = parameter;
                rprintln!("Now controlling: {:?}", parameter);
                self.state.show();
            }

            let raw_knob_value = self.knob.measure().await;
            let mapped_value = self.map_knob_value(raw_knob_value, parameter);
            let mut changed = false;

            match parameter {
                ControlParameter::FrameRate => {
                    let new_frame_rate: u64 = mapped_value.into();
                    if new_frame_rate != self.state.frame_rate {
                        self.state.frame_rate = new_frame_rate;
                        changed = true;
                    }
                }
                ControlParameter::Red => {
                    if mapped_value != self.state.levels[0] {
                        self.state.levels[0] = mapped_value;
                        changed = true;
                    }
                }
                ControlParameter::Green => {
                    if mapped_value != self.state.levels[1] {
                        self.state.levels[1] = mapped_value;
                        changed = true;
                    }
                }
                ControlParameter::Blue => {
                    if mapped_value != self.state.levels[2] {
                        self.state.levels[2] = mapped_value;
                        changed = true;
                    }
                }
            }

            if changed {
                self.state.show();

                set_rgb_levels(|rgb| {
                    *rgb = self.state.levels;
                })
                .await;

                if matches!(parameter, ControlParameter::FrameRate) {
                    set_frame_rate(|rate| *rate = self.state.frame_rate).await;
                    rprintln!("Frame rate changed to : {} fps", self.state.frame_rate);
                }
            }
            Timer::after_millis(50).await;
        }
    }
}
