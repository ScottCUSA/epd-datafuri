//! IL0373 controller commands
//! IL0373 command opcodes.

/// IL0373 command byte constants.
pub struct Il0373Cmd;

#[allow(dead_code)]
impl Il0373Cmd {
    /// Configure panel resolution, LUT selection, and gate scan direction.
    pub const PANEL_SETTING: u8 = 0x00;
    /// Set power supply voltages (VGH, VGL, VDH, VDL).
    pub const POWER_SETTING: u8 = 0x01;
    /// Turn off the internal power supply.
    pub const POWER_OFF: u8 = 0x02;
    /// Configure the power-off sequence timing.
    pub const POWER_OFF_SEQUENCE: u8 = 0x03;
    /// Turn on the internal power supply.
    pub const POWER_ON: u8 = 0x04;
    /// Measure the internal power supply voltage after power-on.
    pub const POWER_ON_MEASURE: u8 = 0x05;
    /// Configure the charge pump booster soft-start phases A, B, and C.
    pub const BOOSTER_SOFT_START: u8 = 0x06;
    /// Enter deep sleep mode (send 0xA5 to confirm).
    pub const DEEP_SLEEP: u8 = 0x07;
    /// Display start transmission 1: write black/white pixel data into RAM.
    pub const DTM1: u8 = 0x10;
    /// Signal the end of a data transmission sequence.
    pub const DATA_STOP: u8 = 0x11;
    /// Trigger a full display refresh from the current RAM contents.
    pub const DISPLAY_REFRESH: u8 = 0x12;
    /// Display start transmission 2: write color/red pixel data into RAM.
    pub const DTM2: u8 = 0x13;
    /// Load LUT1: no-update waveform (pixel state unchanged).
    pub const LUT1: u8 = 0x20;
    /// Load LUTWW: white-to-white transition waveform.
    pub const LUTWW: u8 = 0x21;
    /// Load LUTBW: black-to-white transition waveform.
    pub const LUTBW: u8 = 0x22;
    /// Load LUTWB: white-to-black transition waveform.
    pub const LUTWB: u8 = 0x23;
    /// Load LUTBB: black-to-black transition waveform.
    pub const LUTBB: u8 = 0x24;
    /// Set the PLL clock frequency (controls frame rate).
    pub const PLL: u8 = 0x30;
    /// Set CDI (common driving interval) and border/data polarity.
    pub const CDI: u8 = 0x50;
    /// Set the panel resolution (height and width in pixels).
    pub const RESOLUTION: u8 = 0x61;
    /// Set the VCOM DC voltage level.
    pub const VCM_DC_SETTING: u8 = 0x82;
    /// Define the partial update window boundaries.
    pub const PARTIAL_WINDOW: u8 = 0x90;
    /// Enter partial update mode.
    pub const PARTIAL_ENTER: u8 = 0x91;
    /// Exit partial update mode.
    pub const PARTIAL_EXIT: u8 = 0x92;
}
