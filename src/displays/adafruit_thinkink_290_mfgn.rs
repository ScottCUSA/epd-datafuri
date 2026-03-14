//! Driver and graphics buffers for Adafruit 2.9" e-ink displays using the SSD1680 controller.
//!
//! This module targets the 2025 revision of the Adafruit ThinkInk 2.9" and MagTag boards,
//! which use the SSD1680 e-paper controller. The original MagTag revision uses an IL0373
//! instead; see [`adafruit_thinkink_290_t5`](self::adafruit_thinkink_290_t5).
//!
//! ## Key differences from the IL0373 variant
//!
//! | Property | SSD1680 | IL0373 |
//! |----------|---------|--------|
//! | Busy pin polarity | Active-high (HIGH = busy) | Active-low (LOW = busy) |
//! | LUT registers | Single register (0x32) | Five registers (0x20–0x24) |
//! | RAM address counters | Required | Not used |
//! | Gray2 encoding | Identical two-plane scheme | Identical two-plane scheme |
//!
//! Two driver structs are provided:
//! - [`adafruit_thinkink_290_mfgn::ThinkInk2in9Mono`](self::adafruit_thinkink_290_mfgn::ThinkInk2in9Mono): black/white rendering using the mono full LUT
//! - [`adafruit_thinkink_290_mfgn::ThinkInk2in9Gray2`](self::adafruit_thinkink_290_mfgn::ThinkInk2in9Gray2): 4-level grayscale rendering using the Gray2 LUT

use crate::color::Color;
use crate::driver::ssd1680::{Cmd, Flag};
use crate::driver::EpdDriver;
use crate::interface::SpiDisplayInterface;
use display_interface::DisplayError;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::SpiDevice;
use log::debug;

pub use crate::graphics::display290_gray4_mfgn::Display2in9Gray2;
pub use crate::graphics::display290_mono::Display2in9Mono;

/// Display width for 2.9in display
pub const WIDTH: u16 = 128;
/// Display height for 2.9in display
pub const HEIGHT: u16 = 296;

#[rustfmt::skip]
/// Adafruit ThinkInk 290 EA4MFGN MONO FULL LUT CODE Cmd: 0x32 Size: 0x99,
pub const TI_290_MONOFULL_LUT_CODE: [u8; 153] = [
  0x80,	0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, //VS L0
  0x10, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, //VS L1
  0x80, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, //VS L2
  0x10, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, //VS L3
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //VS L4
  0x14, 0x08, 0x00, 0x00, 0x00, 0x00, 0x01,                               //TP, SR, RP of Group0
  0x0A, 0x0A, 0x00, 0x0A, 0x0A, 0x00, 0x01,                               //TP, SR, RP of Group1
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group2
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group3
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group4
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group5
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group6
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group7
  0x14, 0x08, 0x00, 0x01, 0x00, 0x00, 0x01,                               //TP, SR, RP of Group8
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,                               //TP, SR, RP of Group9
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group10
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group11
  0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x00, 0x00, 0x00                    //FR, XON
];

#[rustfmt::skip]
/// Adafruit ThinkInk 290 EA4MFGN GRAY2 LUT CODE Cmd: 0x32 Size: 0x99,
pub const TI_290MFGN_GRAY2_LUT_CODE: [u8; 153] = [
  0x00, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //VS L0	 //2.28s
  0x20, 0x60, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //VS L1
  0x28, 0x60, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //VS L2
  0x2A, 0x60, 0x15, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //VS L3
  0x00, 0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //VS L4
  0x00, 0x02, 0x00, 0x05, 0x14, 0x00, 0x00,                               //TP, SR, RP of Group0
  0x1E, 0x1E, 0x00, 0x00, 0x00, 0x00, 0x01,                               //TP, SR, RP of Group1
  0x00, 0x02, 0x00, 0x05, 0x14, 0x00, 0x00,                               //TP, SR, RP of Group2
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group3
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group4
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group5
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group6
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group7
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group8
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group9
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group10
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                               //TP, SR, RP of Group11
  0x24, 0x22, 0x22, 0x22, 0x23, 0x32, 0x00, 0x00, 0x00,                   //FR, XON
];

