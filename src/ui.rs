use crate::*;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ControlParameter {
    FrameRate, // No buttons
    Blue,      // A button
    Green,     // B button
    Red,       // A+B buttons
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

pub struct Ui {
    knob: Knob,
    button_a: Button,
    button_b: Button,
    state: UiState,
    current_parameter: ControlParameter,
}

impl Ui {
    pub fn new(knob: Knob, button_a: Button, button_b: Button) -> Self {
        Self {
            knob,
            button_a,
            button_b,
            state: UiState::default(),
            current_parameter: ControlParameter::FrameRate,
        }
    }

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

    fn map_knob_value(&self, knob_value: u32, parameter: ControlParameter) -> u32 {
        match parameter {
            ControlParameter::FrameRate => 10 + (knob_value * 10),
            ControlParameter::Blue | ControlParameter::Green | ControlParameter::Red => knob_value,
        }
    }

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
                    rprintln!("Frame rate changed to : {} fps", self.state.frame_rate);
                }
            }
            Timer::after_millis(50).await;
        }
    }
}
