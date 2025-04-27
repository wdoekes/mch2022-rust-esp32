// Use and re-export.
pub use embedded_graphics::pixelcolor::{Rgb565, RgbColor};

// Pull in alloc even though we're using no_std.
extern crate alloc;
use alloc::vec::Vec;

use esp_hal::Blocking;
use esp_hal::delay::Delay;
use esp_hal::gpio::{GpioPin, Level, Output, OutputConfig};
use esp_hal::peripherals::SPI3;
use esp_hal::spi::master::{Config, Spi};
use esp_hal::time::Rate;

use esp_println::println;

use display_interface_spi::SPIInterface;

use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Baseline, Text},
};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};

use ili9341::{DisplaySize240x320, Ili9341, Orientation};


const ILI9341_POWERA: u8 = 0xCB; // Power control A register
const ILI9341_POWERB: u8 = 0xCF; // Power control B register
const ILI9341_DTCA: u8 = 0xE8; // Driver timing control A
const ILI9341_DTCB: u8 = 0xEA; // Driver timing control B
const ILI9341_POWER_SEQ: u8 = 0xED; // Power on sequence register
const ILI9341_3GAMMA_EN: u8 = 0xF2; // 3 Gamma enable register
const ILI9341_PRC: u8 = 0xF7; // Pump ratio control register
const ILI9341_LCMCTRL: u8 = 0xC0; // LCM Control
const ILI9341_POWER2: u8 = 0xC1; // Power Control 2 register
const ILI9341_VCOM1: u8 = 0xC5; // VCOM Control 1 register
const ILI9341_VCOM2: u8 = 0xC7; // VCOM Control 2 register
const ILI9341_MADCTL: u8 = 0x36; // Memory Data Access Control
const ILI9341_COLMOD: u8 = 0x3A; // Interface Pixel Format
const ILI9341_FRMCTR1: u8 = 0xB1; // Frame Rate Control (In Normal Mode)
const ILI9341_DFC: u8 = 0xB6; // Display Function Control register
const ILI9341_PVGAMCTRL: u8 = 0xE0; // Positive Voltage Gamma control
const ILI9341_NVGAMCTRL: u8 = 0xE1; // Negative Voltage Gamma control
const ILI9341_GAMSET: u8 = 0x26; // Display Invert On Gamma


struct Ili9341Command {
    cmd: u8,
    data: Vec<u8>,
}

type TFTSpiInterface<'spi> = SPIInterface<
    ExclusiveDevice<Spi<'spi, Blocking>, Output<'spi>, NoDelay>,
    Output<'spi>>;

pub struct Display<'spi> {
    display: Ili9341<TFTSpiInterface<'spi>, Output<'spi>>,
}


impl<'spi> Display<'spi> {
    pub fn new(
        spi3: SPI3,
        sclk: GpioPin<18>,
        miso: GpioPin<21>, // sdi
        mosi: GpioPin<23>, // sdo
        cs: GpioPin<32>, // or also 21..
        rst: GpioPin<25>,
        dc: GpioPin<33>,
    ) -> Display<'spi> {
        println!("Starting mchdisplay::Display");

        let config = OutputConfig::default();
        let rst_output = Output::new(rst, Level::Low, config);
        let dc_output = Output::new(dc, Level::Low, config);

        let spi = Spi::new(spi3, Self::create_config())
            .unwrap()
            .with_sck(sclk)
            .with_miso(miso) // order matters
            .with_mosi(mosi) // order matters
            // with cable-select 21 ??
            ;

        let cs_output = Output::new(cs, Level::High, config); // cs == miso??
        let spi_device = ExclusiveDevice::new_no_delay(spi, cs_output).unwrap();
        let mut interface = SPIInterface::new(spi_device, dc_output);