/// Driver for the Adafruit ThinkInk 2.9" monochrome e-ink display (SSD1680 controller).
///
/// The SSD1680 busy pin is active-high (HIGH = busy, LOW = ready).
///
/// Use [`Display2in9Mono`] as the graphics buffer for black/white rendering, or
/// [`ThinkInk2in9Gray2`] with [`Display2in9Gray2`] for 4-level grayscale.
pub struct ThinkInk2in9Mono<SPI, BSY, DC, RST>
where
    SPI: SpiDevice,
    BSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    interface: SpiDisplayInterface<SPI, BSY, DC, RST>,
}

impl<SPI, BSY, DC, RST> ThinkInk2in9Mono<SPI, BSY, DC, RST>
where
    SPI: SpiDevice,
    BSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Create a new ThinkInk 2.9" display
    pub fn new(spi: SPI, busy: BSY, dc: DC, rst: RST) -> Result<Self, DisplayError> {
        let interface = SpiDisplayInterface::new(spi, busy, dc, rst);
        Ok(Self { interface })
    }

    /// Update the whole BW buffer on the display driver
    fn write_bw_ram(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.set_ram_counter(0, 0)?;
        self.interface.cmd_with_data(Cmd::WRITE_BW_DATA, buffer)
    }

    /// Update the whole Red buffer on the display driver
    fn write_red_ram(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.set_ram_counter(0, 0)?;
        self.interface.cmd_with_data(Cmd::WRITE_RED_DATA, buffer)
    }

    fn display(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.interface
            .cmd_with_data(Cmd::DISPLAY_UPDATE_CTRL2, &[Flag::DISPLAY_MODE_1])?;
        self.interface.cmd(Cmd::MASTER_ACTIVATE)?;
        self.interface.wait_until_idle(delay);
        Ok(())
    }

    /// Fill both RAM planes with white and trigger two successive refreshes.
    ///
    /// A double refresh is used to fully clear any image ghosting from the panel.
    pub fn clear_display(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.clear_bw_ram()?;
        self.clear_red_ram()?;
        self.display(delay)?;
        delay.delay_ms(100);
        self.display(delay)
    }

    /// Write the black/white buffer to the display and trigger a full refresh.
    ///
    /// This method calls [`EpdDriver::init`] internally, so there is no need
    /// to call it separately before invoking this method.
    pub fn update_and_display(
        &mut self,
        bw_buffer: &[u8],
        delay: &mut impl DelayNs,
    ) -> Result<(), DisplayError> {
        self.init(delay)?;
        self.update_bw(bw_buffer, delay)?;
        self.display(delay)?;
        self.sleep(delay)
    }

    /// Write both Gray2 framebuffer planes to the display and trigger a full refresh.
    ///
    /// `bw_buffer` is written to the black/white RAM plane and `red_buffer` to the
    /// red RAM plane. Pass [`Display2in9Gray2::high_buffer`] and
    /// [`Display2in9Gray2::low_buffer`] respectively.
    ///
    /// This method calls [`EpdDriver::init`] internally, so there is no need
    /// to call it separately before invoking this method.
    pub fn update_gray2_and_display(
        &mut self,
        bw_buffer: &[u8],
        red_buffer: &[u8],
        delay: &mut impl DelayNs,
    ) -> Result<(), DisplayError> {
        self.init(delay)?;
        self.update_bw(bw_buffer, delay)?;
        self.update_red(red_buffer, delay)?;
        self.display(delay)?;
        self.sleep(delay)
    }

    fn set_ram_counter(&mut self, x: u32, y: u32) -> Result<(), DisplayError> {
        self.interface
            .cmd_with_data(Cmd::SET_RAMX_COUNTER, &[(x >> 3) as u8])?;

        self.interface.cmd_with_data(
            Cmd::SET_RAMY_COUNTER,
            &[(y & 0xFF) as u8, ((y >> 8) & 0x01) as u8],
        )?;
        Ok(())
    }
}

