// Use and re-export.
pub use embedded_graphics::pixelcolor::{Rgb565, RgbColor};

use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::{
    AnyInputPin,
    AnyOutputPin,
    Output,
    PinDriver,
};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::spi::{
    SpiAnyPins,
    SpiDeviceDriver,
    SpiDriverConfig,
    SpiDriver,
    SpiConfig,
};
use esp_idf_svc::hal::units::Hertz;

use display_interface_spi::SPIInterface;

use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Baseline, Text},
};

use ili9341::{DisplaySize240x320, Ili9341, Orientation};

#[cfg(feature = "framebuffer")]
use ili9341::DisplayError;

#[cfg(feature = "framebuffer")]
use crate::framebuffer::{DrawRawSlice, Framebuffer};

#[cfg(feature = "framebuffer")]
type DisplayResult<T = (), E = DisplayError> = core::result::Result<T, E>;

type TftSpiInterface<'spi> = SPIInterface<
    SpiDeviceDriver<'spi, SpiDriver<'spi>>,
    PinDriver<'spi, AnyOutputPin, Output>,
>;

type MchIli9341<'spi> = Ili9341<
    TftSpiInterface<'spi>,
    PinDriver<'spi, AnyOutputPin, Output>,
>;

#[cfg(feature = "framebuffer")]
type MchFramebuffer = Framebuffer<320, 240>;

pub struct Display<'spi> {
    display: MchIli9341<'spi>,
    #[cfg(feature = "framebuffer")]
    framebuffer: MchFramebuffer,
}


#[cfg(feature = "framebuffer")]
impl<'spi> DrawRawSlice for MchIli9341<'spi> {
    fn draw_raw_slice(&mut self, x0: u16, y0: u16, x1: u16, y1: u16, data: &[u16]) -> DisplayResult {
        Ili9341::draw_raw_slice(self, x0, y0, x1, y1, data)
    }
}



impl<'spi> Display<'spi> {
    pub fn new<SPI: SpiAnyPins>(
        spi: impl Peripheral<P = SPI> + 'spi,
        sclk: AnyOutputPin, // Gpio18; Clock signal, driven by master
        mosi: AnyOutputPin, // Gpio23; Master Out Slave In, driven by master
        cs: AnyOutputPin,   // Gpio32; Chip select, driven by master
        rst: AnyOutputPin,  // Gpio25; Reset, hold low to reset the ili9341
        dc: AnyOutputPin,   // Gpio33; Data/Command selection, driven by master
    ) -> Display<'spi> {
        log::info!("Starting mchdisplay::Display");

        let spi_device = SpiDeviceDriver::new_single(
            spi,
            sclk,
            mosi, // sdo/MOSI
            Option::<AnyInputPin>::None, // sdi/MISO, unused
            Some(cs),
            &SpiDriverConfig::new(),
            &Self::create_config(),
        ).unwrap();

        let dc_output = PinDriver::output(dc).unwrap();
        let interface = SPIInterface::new(spi_device, dc_output);

        let rst_output = PinDriver::output(rst).unwrap();
        let display = Ili9341::new(
            interface,
            rst_output,
            &mut Ets,
            Orientation::Landscape,
            DisplaySize240x320,
        ).unwrap();

        Display {
            display,
            #[cfg(feature = "framebuffer")]
            // TODO: Decide whether to keep this beast. It's very memory
            // expensive (150KiB).  But it makes drawing on the screen a
            // lot nicer. (No manual clearing.) Note that esp-hal raw
            // SPI stuff was blazing fast, so if we want back to pure
            // ESP-HAL without ESP-IDF, we could do without.
            framebuffer: MchFramebuffer::new(),
        }
    }

    #[cfg(not(feature = "framebuffer"))]
    fn virtual_display(&mut self) -> &mut MchIli9341<'spi> {
        &mut self.display
    }
    #[cfg(feature = "framebuffer")]
    fn virtual_display(&mut self) -> &mut MchFramebuffer {
        &mut self.framebuffer
    }

    fn create_config() -> SpiConfig {
        SpiConfig::default()
            .baudrate(Hertz(40_000_000))
            .write_only(true)
    }

    pub fn clear(&mut self, color: Rgb565) {
        self.virtual_display().clear(color).unwrap();
    }

    pub fn part_clear(&mut self, color: Rgb565, x: i32, y: i32, w: u32, h: u32) {
        Rectangle::new(Point::new(x, y), Size::new(w, h))
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(self.virtual_display())
            .unwrap();
    }

    pub fn println(&mut self, text: &str, x: i32, y: i32) {
        let style = MonoTextStyle::new(&FONT_8X13, Rgb565::RED);
        //Text::with_alignment(text, Point::new(x, y), style, Alignment::Center)
        //    .draw(&mut self.framebuffer)
        //    .unwrap();
        Text::with_baseline(text, Point::new(x, y), style, Baseline::Top)
            .draw(self.virtual_display())
            .unwrap();
    }

    pub fn flush(&mut self) {
        #[cfg(feature = "framebuffer")]
        self.framebuffer.flush(&mut self.display).unwrap();
    }
}
