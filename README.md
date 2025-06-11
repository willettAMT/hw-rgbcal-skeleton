# rgbcal: RGB LED calibration tool
Bart Massey 2024

This tool is designed to find out a decent frame rate and
maximum RGB component values to produce a white-looking RGB
of reasonable brightness.

See below for UI.

**XXX This tool is *mostly* finished! Please wire your
hardware up (see below), finish it, comment it, and use it
to find good values. Then document those values in this
README.**

## Build and Run

Run with `cargo embed --release`. You'll need `cargo embed`, as
`cargo run` / `probe-rs run` does not reliably maintain a
connection for printing. See
https://github.com/probe-rs/probe-rs/issues/1235 for the
details.

## Wiring

Connect the RGB LED to the MB2 as follows:

* Red to P9 (GPIO1)
* Green to P8 (GPIO2)
* Blue to P16 (GPIO3)
* Gnd to Gnd

Connect the potentiometer (knob) to the MB2 as follows:

* Pin 1 to Gnd
* Pin 2 to P2
* Pin 3 to +3.3V

## UI

The knob controls the individual settings: frame rate and
color levels. Which parameter the knob controls should be
determined by which buttons are held. (Right now, the knob
jus always controls Blue. You should see the color change
from green to teal-blue as you turn the knob clockwise.)

* No buttons held: Change the frame rate in steps of 10
  frames per second from 10..160.
* A button held: Change the blue level from off to on over
  16 steps.
* B button held: Change the green level from off to on over
  16 steps.
* A+B buttons held: Change the red level from off to on over
  16 steps.

The "frame rate" (also known as the "refresh rate") is the
time to scan out all three colors. (See the scanout code.)
At 30 frames per second, every 1/30th of a second the LED
should scan out all three colors. If the frame rate is too
low, the LED will appear to "blink". If it is too high, it
will eat CPU for no reason.

I think the frame rate is probably set higher than it needs
to be right now: it can be tuned lower.

**LED Specifications**