impl<SPI, BSY, DC, RST> EpdDriver for ThinkInk2in9Mono<SPI, BSY, DC, RST>
where
    SPI: SpiDevice,
    BSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Reset and fully initialize the display for monochrome operation.
    ///
    /// Performs a hardware reset, sends the panel configuration, loads the mono
    /// full LUT ([`TI_290_MONOFULL_LUT_CODE`]), and waits for the busy pin to
    /// go low before returning.
    fn init(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        debug!("powering up ThinkInk 2.9in mono display");
        // hard reset
        self.interface.hard_reset(delay)?;
        // Set Initialial Configuration
        self.interface.cmd(Cmd::SW_RESET)?;
        delay.delay_ms(10);
        // Send Initialization Code 0x01, 0x11, 0x44, 0x45, 0x3C
        self.interface
            .cmd_with_data(Cmd::DRIVER_OUTPUT_CTRL, &[0x27, 0x01, 0x00])?;
        self.interface
            .cmd_with_data(Cmd::DATA_ENTRY_MODE, &[Flag::DATA_ENTRY_INCRY_INCRX])?;
        self.interface
            .cmd_with_data(Cmd::SET_RAMXPOS, &[0x00, 0x0F])?;
        self.interface
            .cmd_with_data(Cmd::SET_RAMYPOS, &[0x00, 0x00, 0x27, 0x01])?;
        self.interface.cmd_with_data(
            Cmd::BORDER_WAVEFORM_CTRL,
            &[Flag::BORDER_WAVEFORM_FOLLOW_LUT | Flag::BORDER_WAVEFORM_LUT1],
        )?;
        // Load Waveform LUT
        self.interface
            .cmd_with_data(Cmd::TEMP_CONTROL, &[Flag::INTERNAL_TEMP_SENSOR])?;
        self.interface
            .cmd_with_data(Cmd::DISPLAY_UPDATE_CTRL1, &[0x00, 0x80])?;
        self.interface
            .cmd_with_data(Cmd::END_OPTION, &[Flag::END_OPTION_NORMAL])?;
        self.interface
            .cmd_with_data(Cmd::GATE_VOLTAGE_CTRL, &[0x17])?;
        self.interface
            .cmd_with_data(Cmd::SOURCE_VOLTAGE_CTRL, &[0x41, 0x00, 0x32])?;
        self.interface.cmd_with_data(Cmd::WRITE_VCOM_REG, &[0x36])?;
        // load LUT into memory
        self.interface
            .cmd_with_data(Cmd::WRITE_LUT_REG, &TI_290_MONOFULL_LUT_CODE)?;
        self.interface.wait_until_idle(delay);
        Ok(())
    }

    /// Power down the display controller into deep sleep mode.
    fn sleep(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        debug!("powering down ThinkInk 2.9\" display");
        self.interface.cmd_with_data(Cmd::DEEP_SLEEP, &[0x01])?;
        delay.delay_ms(1);
        Ok(())
    }

    /// Write `buffer` to the black/white RAM plane and wait until ready.
    fn update_bw(&mut self, buffer: &[u8], delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.write_bw_ram(buffer)?;
        self.interface.wait_until_idle(delay);
        Ok(())
    }

    /// Write `buffer` to the red RAM plane and wait until ready.
    fn update_red(&mut self, buffer: &[u8], delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.write_red_ram(buffer)?;
        self.interface.wait_until_idle(delay);
        Ok(())
    }

    /// Write both RAM planes and wait until ready.
    ///
    /// Does not trigger a display refresh; call [`Self::update_and_display`] or
    /// [`Self::update_gray2_and_display`] for a complete write-and-refresh cycle.
    fn update(
        &mut self,
        low_buffer: &[u8],
        high_buffer: &[u8],
        delay: &mut impl DelayNs,
    ) -> Result<(), DisplayError> {
        self.write_bw_ram(low_buffer)?;
        self.write_red_ram(high_buffer)?;
        self.interface.wait_until_idle(delay);
        Ok(())
    }

    /// Fill the black/white RAM plane with white, clearing it.
    fn clear_bw_ram(&mut self) -> Result<(), DisplayError> {
        let color = Color::White.get_byte_value();
        self.interface.cmd(Cmd::WRITE_BW_DATA)?;
        self.interface
            .data_x_times(color, u32::from(WIDTH).div_ceil(8) * u32::from(HEIGHT))?;
        Ok(())
    }

    /// Fill the red RAM plane with its cleared state.
    fn clear_red_ram(&mut self) -> Result<(), DisplayError> {
        let color = Color::White.inverse().get_byte_value();
        self.interface.cmd(Cmd::WRITE_RED_DATA)?;
        self.interface
            .data_x_times(color, u32::from(WIDTH).div_ceil(8) * u32::from(HEIGHT))?;
        Ok(())
    }

    /// Hardware-reset the panel and put it into a low-power sleep state.
    ///
    /// Use this to prepare the display before the first [`Self::init`] call.
    fn begin(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.interface.hard_reset(delay)?;
        self.sleep(delay)
    }
}

