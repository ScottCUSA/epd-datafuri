//! Rust driver for Adafruit e-Paper displays (EPD), for use with [embedded-hal].
//!
//! ## Supported Displays
//!
//! | Display | Controller | Colors | Grayscale |
//! |---------|-----------|--------|-----------|
//! | Adafruit ThinkInk 2.9" EAAMFGN (2025 MagTag) | SSD1680 | BW | Gray2 (2-bit, 4-level) |
//! | Adafruit ThinkInk 2.9" T5 (original MagTag) | IL0373 | BW | Gray2 (2-bit, 4-level) |
//!
//! ## Usage
//!
//! ### ThinkInk 2.9" — Monochrome (SSD1680)
//!
//! ```rust, ignore
//! use epd_datafuri::displays::adafruit_thinkink_290_mfgn::{ThinkInk2in9Mono, Display2in9Mono};
//! use epd_datafuri::prelude::*;
//!
//! let mut driver = ThinkInk2in9Mono::new(spi, busy, dc, rst)?;
//! let mut display = Display2in9Mono::new();
//!
//! // Draw using embedded-graphics
//! Text::new("Hello!", Point::new(10, 20), text_style).draw(&mut display)?;
//!
//! driver.begin(&mut delay)?;
//! driver.update_and_display(display.buffer(), &mut delay)?;
//! ```
//!
//! ### ThinkInk 2.9" — Grayscale (SSD1680)
//!
//! ```rust, ignore
//! use epd_datafuri::displays::adafruit_thinkink_290_mfgn::{ThinkInk2in9Gray2, Display2in9Gray2};
//! use epd_datafuri::prelude::*;
//! use embedded_graphics::pixelcolor::Gray2;
//!
//! let mut driver = ThinkInk2in9Gray2::new(spi, busy, dc, rst)?;
//! let mut display = Display2in9Gray2::new();
//!
//! // Draw using embedded-graphics with 4-level gray
//! Text::new("Hello!", Point::new(10, 20), text_style).draw(&mut display)?;
//!
//! driver.begin(&mut delay)?;
//! driver.update_gray2_and_display(display.high_buffer(), display.low_buffer(), &mut delay)?;
//! ```
//!
//! ### MagTag 2.9" — Grayscale (IL0373)
//!
//! ```rust, ignore
//! use epd_datafuri::displays::adafruit_thinkink_290_t5::{ThinkInk2in9Gray2, Display2in9Gray2};
//! use epd_datafuri::prelude::*;
//! use embedded_graphics::pixelcolor::Gray2;
//!
//! let mut driver = ThinkInk2in9Gray2::new(spi, busy, dc, rst)?;
//! let mut display = Display2in9Gray2::new();
//!
//! driver.begin(&mut delay)?;
//! driver.update_gray2_and_display(display.high_buffer(), display.low_buffer(), &mut delay)?;
//! ```
//!
//! ## Grayscale
//!
//! 2-bit, 4-level grayscale (Gray2) is supported for both displays. Each display module
//! exports its own `Display2in9Gray2` with the correct plane mapping for that controller:
//!
//! - [`displays::adafruit_thinkink_290_mfgn::Display2in9Gray2`] for the ThinkInk 2.9" (SSD1680)
//! - [`displays::adafruit_thinkink_290_t5::Display2in9Gray2`] for the MagTag 2.9" (IL0373)
//!
//! ## Notes
//!
//! Partial updates are not supported.
//!
//! [embedded-hal]: https://crates.io/crates/embedded-hal
#![no_std]
#![deny(missing_docs)]
#![allow(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]

pub mod color;
#[cfg(feature = "graphics")]
pub mod displays;
pub mod driver;
#[cfg(feature = "graphics")]
pub mod graphics;

pub mod interface;

/// Useful exports
pub mod prelude {
    pub use crate::color::Color;
    pub use crate::driver::EpdDriver;

    #[cfg(feature = "graphics")]
    pub use crate::graphics::{Display, DisplayRotation};
}
