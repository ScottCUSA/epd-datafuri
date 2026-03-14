//! 2.9" display graphics b/w buffer
use display_interface::DisplayError;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};

use crate::color::Color;
use crate::prelude::Display;

use crate::graphics::buffer_len;
use crate::graphics::DisplayRotation;

/// Display width of the MagTag 2.9" IL0373 and SSD1680 panel in pixels.
pub const WIDTH: u16 = 128;
/// Display height of the MagTag 2.9" IL0373 and SSD1680  panel in pixels.
pub const HEIGHT: u16 = 296;

/// Graphics buffer for the 2.9" display
pub struct Display2in9Mono {
    buffer: [u8; buffer_len(WIDTH as usize, HEIGHT as usize)],
    rotation: DisplayRotation,
    is_inverted: bool,
}

impl Default for Display2in9Mono {
    fn default() -> Self {
        Self::new()
    }
}

impl Display2in9Mono {
    /// Create a new black and white graphics buffer for the 2.9" display
    pub fn new() -> Self {
        Self {
            buffer: [Color::White.get_byte_value(); buffer_len(WIDTH as usize, HEIGHT as usize)],
            rotation: DisplayRotation::Rotate270,
            is_inverted: false,
        }
    }
}

impl DrawTarget for Display2in9Mono {
    type Error = DisplayError;
    type Color = BinaryColor;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for p in pixels.into_iter() {
            self.draw_helper(WIDTH.into(), HEIGHT.into(), p)?;
        }
        Ok(())
    }
}

impl OriginDimensions for Display2in9Mono {
    fn size(&self) -> Size {
        //if display is rotated 90 deg or 270 then swap height and width
        match self.rotation() {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                Size::new(WIDTH.into(), HEIGHT.into())
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                Size::new(HEIGHT.into(), WIDTH.into())
            }
        }
    }
}

impl Display for Display2in9Mono {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn get_mut_buffer(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }

    fn rotation(&self) -> DisplayRotation {
        self.rotation
    }

    fn is_inverted(&self) -> bool {
        self.is_inverted
    }
}

#[cfg(test)]
mod tests {
    use super::{Display, Display2in9Mono, DisplayRotation, HEIGHT, WIDTH};
    use crate::color::{Black, Color};
    use crate::graphics::{find_position, outside_display};
    use embedded_graphics::{prelude::*, primitives::Line, primitives::PrimitiveStyle};

    #[test]
    fn buffer_clear() {
        let mut display = Display2in9Mono::new();

        for &byte in display.buffer().iter() {
            assert_eq!(byte, Color::White.get_byte_value());
        }

        display.clear_buffer(Color::Black);

        for &byte in display.buffer().iter() {
            assert_eq!(byte, Color::Black.get_byte_value());
        }
    }

    #[test]
    fn rotation_overflow() {
        let width = WIDTH as u32;
        let height = HEIGHT as u32;
        test_rotation_overflow(width, height, DisplayRotation::Rotate0);
        test_rotation_overflow(width, height, DisplayRotation::Rotate90);
        test_rotation_overflow(width, height, DisplayRotation::Rotate180);
        test_rotation_overflow(width, height, DisplayRotation::Rotate270);
    }

    fn test_rotation_overflow(width: u32, height: u32, rotation: DisplayRotation) {
        let max_value = width.div_ceil(8) * height;
        for x in 0..(width + height) {
            for y in 0..u32::MAX {
                if outside_display(Point::new(x as i32, y as i32), width, height, rotation) {
                    break;
                } else {
                    let (idx, _) = find_position(x, y, width, height, rotation);
                    assert!(idx < max_value, "{idx} !< {max_value}",);
                }
            }
        }
    }

    #[test]
    fn graphics_rotation_0() {
        let mut display = Display2in9Mono::new();
        display.set_rotation(DisplayRotation::Rotate0);

        let _ = Line::new(Point::new(0, 0), Point::new(7, 0))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

        let buffer = display.buffer();
        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, Color::White.get_byte_value());
        }
    }
}