/// Driver for the Adafruit ThinkInk 2.9" 4-level grayscale e-ink display (SSD1680 controller).
///
/// The SSD1680 busy pin is active-high (HIGH = busy, LOW = ready).
///
/// Use [`Display2in9Gray2::new`] to create a correctly configured
/// graphics buffer for this driver.
pub struct ThinkInk2in9Gray2<SPI, BSY, DC, RST>
where
    SPI: SpiDevice,
    BSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    interface: SpiDisplayInterface<SPI, BSY, DC, RST>,
}

impl<SPI, BSY, DC, RST> ThinkInk2in9Gray2<SPI, BSY, DC, RST>
where
    SPI: SpiDevice,
    BSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Create a new ThinkInk 2.9" grayscale display
    pub fn new(spi: SPI, busy: BSY, dc: DC, rst: RST) -> Result<Self, DisplayError> {
        let interface = SpiDisplayInterface::new(spi, busy, dc, rst);
        Ok(Self { interface })
    }

    /// Update the whole BW buffer on the display driver
    fn write_bw_ram(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.set_ram_counter(0, 0)?;
        self.interface.cmd_with_data(Cmd::WRITE_BW_DATA, buffer)
    }

    /// Update the whole Red buffer on the display driver
    fn write_red_ram(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.set_ram_counter(0, 0)?;
        self.interface.cmd_with_data(Cmd::WRITE_RED_DATA, buffer)
    }

    fn display(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.interface
            .cmd_with_data(Cmd::DISPLAY_UPDATE_CTRL2, &[Flag::DISPLAY_MODE_1])?;
        self.interface.cmd(Cmd::MASTER_ACTIVATE)?;
        self.interface.wait_until_idle(delay);
        Ok(())
    }

    /// Fill both RAM planes with white and trigger two successive refreshes.
    ///
    /// A double refresh is used to fully clear any image ghosting from the panel.
    pub fn clear_display(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.clear_bw_ram()?;
        self.clear_red_ram()?;
        self.display(delay)?;
        delay.delay_ms(100);
        self.display(delay)
    }

    /// Write the black/white buffer to the display and trigger a full refresh.
    ///
    /// This method calls [`EpdDriver::init`] internally, so there is no need
    /// to call it separately before invoking this method.
    pub fn update_and_display(
        &mut self,
        bw_buffer: &[u8],
        delay: &mut impl DelayNs,
    ) -> Result<(), DisplayError> {
        self.init(delay)?;
        self.update_bw(bw_buffer, delay)?;
        self.display(delay)?;
        self.sleep(delay)
    }

    /// Write both Gray2 framebuffer planes to the display and trigger a full refresh.
    ///
    /// `high_buffer` is written to the black/white RAM plane and `low_buffer` to the
    /// red RAM plane. Pass [`Display2in9Gray2::high_buffer`] and
    /// [`Display2in9Gray2::low_buffer`] respectively.
    ///
    /// This method calls [`EpdDriver::init`] internally, so there is no need
    /// to call it separately before invoking this method.
    pub fn update_gray2_and_display(
        &mut self,
        high_buffer: &[u8],
        low_buffer: &[u8],
        delay: &mut impl DelayNs,
    ) -> Result<(), DisplayError> {
        self.init(delay)?;
        self.update_bw(high_buffer, delay)?;
        self.update_red(low_buffer, delay)?;
        self.display(delay)?;
        self.sleep(delay)
    }

    fn set_ram_counter(&mut self, x: u32, y: u32) -> Result<(), DisplayError> {
        self.interface
            .cmd_with_data(Cmd::SET_RAMX_COUNTER, &[(x >> 3) as u8])?;

        self.interface.cmd_with_data(
            Cmd::SET_RAMY_COUNTER,
            &[(y & 0xFF) as u8, ((y >> 8) & 0x01) as u8],
        )?;
        Ok(())
    }
}

