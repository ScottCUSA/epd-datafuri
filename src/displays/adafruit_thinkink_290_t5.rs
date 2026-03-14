//! Driver and graphics buffer for the Adafruit MagTag 2.9" e-ink display (IL0373 controller).
//!
//! This module targets the **original** Adafruit MagTag board revision which
//! uses the IL0373 e-paper controller. The 2025 edition uses an SSD1680
//! instead; see [`adafruit_thinkink_2in9_mfngr`](super::adafruit_thinkink_2in9_mfngr).
//!
//! ## Key differences from the SSD1680 variant
//!
//! | Property | SSD1680 | IL0373 |
//! |----------|---------|--------|
//! | Busy pin polarity | Active-high (HIGH = busy) | Active-low (LOW = busy) |
//! | LUT registers | Single register (0x32) | Five registers (0x20–0x24) |
//! | RAM address counters | Required | Not used |
//! | Gray2 encoding | Identical two-plane scheme | Identical two-plane scheme |
//!
//! Because the panel dimensions (296×128) and Gray2 encoding are identical,
//! - [`ThinkInk2in9Gray2`]: 2-bit, 4-level grayscale rendering using the Gray2 LUT
//! - [`ThinkInk2in9Mono`]: black/white rendering using the mono full LUT

use crate::color::Color;
use crate::driver::il0373::Il0373Cmd;
use crate::driver::EpdDriver;
use crate::interface::SpiDisplayInterface;
use display_interface::DisplayError;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::SpiDevice;
use log::debug;

pub use crate::graphics::display290_gray4_t5::Display2in9Gray2;
pub use crate::graphics::display290_mono::Display2in9Mono;

/// Display width of the MagTag 2.9" IL0373 panel in pixels.
pub const WIDTH: u16 = 296;
/// Display height of the MagTag 2.9" IL0373 panel in pixels.
pub const HEIGHT: u16 = 128;

// ---------------------------------------------------------------------------
// Gray4 waveform LUTs (5 separate tables, each 42 bytes).
// Ported from Adafruit's ThinkInk_290_Grayscale4_T5.h (ti_290t5_gray4_lut_code).
// ---------------------------------------------------------------------------