        let init_sequence = [
            Ili9341Command {
                cmd: ILI9341_POWERB,
                data: Vec::from([0x00, 0xC1, 0x30]),
            },
            Ili9341Command {
                cmd: ILI9341_POWER_SEQ,
                data: Vec::from([0x64, 0x03, 0x12, 0x81]),
            },
            Ili9341Command {
                cmd: ILI9341_DTCA,
                data: Vec::from([0x85, 0x00, 0x78]),
            },
            Ili9341Command {
                cmd: ILI9341_POWERA,
                data: Vec::from([0x39, 0x2C, 0x00, 0x34, 0x02]),
            },
            Ili9341Command {
                cmd: ILI9341_PRC,
                data: Vec::from([0x20]),
            },
            Ili9341Command {
                cmd: ILI9341_DTCB,
                data: Vec::from([0x00, 0x00]),
            },
            Ili9341Command {
                cmd: ILI9341_LCMCTRL,
                data: Vec::from([0x23]),
            },
            Ili9341Command {
                cmd: ILI9341_POWER2,
                data: Vec::from([0x10]),
            },
            Ili9341Command {
                cmd: ILI9341_VCOM1,
                data: Vec::from([0x3e, 0x28]),
            },
            Ili9341Command {
                cmd: ILI9341_VCOM2,
                data: Vec::from([0x86]),
            },
            Ili9341Command {
                cmd: ILI9341_MADCTL,
                data: Vec::from([0x48]),
            },
            Ili9341Command {
                cmd: ILI9341_COLMOD,
                data: Vec::from([0x55]),
            },
            Ili9341Command {
                cmd: ILI9341_FRMCTR1,
                data: Vec::from([0x00, 0x18]),
            },
            Ili9341Command {
                cmd: ILI9341_DFC,
                data: Vec::from([0x08, 0x82, 0x27]),
            },
            Ili9341Command {
                cmd: ILI9341_3GAMMA_EN,
                data: Vec::from([0x00]),
            },
            Ili9341Command {
                cmd: ILI9341_GAMSET,
                data: Vec::from([0x01]),
            },
            Ili9341Command {
                cmd: ILI9341_PVGAMCTRL,
                data: Vec::from([
                    0x0F, 0x31, 0x2B, 0x0C, 0x0E, 0x08, 0x4E, 0xF1,
                    0x37, 0x07, 0x10, 0x03, 0x0E, 0x09, 0x00,
                ]),
            },
            Ili9341Command {
                cmd: ILI9341_NVGAMCTRL,
                data: Vec::from([
                    0x00, 0x0E, 0x14, 0x03, 0x11, 0x07, 0x31, 0xC1,
                    0x48, 0x08, 0x0F, 0x0C, 0x31, 0x36, 0x0F,
                ]),
            },
        ];

        for cmd in init_sequence {
            use display_interface::DataFormat;
            use display_interface::WriteOnlyDataCommand;

            interface.send_commands(DataFormat::U8(&[cmd.cmd])).unwrap();
            interface.send_data(DataFormat::U8(&cmd.data)).unwrap();
        }

        let display = Ili9341::new(
            interface,
            rst_output,
            &mut Delay::new(),
            Orientation::Landscape,
            DisplaySize240x320,
        ).unwrap();

        Display { display }
    }

    fn create_config() -> Config {
        Config::default().with_frequency(Rate::from_mhz(40))
    }

    pub fn clear(&mut self, color: Rgb565) {
        self.display.clear(color).unwrap();
    }

    //pub fn draw_on_display(&mut self, draw_func: impl Fn(f64) -> None)

    pub fn part_clear(&mut self, x: i32, y: i32, w: u32, h: u32) {
        Rectangle::new(Point::new(x, y), Size::new(w, h))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
            .draw(&mut self.display)
            .unwrap();
    }

    pub fn println(&mut self, text: &str, x: i32, y: i32) {
        let style = MonoTextStyle::new(&FONT_8X13, Rgb565::RED);
        //Text::with_alignment(text, Point::new(x, y), style, Alignment::Center)
        //    .draw(&mut self.display)
        //    .unwrap();
        Text::with_baseline(text, Point::new(x, y), style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
    }
}