[LED Wiring Diagram](https://docs.sunfounder.com/projects/sf-components/en/latest/component_rgb_led.html#:~:text=We%20use%20the%20common%20cathode%20one.&text=An%20RGB%20LED%20has%204,%2C%20GND%2C%20Green%20and%20Blue)

---

# Aaron Willett<br>CS 410: Embedded Rust
# Micro:bit v2 RGB Calibration: My Implementation Journey 🌈

## What I Completed 🎯

I finished implementing the RGB LED calibration tool for the BBC micro:bit v2! This project involved completing a partially-finished codebase that controls an external RGB LED using a potentiometer knob and the micro:bit's built-in buttons. The goal was to create a tool that helps find optimal frame rates and RGB values for producing clean white light output.

![RGB LED Setup](https://docs.sunfounder.com/projects/sf-components/en/latest/_images/image216.png)

## The Hardware Experience 🔧

### Breadboard Wiring: Grown-Up Legos! 🧱

One of the most enjoyable parts of this project was wiring up the breadboard - it felt like playing with grown-up Legos! The connections were relatively straightforward:

- **RGB LED**: Red to P9, Green to P8, Blue to P16, with a common ground
- **Potentiometer**: Simple three-pin setup with power, ground, and signal to P2

There's something deeply satisfying about making physical connections that directly translate to software control. Watching the LED actually respond to knob turns and button presses felt magical! ✨

## The Code Journey 📝

### Understanding vs. Creating 🔍

Unlike the DROP assignment where I built everything from scratch, this project was more about understanding and completing existing code. The challenge wasn't so much "How do I make this work?" but rather "How does this already work, and what's missing?"

The existing codebase had a solid foundation:
- Modular design with separate files for knob input, RGB control, and UI logic
- Embassy-based async architecture for smooth concurrent operation
- Shared state management using mutexes

My job was to:
1. **Complete the UI logic**: Implement the button-to-parameter mapping
2. **Fix the control flow**: Ensure knob values properly updated the right parameters
3. **Add comprehensive documentation**: Make the code maintainable and understandable

### The Implementation Details 🛠️

The core challenge was implementing the parameter selection logic in `ui.rs`:

```rust
fn read_button_state(&mut self) -> ControlParameter {
    let a_pressed = self.button_a.is_low();
    let b_pressed = self.button_b.is_low();

    match (a_pressed, b_pressed) {
        (false, false) => ControlParameter::FrameRate,
        (true, false) => ControlParameter::Blue,
        (false, true) => ControlParameter::Green,
        (true, true) => ControlParameter::Red,
    }
}
```

This simple pattern matching elegantly handles all four control modes. The beauty is in how the knob seamlessly switches between controlling frame rate (10-160 FPS) and RGB intensity levels (0-15) based on button state.

## Documentation: The Real Challenge 📚

### The Documentation Dilemma 🤔

Writing good documentation comments turned out to be more challenging than I expected! I found myself constantly questioning:

- **What to include?** How much detail is helpful vs. overwhelming?
- **How much is too much?** When does documentation become verbose noise?
- **Technical accuracy?** Am I explaining the PWM timing correctly? Are my examples realistic?

I went through several iterations, starting with overly verbose documentation that explained every tiny detail, then paring it back to focus on what users actually need to know.

### The Magic of `cargo doc --open` ✨

The most fascinating discovery was running `cargo doc --open` and seeing my documentation come to life! The generated HTML documentation was beautiful - cross-referenced, searchable, and professionally formatted. It made me wish every programming language had this kind of built-in documentation system. 

```bash
$ cargo doc --open
# Opens a beautiful HTML interface showing all my docs!
```

Seeing the module-level documentation, function descriptions, and examples all properly formatted and linked together was incredibly satisfying. It really drove home the value of taking time to write good documentation.

## The Technical Deep Dive 🔬

### Software PWM Implementation

The RGB control uses software-based PWM that's surprisingly elegant:

```rust
async fn step(&mut self, led: usize) {
    let level = self.levels[led];
    if level > 0 {
        self.rgb[led].set_high();
        let on_time = level as u64 * self.tick_time;
        Timer::after_micros(on_time).await;
        self.rgb[led].set_low();
    }
    // ... off-time handling
}
```

The timing calculation ensures smooth color blending:
```
tick_time = 1_000_000 / (3 * frame_rate * LEVELS)
```

### Async Architecture Benefits

Using Embassy's async system made the concurrent operation seamless - the RGB controller runs independently while the UI processes button and knob inputs. No complex interrupt handling needed!

## What I Learned 🧠

1. **Reading Code is a Skill**: Understanding existing codebases requires different skills than writing from scratch. You need to trace data flow and understand architectural decisions.

2. **Documentation is an Art**: Good documentation strikes a balance between completeness and clarity. It's harder than it looks!

3. **Hardware Abstraction Layers Rock**: The microbit-bsp crate made hardware access feel natural and safe. No register manipulation needed!

4. **Modular Design Pays Off**: Having separate modules for knob, RGB, and UI made the code much easier to understand and modify.

5. **`cargo doc` is Amazing**: This tool should exist in every language. The automatically generated, cross-referenced documentation is invaluable.

## The Final Result 🎉

The completed tool provides smooth, real-time control over:
- **Frame rate**: 10-160 FPS for finding the sweet spot between smoothness and efficiency
- **RGB levels**: 0-15 intensity for each color channel
- **Immediate feedback**: Console output shows current values as you adjust them

It's incredibly satisfying to turn the knob and see the LED smoothly transition through colors, or adjust the frame rate and watch the PWM timing adapt in real-time!

## Try It Yourself! 🚀

Want to build your own RGB calibration tool? You'll need:
- BBC micro:bit v2
- RGB LED (common cathode)
- 10kΩ potentiometer
- Breadboard and jumper wires
- Rust toolchain with embedded targets

Then just:
```bash
cargo embed --release
```

Start turning that knob and pressing those buttons - you've got yourself a professional LED calibration tool! 🎨