#[rustfmt::skip]
/// IL0373 Gray4 LUT1 (no-update waveform), register 0x20, 42 bytes.
///
/// Applied when a pixel does not need to change state. Keeping the pixel at
/// its current level without unnecessary voltage transitions reduces flicker.
const TI_290T5_GRAY4_LUT1: [u8; 42] = [
    0x00, 0x0A, 0x00, 0x00, 0x00, 0x01,
    0x60, 0x14, 0x14, 0x00, 0x00, 0x01,
    0x00, 0x14, 0x00, 0x00, 0x00, 0x01,
    0x00, 0x13, 0x0A, 0x01, 0x00, 0x01,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[rustfmt::skip]
/// IL0373 Gray4 LUTWW (white-to-white waveform), register 0x21, 42 bytes.
///
/// Applied when a pixel stays white across the update, ensuring the panel
/// maintains proper white-level driving voltage.
const TI_290T5_GRAY4_LUTWW: [u8; 42] = [
    0x40, 0x0A, 0x00, 0x00, 0x00, 0x01,
    0x90, 0x14, 0x14, 0x00, 0x00, 0x01,
    0x10, 0x14, 0x0A, 0x00, 0x00, 0x01,
    0xA0, 0x13, 0x01, 0x00, 0x00, 0x01,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[rustfmt::skip]
/// IL0373 Gray4 LUTBW (black-to-white waveform), register 0x22, 42 bytes.
///
/// Applied when a pixel transitions from black to white, driving the pixel
/// through the correct voltage sequence to fully clear the e-ink capsules.
const TI_290T5_GRAY4_LUTBW: [u8; 42] = [
    0x40, 0x0A, 0x00, 0x00, 0x00, 0x01,
    0x90, 0x14, 0x14, 0x00, 0x00, 0x01,
    0x00, 0x14, 0x0A, 0x00, 0x00, 0x01,
    0x99, 0x0C, 0x01, 0x03, 0x04, 0x01,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[rustfmt::skip]
/// IL0373 Gray4 LUTWB (white-to-black waveform), register 0x23, 42 bytes.
///
/// Applied when a pixel transitions from white to black, driving the e-ink
/// capsules to their fully actuated (dark) state.
const TI_290T5_GRAY4_LUTWB: [u8; 42] = [
    0x40, 0x0A, 0x00, 0x00, 0x00, 0x01,
    0x90, 0x14, 0x14, 0x00, 0x00, 0x01,
    0x00, 0x14, 0x0A, 0x00, 0x00, 0x01,
    0x99, 0x0B, 0x04, 0x04, 0x01, 0x01,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[rustfmt::skip]
/// IL0373 Gray4 LUTBB (black-to-black waveform), register 0x24, 42 bytes.
///
/// Applied when a pixel stays black across the update, maintaining the dark
/// state without unnecessary voltage transitions.
const TI_290T5_GRAY4_LUTBB: [u8; 42] = [
    0x80, 0x0A, 0x00, 0x00, 0x00, 0x01,
    0x90, 0x14, 0x14, 0x00, 0x00, 0x01,
    0x20, 0x14, 0x0A, 0x00, 0x00, 0x01,
    0x50, 0x13, 0x01, 0x00, 0x00, 0x01,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Driver for the Adafruit MagTag 2.9" monochrome e-ink display (IL0373 controller).
///
/// The IL0373 busy pin is active-low (LOW = busy, HIGH = ready).
///
/// Full refreshes use the factory OTP waveform (no custom LUT is loaded).
/// Use [`Display2in9Mono`] as the graphics buffer.
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
    /// Create a new MagTag 2.9" IL0373 monochrome display driver.
    pub fn new(spi: SPI, busy: BSY, dc: DC, rst: RST) -> Result<Self, DisplayError> {
        let interface = SpiDisplayInterface::new(spi, busy, dc, rst);
        Ok(Self { interface })
    }

    fn write_dtm1(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.interface.cmd_with_data(Il0373Cmd::DTM1, buffer)
    }

    fn display(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.interface.cmd(Il0373Cmd::DISPLAY_REFRESH)?;
        delay.delay_ms(100);
        self.interface.wait_until_idle_active_low(delay);
        Ok(())
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
}

impl<SPI, BSY, DC, RST> EpdDriver for ThinkInk2in9Mono<SPI, BSY, DC, RST>
where
    SPI: SpiDevice,
    BSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Initialize for full monochrome operation using the factory OTP waveform.
    ///
    /// PANEL_SETTING `0x1f` selects OTP waveform (bit 5 = 0), so no custom LUT
    /// is loaded. Matches Adafruit's `ti_290t5_monofull_init_code`.
    fn init(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        debug!("powering up MagTag 2.9\" IL0373 mono display");

        self.interface.hard_reset(delay)?;

        self.interface
            .cmd_with_data(Il0373Cmd::BOOSTER_SOFT_START, &[0x17, 0x17, 0x17])?;

        self.interface.cmd(Il0373Cmd::POWER_ON)?;
        self.interface.wait_until_idle_active_low(delay);
        delay.delay_ms(200);

        // 0x1f: OTP waveform (bit 5 = 0), scan up, shift right, booster on
        // 0x0d: additional panel configuration
        self.interface
            .cmd_with_data(Il0373Cmd::PANEL_SETTING, &[0x1f, 0x0d])?;

        self.interface.cmd_with_data(Il0373Cmd::CDI, &[0x97])?;

        Ok(())
    }

    /// Power down the display controller.
    fn sleep(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        debug!("powering down MagTag 2.9\" IL0373 mono display");
        self.interface.cmd_with_data(Il0373Cmd::CDI, &[0x17])?;
        self.interface.cmd(Il0373Cmd::VCM_DC_SETTING)?;
        self.interface.cmd(Il0373Cmd::POWER_OFF)?;
        delay.delay_ms(1);
        Ok(())
    }

    /// Write `buffer` to the black/white RAM plane (DTM1) and wait until ready.
    fn update_bw(&mut self, buffer: &[u8], delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.write_dtm1(buffer)?;
        self.interface.wait_until_idle_active_low(delay);
        Ok(())
    }

    /// DTM2 is not used in mono OTP mode; this is a no-op.
    fn update_red(&mut self, _buffer: &[u8], _delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        Ok(())
    }

    /// Write the BW plane and wait until ready. DTM2 is ignored in mono OTP mode.
    fn update(
        &mut self,
        bw_buffer: &[u8],
        _red_buffer: &[u8],
        delay: &mut impl DelayNs,
    ) -> Result<(), DisplayError> {
        self.write_dtm1(bw_buffer)?;
        self.interface.wait_until_idle_active_low(delay);
        Ok(())
    }

    /// Fill the black/white RAM plane (DTM1) with white, clearing it.
    fn clear_bw_ram(&mut self) -> Result<(), DisplayError> {
        let color = Color::White.get_byte_value();
        self.interface.cmd(Il0373Cmd::DTM1)?;
        self.interface
            .data_x_times(color, u32::from(HEIGHT).div_ceil(8) * u32::from(WIDTH))?;
        Ok(())
    }

    /// DTM2 is not used in mono OTP mode; this is a no-op.
    fn clear_red_ram(&mut self) -> Result<(), DisplayError> {
        Ok(())
    }

    /// Hardware-reset the panel and put it into a low-power sleep state.
    fn begin(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.interface.hard_reset(delay)?;
        self.sleep(delay)
    }
}

/// Driver for the Adafruit MagTag 2.9" 4-level grayscale e-ink display (IL0373 controller).
///
/// The IL0373 busy pin is active-low (LOW = busy, HIGH = ready), which is the
/// opposite of the SSD1680. This struct handles the polarity difference internally.
///
/// Use [`Display2in9Gray2`] as the graphics buffer; it is identical to the
/// SSD1680 variant since both panels share 296×128 dimensions and the same
/// Gray2 bit encoding across the two framebuffer planes.
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
    /// Create a new MagTag 2.9" IL0373 grayscale display driver.
    pub fn new(spi: SPI, busy: BSY, dc: DC, rst: RST) -> Result<Self, DisplayError> {
        let interface = SpiDisplayInterface::new(spi, busy, dc, rst);
        Ok(Self { interface })
    }

    /// Write `buffer` to the black/white RAM plane (DTM1).
    fn write_dtm1(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.interface.cmd_with_data(Il0373Cmd::DTM1, buffer)
    }

    /// Write `buffer` to the color RAM plane (DTM2).
    fn write_dtm2(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.interface.cmd_with_data(Il0373Cmd::DTM2, buffer)
    }

    /// Trigger a full display refresh and wait for the panel to become ready.
    ///
    /// Issues `DISPLAY_REFRESH`, waits 100 ms for the panel to start updating,
    /// then polls the busy pin (active-low) until the refresh completes.
    fn display(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.interface.cmd(Il0373Cmd::DISPLAY_REFRESH)?;
        delay.delay_ms(100);
        self.interface.wait_until_idle_active_low(delay);
        Ok(())
    }

    /// Write both Gray2 framebuffer planes to the display and trigger a full refresh.
    ///
    /// `high_buffer` is written to DTM1 (black/white plane) and `low_buffer` to
    /// DTM2 (color plane). Pass [`Display2in9Gray2::high_buffer`] and
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
    /// Performs a hardware reset, sends the Gray4 power and panel configuration,
    /// loads all five waveform LUT tables, and sets the panel resolution.
    /// The busy pin (active-low) is polled after power-on before proceeding.
    fn init(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        debug!("powering up MagTag 2.9\" IL0373 grayscale display");

        // Hardware reset
        self.interface.hard_reset(delay)?;

        // Power setting: external power, VGH=20V, VGL=-20V, VDH=15V, VDL=-15V
        self.interface
            .cmd_with_data(Il0373Cmd::POWER_SETTING, &[0x03, 0x00, 0x2b, 0x2b, 0x13])?;

        // Booster soft start: phase A/B/C all 0x17
        self.interface
            .cmd_with_data(Il0373Cmd::BOOSTER_SOFT_START, &[0x17, 0x17, 0x17])?;

        // Power on, then wait for the display to be ready
        self.interface.cmd(Il0373Cmd::POWER_ON)?;
        self.interface.wait_until_idle_active_low(delay);
        delay.delay_ms(200);

        // Panel setting: KW/R mode, scan up, shift right, booster on
        self.interface
            .cmd_with_data(Il0373Cmd::PANEL_SETTING, &[0x3F])?;

        // PLL: 50Hz frame rate
        self.interface.cmd_with_data(Il0373Cmd::PLL, &[0x3C])?;

        // VCM DC setting
        self.interface
            .cmd_with_data(Il0373Cmd::VCM_DC_SETTING, &[0x12])?;

        // CDI: border/data polarity
        self.interface.cmd_with_data(Il0373Cmd::CDI, &[0x97])?;

        // Load the 5-part Gray4 LUT
        self.interface
            .cmd_with_data(Il0373Cmd::LUT1, &TI_290T5_GRAY4_LUT1)?;
        self.interface
            .cmd_with_data(Il0373Cmd::LUTWW, &TI_290T5_GRAY4_LUTWW)?;
        self.interface
            .cmd_with_data(Il0373Cmd::LUTBW, &TI_290T5_GRAY4_LUTBW)?;
        self.interface
            .cmd_with_data(Il0373Cmd::LUTWB, &TI_290T5_GRAY4_LUTWB)?;
        self.interface
            .cmd_with_data(Il0373Cmd::LUTBB, &TI_290T5_GRAY4_LUTBB)?;

        // IL0373 RESOLUTION format: [gate_lines, source_lines_hi, source_lines_lo]
        // Gate lines = 128 (our WIDTH), source lines = 296 (our HEIGHT).
        // Note: our WIDTH/HEIGHT naming is inverted relative to the Arduino library,
        // which uses WIDTH=296 and HEIGHT=128 for this panel.
        self.interface.cmd_with_data(
            Il0373Cmd::RESOLUTION,
            &[
                (HEIGHT & 0xFF) as u8,       // gate lines:        128 = 0x80
                ((WIDTH >> 8) & 0xFF) as u8, // source lines high:   1 = 0x01
                (HEIGHT & 0xFF) as u8,       // source lines low:   40 = 0x28
            ],
        )?;

        Ok(())
    }

    /// Power down the display controller.
    ///
    /// Sets CDI to the border-floating state, discharges the VCM DC voltage,
    /// then issues the power-off command.
    fn sleep(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        debug!("powering down MagTag 2.9\" IL0373 grayscale display");

        // CDI: border floating
        self.interface.cmd_with_data(Il0373Cmd::CDI, &[0x17])?;

        // VCM DC: discharge
        self.interface.cmd(Il0373Cmd::VCM_DC_SETTING)?;

        // Power off
        self.interface.cmd(Il0373Cmd::POWER_OFF)?;
        delay.delay_ms(1);
        Ok(())
    }

    /// Write `buffer` to the black/white RAM plane (DTM1) and wait until ready.
    fn update_bw(&mut self, buffer: &[u8], delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.write_dtm1(buffer)?;
        self.interface.wait_until_idle_active_low(delay);
        Ok(())
    }

    /// Write `buffer` to the color RAM plane (DTM2) and wait until ready.
    fn update_red(&mut self, buffer: &[u8], delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.write_dtm2(buffer)?;
        self.interface.wait_until_idle_active_low(delay);
        Ok(())
    }

    /// Write both RAM planes and wait until ready.
    ///
    /// `bw_buffer` goes to DTM1 and `red_buffer` goes to DTM2. Does not trigger
    /// a display refresh; call [`Self::update_gray2_and_display`] for a
    /// complete write-and-refresh cycle.
    fn update(
        &mut self,
        bw_buffer: &[u8],
        red_buffer: &[u8],
        delay: &mut impl DelayNs,
    ) -> Result<(), DisplayError> {
        self.write_dtm1(bw_buffer)?;
        self.write_dtm2(red_buffer)?;
        self.interface.wait_until_idle_active_low(delay);
        Ok(())
    }

    /// Fill the black/white RAM plane (DTM1) with white, clearing it.
    fn clear_bw_ram(&mut self) -> Result<(), DisplayError> {
        let color = Color::White.get_byte_value();
        self.interface.cmd(Il0373Cmd::DTM1)?;
        self.interface
            .data_x_times(color, u32::from(HEIGHT).div_ceil(8) * u32::from(WIDTH))?;
        Ok(())
    }

    /// Fill the color RAM plane (DTM2) with its cleared state.
    fn clear_red_ram(&mut self) -> Result<(), DisplayError> {
        let color = Color::White.inverse().get_byte_value();
        self.interface.cmd(Il0373Cmd::DTM2)?;
        self.interface
            .data_x_times(color, u32::from(HEIGHT).div_ceil(8) * u32::from(WIDTH))?;
        Ok(())
    }

    /// Hardware-reset the panel and put it into a low-power sleep state.
    ///
    /// Equivalent to calling [`Self::sleep`] after a hard reset. Use this to
    /// prepare the display before the first [`Self::init`] call.
    fn begin(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.interface.hard_reset(delay)?;
        self.sleep(delay)
    }
}
