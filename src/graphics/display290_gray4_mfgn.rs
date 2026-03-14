//! Grayscale graphics buffer for the SSD1680-based 2.9" display (ThinkInk MFGN / MagTag 2025).
use display_interface::DisplayError;
use embedded_graphics::{
    pixelcolor::{BinaryColor, Gray2},
    prelude::*,
};

use crate::graphics::buffer_len;
use crate::graphics::DisplayRotation;

/// Display width of the MFGN 2.9" panel in pixels.
pub const WIDTH: u16 = 128;
/// Display height of the MFGN 2.9" panel in pixels.
pub const HEIGHT: u16 = 296;

/// Grayscale (2-bit) graphics buffer for the SSD1680-based 2.9" MFGN display.
///
/// Encodes 4-level grayscale across two RAM planes (BW and Red). The SSD1680
/// expects inverted polarity in Gray2 mode, so stored bits are the logical
/// complement of the Gray2 storage value:
///
/// | Gray2 | BW RAM (high) | Red RAM (low) | Displayed |
/// |-------|---------------|---------------|-----------|
/// | 0     | 1             | 1             | Black     |
/// | 1     | 1             | 0             | Light gray|
/// | 2     | 0             | 1             | Dark gray |
/// | 3     | 0             | 0             | White     |
///
/// Pass [`Display2in9Gray2::high_buffer`] to `update_gray2_and_display` as the
/// BW buffer and [`Display2in9Gray2::low_buffer`] as the Red buffer.
pub struct Display2in9Gray2 {
    high_buffer: [u8; buffer_len(WIDTH as usize, HEIGHT as usize)],
    low_buffer: [u8; buffer_len(WIDTH as usize, HEIGHT as usize)],
    rotation: DisplayRotation,
}

impl Default for Display2in9Gray2 {
    fn default() -> Self {
        Self::new()
    }
}

impl Display2in9Gray2 {
    /// Create a new grayscale buffer initialised to white.
    pub fn new() -> Self {
        // Inverted polarity: stored 0x00 = logical white.
        Self {
            high_buffer: [0x00; buffer_len(WIDTH as usize, HEIGHT as usize)],
            low_buffer: [0x00; buffer_len(WIDTH as usize, HEIGHT as usize)],
            rotation: DisplayRotation::Rotate270,
        }
    }

    /// Get a reference to the high buffer (BW RAM plane).
    pub fn high_buffer(&self) -> &[u8] {
        &self.high_buffer
    }

    /// Get a reference to the low buffer (Red RAM plane).
    pub fn low_buffer(&self) -> &[u8] {
        &self.low_buffer
    }

    /// Get mutable access to the high buffer.
    pub fn get_mut_high_buffer(&mut self) -> &mut [u8] {
        &mut self.high_buffer
    }

    /// Get mutable access to the low buffer.
    pub fn get_mut_low_buffer(&mut self) -> &mut [u8] {
        &mut self.low_buffer
    }

    /// Set the rotation used for coordinate transforms.
    pub fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }

    /// Get the current rotation.
    pub fn rotation(&self) -> DisplayRotation {
        self.rotation
    }

    /// Clear both buffers to the given grayscale level.
    pub fn clear_buffer(&mut self, level: Gray2) {
        let storage = level.into_storage();
        // MSB of storage → BW RAM (high), LSB → Red RAM (low); bits inverted for SSD1680.
        let bw_byte = if (storage & 0b10) != 0 { 0x00 } else { 0xFF };
        let red_byte = if (storage & 0b01) != 0 { 0x00 } else { 0xFF };

        self.high_buffer.fill(bw_byte);
        self.low_buffer.fill(red_byte);
    }

    /// Set a single pixel to the given 2-bit grayscale level.
    pub fn set_pixel(&mut self, x: i32, y: i32, level: Gray2) {
        if super::outside_display(Point::new(x, y), WIDTH.into(), HEIGHT.into(), self.rotation) {
            return;
        }
        let (idx_u32, bit) = super::find_position(
            x as u32,
            y as u32,
            WIDTH.into(),
            HEIGHT.into(),
            self.rotation,
        );
        let idx = idx_u32 as usize;

        let storage = level.into_storage();
        // MSB → BW RAM (high), LSB → Red RAM (low); inverted polarity for SSD1680.
        let bw_val = (storage & 0b10) == 0; // inverted: set when MSB is 0
        let red_val = (storage & 0b01) == 0; // inverted: set when LSB is 0

        if bw_val {
            self.high_buffer[idx] |= bit;
        } else {
            self.high_buffer[idx] &= !bit;
        }

        if red_val {
            self.low_buffer[idx] |= bit;
        } else {
            self.low_buffer[idx] &= !bit;
        }
    }

    /// Get an adapter that implements `DrawTarget<BinaryColor>`.
    pub fn as_binary_draw_target(&mut self) -> BinaryDrawTarget<'_> {
        BinaryDrawTarget::new(self)
    }
}

impl DrawTarget for Display2in9Gray2 {
    type Error = DisplayError;
    type Color = Gray2;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            self.set_pixel(point.x, point.y, color);
        }
        Ok(())
    }
}

impl OriginDimensions for Display2in9Gray2 {
    fn size(&self) -> Size {
        match self.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                Size::new(WIDTH.into(), HEIGHT.into())
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                Size::new(HEIGHT.into(), WIDTH.into())
            }
        }
    }
}

/// Adapter that exposes a `BinaryColor` `DrawTarget` view over a `Display2in9Gray2` buffer.
pub struct BinaryDrawTarget<'a>(&'a mut Display2in9Gray2);

impl<'a> BinaryDrawTarget<'a> {
    /// Create a new `BinaryDrawTarget` adapter.
    pub fn new(display: &'a mut Display2in9Gray2) -> Self {
        Self(display)
    }
}

impl DrawTarget for BinaryDrawTarget<'_> {
    type Color = BinaryColor;
    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            let level = match color {
                BinaryColor::On => Gray2::WHITE,
                BinaryColor::Off => Gray2::BLACK,
            };
            self.0.set_pixel(point.x, point.y, level);
        }
        Ok(())
    }
}

impl OriginDimensions for BinaryDrawTarget<'_> {
    fn size(&self) -> Size {
        match self.0.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                Size::new(WIDTH.into(), HEIGHT.into())
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                Size::new(HEIGHT.into(), WIDTH.into())
            }
        }
    }
}
