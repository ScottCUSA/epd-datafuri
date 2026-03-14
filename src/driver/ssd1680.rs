//! SSD1680 command opcodes.
//! SSD1680 configuration flag values used alongside command bytes.

/// SSD1680 command byte constants.
pub struct Cmd;

#[allow(dead_code)]
impl Cmd {
    /// Set the number of gate lines and scanning sequence (MUX ratio / output control).
    pub const DRIVER_OUTPUT_CTRL: u8 = 0x01;
    /// Set gate driving voltage (VGH / VGL levels).
    pub const GATE_VOLTAGE_CTRL: u8 = 0x03;
    /// Set source driving voltage (VSH1, VSH2, VSL).
    pub const SOURCE_VOLTAGE_CTRL: u8 = 0x04;
    /// Enter deep sleep mode. Send 0x01 to keep RAM, 0x11 to discard.
    pub const DEEP_SLEEP: u8 = 0x10;
    /// Set the RAM data entry mode (scan direction for X and Y address counters).
    pub const DATA_ENTRY_MODE: u8 = 0x11;
    /// Trigger a software reset; all registers revert to defaults.
    pub const SW_RESET: u8 = 0x12;
    /// Select the temperature sensor source (internal or external).
    pub const TEMP_CONTROL: u8 = 0x18;
    /// Activate the display update sequence defined by DISPLAY_UPDATE_CTRL2.
    pub const MASTER_ACTIVATE: u8 = 0x20;
    /// Configure which internal blocks run during the display update sequence.
    pub const DISPLAY_UPDATE_CTRL1: u8 = 0x21;
    /// Set the display update sequence option byte used by MASTER_ACTIVATE.
    pub const DISPLAY_UPDATE_CTRL2: u8 = 0x22;
    /// Write data into the black/white RAM plane.
    pub const WRITE_BW_DATA: u8 = 0x24;
    /// Write data into the red/accent RAM plane.
    pub const WRITE_RED_DATA: u8 = 0x26;
    /// Set the VCOM voltage register.
    pub const WRITE_VCOM_REG: u8 = 0x2C;
    /// Load a custom waveform look-up table (LUT) into the controller.
    pub const WRITE_LUT_REG: u8 = 0x32;
    /// Configure the border waveform for pixels outside the active area.
    pub const BORDER_WAVEFORM_CTRL: u8 = 0x3C;
    /// Set the source output behaviour at the end of a display update.
    pub const END_OPTION: u8 = 0x3F;
    /// Set the start and end X (column) address of the RAM window.
    pub const SET_RAMXPOS: u8 = 0x44;
    /// Set the start and end Y (row) address of the RAM window.
    pub const SET_RAMYPOS: u8 = 0x45;
    /// Set the current X address counter (column pointer) for RAM writes.
    pub const SET_RAMX_COUNTER: u8 = 0x4E;
    /// Set the current Y address counter (row pointer) for RAM writes.
    pub const SET_RAMY_COUNTER: u8 = 0x4F;
}

/// SSD1680 flag/option byte constants.
pub struct Flag;

#[allow(dead_code)]
impl Flag {
    /// Data entry mode: increment Y, decrement X (right-to-left, top-to-bottom).
    pub const DATA_ENTRY_INCRY_DECRX: u8 = 0x01;
    /// Data entry mode: increment Y, increment X (left-to-right, top-to-bottom).
    pub const DATA_ENTRY_INCRY_INCRX: u8 = 0x03;
    /// Use the on-chip temperature sensor for waveform selection.
    pub const INTERNAL_TEMP_SENSOR: u8 = 0x80;
    /// Border waveform: follow the currently loaded LUT.
    pub const BORDER_WAVEFORM_FOLLOW_LUT: u8 = 0x04;
    /// Border waveform LUT selection: LUT0 (black).
    pub const BORDER_WAVEFORM_LUT0: u8 = 0x00;
    /// Border waveform LUT selection: LUT1 (white).
    pub const BORDER_WAVEFORM_LUT1: u8 = 0x01;
    /// Display update sequence: clock enable → analog on → load LUT → initial display → pattern display.
    pub const DISPLAY_MODE_1: u8 = 0xC7;
    /// Display update sequence: clock enable → analog on → load temperature → load LUT → pattern display.
    pub const DISPLAY_MODE_LOAD_TEMP_1: u8 = 0xF7;
    /// Source output at end of update: normal (VSS).
    pub const END_OPTION_NORMAL: u8 = 0x22;
    /// Source output at end of update: keep previous level (floating).
    pub const END_OPTION_KEEP: u8 = 0x07;
}