impl<SPI, BSY, DC, RST> EpdDriver for ThinkInk2in9Gray2<SPI, BSY, DC, RST>
where
    SPI: SpiDevice,
    BSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Reset and fully initialize the display for 4-level grayscale operation.
    ///
    /// Performs a hardware reset, sends the panel configuration, loads the Gray2
    /// LUT ([`TI_290MFGN_GRAY2_LUT_CODE`]), and waits for the busy pin to go
    /// low before returning.
    fn init(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        debug!("powering up ThinkInk 2.9in grayscale display");
        // hard reset
        self.interface.hard_reset(delay)?;
        self.interface.cmd(Cmd::SW_RESET)?;
        delay.delay_ms(10);
        // Send Initialization Code 0x01, 0x11, 0x44, 0x45, 0x3C
        self.interface
            .cmd_with_data(Cmd::DRIVER_OUTPUT_CTRL, &[0x27, 0x01, 0x00])?;
        self.interface
            .cmd_with_data(Cmd::DATA_ENTRY_MODE, &[Flag::DATA_ENTRY_INCRY_INCRX])?;
        self.interface
            .cmd_with_data(Cmd::SET_RAMXPOS, &[0x00, 0x0F])?;
        self.interface
            .cmd_with_data(Cmd::SET_RAMYPOS, &[0x00, 0x00, 0x27, 0x01])?;
        self.interface.cmd_with_data(
            Cmd::BORDER_WAVEFORM_CTRL,
            &[Flag::BORDER_WAVEFORM_FOLLOW_LUT | Flag::BORDER_WAVEFORM_LUT1],
        )?;
        // Load Waveform LUT
        self.interface
            .cmd_with_data(Cmd::TEMP_CONTROL, &[Flag::INTERNAL_TEMP_SENSOR])?;
        self.interface
            .cmd_with_data(Cmd::DISPLAY_UPDATE_CTRL1, &[0x00, 0x80])?;
        self.interface
            .cmd_with_data(Cmd::END_OPTION, &[Flag::END_OPTION_NORMAL])?;
        self.interface
            .cmd_with_data(Cmd::GATE_VOLTAGE_CTRL, &[0x17])?;
        self.interface
            .cmd_with_data(Cmd::SOURCE_VOLTAGE_CTRL, &[0x41, 0x00, 0x32])?;
        self.interface.cmd_with_data(Cmd::WRITE_VCOM_REG, &[0x36])?;
        // write LUT into memory
        self.interface
            .cmd_with_data(Cmd::WRITE_LUT_REG, &TI_290MFGN_GRAY2_LUT_CODE)?;
        self.interface.wait_until_idle(delay);
        Ok(())
    }

    /// Power down the display controller into deep sleep mode.
    fn sleep(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        debug!("powering down ThinkInk 2.9\" grayscale display");
        self.interface.cmd_with_data(Cmd::DEEP_SLEEP, &[0x01])?;
        delay.delay_ms(1);
        Ok(())
    }

    /// Write `buffer` to the black/white RAM plane and wait until ready.
    fn update_bw(&mut self, buffer: &[u8], delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.write_bw_ram(buffer)?;
        self.interface.wait_until_idle(delay);
        Ok(())
    }

    /// Write `buffer` to the red RAM plane and wait until ready.
    fn update_red(&mut self, buffer: &[u8], delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.write_red_ram(buffer)?;
        self.interface.wait_until_idle(delay);
        Ok(())
    }

    /// Write both RAM planes and wait until ready.
    ///
    /// Does not trigger a display refresh; call [`Self::update_gray2_and_display`]
    /// for a complete write-and-refresh cycle.
    fn update(
        &mut self,
        bw_buffer: &[u8],
        red_buffer: &[u8],
        delay: &mut impl DelayNs,
    ) -> Result<(), DisplayError> {
        self.write_bw_ram(bw_buffer)?;
        self.write_red_ram(red_buffer)?;
        self.interface.wait_until_idle(delay);
        Ok(())
    }

    /// Fill the black/white RAM plane with white, clearing it.
    fn clear_bw_ram(&mut self) -> Result<(), DisplayError> {
        let color = Color::White.get_byte_value();
        self.interface.cmd(Cmd::WRITE_BW_DATA)?;
        self.interface
            .data_x_times(color, u32::from(WIDTH).div_ceil(8) * u32::from(HEIGHT))?;
        Ok(())
    }

    /// Fill the red RAM plane with its cleared state.
    fn clear_red_ram(&mut self) -> Result<(), DisplayError> {
        let color = Color::White.inverse().get_byte_value();
        self.interface.cmd(Cmd::WRITE_RED_DATA)?;
        self.interface
            .data_x_times(color, u32::from(WIDTH).div_ceil(8) * u32::from(HEIGHT))?;
        Ok(())
    }

    /// Hardware-reset the panel and put it into a low-power sleep state.
    ///
    /// Use this to prepare the display before the first [`Self::init`] call.
    fn begin(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.interface.hard_reset(delay)?;
        self.sleep(delay)
    }
}
