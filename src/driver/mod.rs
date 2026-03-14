//! Driver trait and chip-specific implementations
pub use display_interface::DisplayError;

pub(crate) mod il0373;
pub(crate) mod ssd1680;

use embedded_hal::delay::DelayNs;

/// Trait defining the driver interface for EPD displays
pub trait EpdDriver {
    /// Reset and initialize the display
    fn init(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError>;

    /// Power down the controller
    fn sleep(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError>;

    /// Send black/white buffer to display
    fn update_bw(&mut self, buffer: &[u8], delay: &mut impl DelayNs) -> Result<(), DisplayError>;

    /// Send red buffer
    fn update_red(&mut self, buffer: &[u8], delay: &mut impl DelayNs) -> Result<(), DisplayError>;

    /// Send both low (bw) and high (red) buffers to display
    fn update(
        &mut self,
        low_buffer: &[u8],
        high_buffer: &[u8],
        delay: &mut impl DelayNs,
    ) -> Result<(), DisplayError>;

    /// Clear the BW RAM
    fn clear_bw_ram(&mut self) -> Result<(), DisplayError>;

    /// Clear the red RAM
    fn clear_red_ram(&mut self) -> Result<(), DisplayError>;

    /// Begin operation - reset and put into low power state
    fn begin(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError>;
}
