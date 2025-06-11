//! # Knob Input Module
//!
//! This module provides analog input handling for potentiometer/knob controls
//! using the nRF52's SAADC (Successive Approximation ADC) peripheral.
//!
//! The knob converts analog voltage readings into discrete levels suitable
//! for controlling RGB LED intensity or other stepped parameters.`
use crate::*;

/// Type alias for a single-channel SAADC configuration.
///
/// Represents the SAADC peripheral configured to read from one analog input channel.
pub type Adc = saadc::Saadc<'static, 1>;

/// Analog knob controller that converts ADC readings to discrete levels.
///
/// Wraps the SAADC peripheral to provide convenient analog input reading
/// with automatic calibration and conversion to discrete level values.
pub struct Knob(Adc);
impl Knob {
    /// Creates a new knob controller and calibrates the ADC.
    ///
    /// # Arguments
    ///
    /// * `adc` - Configured SAADC peripheral
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let adc = saadc::Saadc::new(
    ///     board.saadc,
    ///     Irqs,
    ///     saadc_config,
    ///     [saadc::ChannelConfig::single_ended(board.p2)],
    /// );
    /// let knob = Knob::new(adc).await;
    /// ```
    pub async fn new(adc: Adc) -> Self {
        adc.calibrate().await;
        Self(adc)
    }
    /// Reads the knob position and converts it to a discrete level.
    ///
    /// Samples the ADC and maps the result to a discrete level from 0 to [`LEVELS`]-1.
    /// The mapping includes a small offset to ensure the full range is reachable.
    ///
    /// # Returns
    ///
    /// A value from 0 to ([`LEVELS`]-1) representing the knob position:
    /// - 0: Minimum position
    /// - [`LEVELS`]-1: Maximum position
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let level = knob.measure().await;
    /// // With LEVELS=16, level will be 0-15
    /// println!("Knob at level: {}", level);
    /// ```
    pub async fn measure(&mut self) -> u32 {
        let mut buf = [0];
        self.0.sample(&mut buf).await;
        let raw = buf[0].clamp(0, 0x7fff) as u16;
        let scaled = raw as f32 / 10_000.0;
        let result = ((LEVELS + 2) as f32 * scaled - 2.0)
            .clamp(0.0, (LEVELS - 1) as f32)
            .floor();
        result as u32
    }
}
