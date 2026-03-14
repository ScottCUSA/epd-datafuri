//! SSD1680 ePaper Display Driver
//!
//! Used in the [WeAct 2.13" Tri-Color display](https://www.aliexpress.com/item/1005004644515880.html)
//! or [Adafruit ThinkInk 2.9" Mono / 4 Grayscale display](https://www.adafruit.com/product/4777)
//!
//! For a complete example see [the example](https://github.com/mbv/esp32-ssd1680/blob/main/src/main.rs).
//!
//! This driver is loosely modeled after the
//! [epd-waveshare](https://github.com/caemor/epd-waveshare) drivers but built for my needs.
//!
//! ## Architecture
//!
//! This driver separates hardware control from graphics rendering:
//! - **Driver structs** (`ThinkInk2in9Mono`, `MagTag2in9`) handle hardware interface and controller commands
//! - **Graphics structs** (`Display2in9Mono`, `Display2in9Gray2`) handle frame buffers and embedded-graphics integration
//!
//! This allows multiple graphics buffers (e.g., for grayscale planes) to share a single hardware driver.
//!
//! ## Usage
//!
//! ### Adafruit ThinkInk 2.9" Display (Mono/Grayscale, SSD1680)
//!
//! ```rust, ignore
//! use epd_datafuri::displays::adafruit_thinkink_290_mfgn::{ThinkInk2in9Mono, Display2in9Mono};
//!
//! // Create driver and graphics buffer
//! let mut driver = ThinkInk2in9Mono::new(spi, busy, dc, rst)?;
//! let mut display = Display2in9Mono::new();
//!
//! // Draw and update in one command
//! driver.update_and_display(display.buffer(), &mut delay)?;
//! ```
//!
//! For advanced use cases, you can also use the individual `update_bw()`, `update_red()`,
//! `update()`, and `update_display()` methods for more granular control.
//!
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

/// Maximum display height this driver supports
pub const MAX_HEIGHT: u16 = 296;

/// Maximum display width this driver supports
pub const MAX_WIDTH: u16 = 176;

pub mod interface;

/// Useful exports
pub mod prelude {
    pub use crate::color::Color;
    pub use crate::driver::EpdDriver;

    #[cfg(feature = "graphics")]
    pub use crate::graphics::{Display, DisplayRotation};
}
