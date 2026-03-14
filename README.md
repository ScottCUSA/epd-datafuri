# epd-datafuri

Rust driver for Adafruit e-Paper display (EPD) controllers, for use with [embedded-hal].

Supported controllers:
- **SSD1680** (Solomon Systech): [datasheet][SSD1680]
- **IL0373** (Good Display): original Adafruit MagTag 2.9"

[![crates.io](https://img.shields.io/crates/v/epd-datafuri.svg)](https://crates.io/crates/epd-datafuri)
[![Documentation](https://docs.rs/epd-datafuri/badge.svg)](https://docs.rs/epd-datafuri/)


## Supported Displays

| Display | Controller | Colors | Grayscale |
|---------|-----------|--------|-----------|
| Adafruit ThinkInk 2.9" (2025 revision) | SSD1680 | BW | Gray2 (4-level) |
| Adafruit MagTag 2.9" (original revision) | IL0373 | BW | Gray2 (4-level) |

## Description

Built using [embedded-hal] and optionally [embedded-graphics].

### Grayscale

4-level grayscale (Gray2) is supported for both displays. Each display module
exports its own `Display2in9Gray2` with the correct plane mapping for that
controller:

- `adafruit_thinkink_290_mfgn::Display2in9Gray2` for the ThinkInk 2.9" (SSD1680)
- `adafruit_thinkink_290_t5::Display2in9Gray2` for the MagTag 2.9" (IL0373)

### Partial Updates

Partial updates are not currently supported.

## Usage

### ThinkInk 2.9" — Monochrome (SSD1680)

```rust,ignore
use epd_datafuri::displays::adafruit_thinkink_290_mfgn::{ThinkInk2in9Mono, Display2in9Mono};
use epd_datafuri::prelude::*;

let mut driver = ThinkInk2in9Mono::new(spi, busy, dc, rst)?;
let mut display = Display2in9Mono::new();

// Draw using embedded-graphics
Text::new("Hello!", Point::new(10, 20), text_style).draw(&mut display)?;

driver.begin(&mut delay)?;
driver.update_and_display(display.buffer(), &mut delay)?;
```

### ThinkInk 2.9" — Grayscale (SSD1680)

```rust,ignore
use epd_datafuri::displays::adafruit_thinkink_290_mfgn::{ThinkInk2in9Gray2, Display2in9Gray2};
use epd_datafuri::prelude::*;
use embedded_graphics::pixelcolor::Gray2;

let mut driver = ThinkInk2in9Gray2::new(spi, busy, dc, rst)?;
let mut display = Display2in9Gray2::new();

// Draw using embedded-graphics with 4-level gray
Text::new("Hello!", Point::new(10, 20), text_style).draw(&mut display)?;
Rectangle::new(Point::new(50, 50), Size::new(40, 40))
    .into_styled(PrimitiveStyle::with_fill(Gray2::new(1)))
    .draw(&mut display)?;

driver.begin(&mut delay)?;
driver.update_gray2_and_display(display.high_buffer(), display.low_buffer(), &mut delay)?;
```

### MagTag 2.9" — Grayscale (IL0373)

```rust,ignore
use epd_datafuri::displays::adafruit_thinkink_290_t5::{ThinkInk2in9Gray2, Display2in9Gray2};
use epd_datafuri::prelude::*;
use embedded_graphics::pixelcolor::Gray2;

let mut driver = ThinkInk2in9Gray2::new(spi, busy, dc, rst)?;
let mut display = Display2in9Gray2::new();

// Draw using embedded-graphics with 4-level gray
Text::new("Hello!", Point::new(10, 20), text_style).draw(&mut display)?;
Rectangle::new(Point::new(50, 50), Size::new(40, 40))
    .into_styled(PrimitiveStyle::with_fill(Gray2::new(1)))
    .draw(&mut display)?;

driver.begin(&mut delay)?;
driver.update_gray2_and_display(display.high_buffer(), display.low_buffer(), &mut delay)?;
```

## Examples

- [ESP-HAL MagTag examples](examples/)
  - `adafruit_magtag_bw`: black/white rendering (SSD1680)
  - `adafruit_magtag_gray2`: 4-level grayscale rendering (SSD1680)
  - `adafruit_magtag_legacy_gray2`: 4-level grayscale rendering (IL0373)
- [ESP32-S2 example project](https://github.com/ScottCUSA/magtag_esp_hal)

## Credits

This crate is a fork of [ssd1680](https://crates.io/crates/ssd1680) by
[Konstantin Terekhov](https://github.com/mbv), extended to support additional
Adafruit EPD controllers.

* [Arduino Display Library for SPI E-Paper Displays](https://github.com/ZinggJM/GxEPD2)
* [Adafruit ThinkInk Arduino library](https://github.com/adafruit/Adafruit_EPD)
* [SSD1681 EPD driver](https://github.com/afajl/ssd1681)
* [Waveshare EPD driver](https://github.com/caemor/epd-waveshare)

## License

`epd-datafuri` is dual licenced under:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) **or**
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

[embedded-hal]: https://crates.io/crates/embedded-hal
[embedded-graphics]: https://github.com/embedded-graphics/embedded-graphics
[SSD1680]: https://www.solomon-systech.com/product/ssd1680
