use std::sync::{Arc, Mutex};
use std::time::Instant;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::i2c::{I2cDriver, I2cConfig};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::units::Hertz;

use hellomch_mchdisplay::mchdisplay::{Display, Rgb565, RgbColor};
use hellomch_mchcoproc::mchcoproc::Rp2040;

use hellomch::util;

#[cfg(feature = "with-wifi")]
use hellomch::wifi;


const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");
#[cfg(feature = "version-from-env")]
const BUILD_VERSION: &str = env!("BUILD_VERSION");
#[cfg(not(feature = "version-from-env"))]
const BUILD_VERSION: &str = git_version::git_version!();

#[cfg(feature = "with-wifi")]
mod wifi_config {
    pub const DEFAULT_WIFI_SSID: &str = env!("DEFAULT_WIFI_SSID");
    pub const DEFAULT_WIFI_PASSWORD: &str = env!("DEFAULT_WIFI_PASSWORD");
    pub const HUD_URL: &str = env!("HUD_URL");
}


pub type SharedI2c = Arc<Mutex<I2cDriver<'static>>>;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Starting.. waiting 2500ms for debug");
    FreeRtos::delay_ms(2500);
    log::info!("Started {} ts {}", BUILD_VERSION, BUILD_TIMESTAMP);
    util::show_memory_status();

    let peripherals = Peripherals::take().unwrap();

    let mode = peripherals.pins.gpio26; // FPGA vs. ILI
    let mut mode_output = PinDriver::output(mode).unwrap();
    mode_output.set_low().unwrap();
    util::show_memory_status();

    let mut display = Display::new(
        peripherals.spi3,
        peripherals.pins.gpio18.into(), // sclk, clock
        peripherals.pins.gpio23.into(), // mosi/sdo, master out
        peripherals.pins.gpio32.into(), // cs, chip select
        peripherals.pins.gpio25.into(), // reset
        peripherals.pins.gpio33.into(), // dc, data/command
    );
    log::info!("MCH Badge Display inited");
    util::show_memory_status();

    let single_i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio22,     // GPIO_I2C_SDA
        peripherals.pins.gpio21,     // GPIO_I2C_SCL
        &I2cConfig::new().baudrate(Hertz(400_000)),
    ).unwrap();
    let shared_i2c: SharedI2c = Arc::new(Mutex::new(single_i2c));
    let mut rp2040 = Rp2040::new(shared_i2c.clone());

    let rp2040_fw = rp2040.get_firmware_version().unwrap();
    log::info!("RP2040 firmware version: 0x{:02X}", rp2040_fw);

    let s = format!("Hello MCH!\nV:{}\nT:{}\nCO:{:02X}", BUILD_VERSION, BUILD_TIMESTAMP, rp2040_fw);
    display.clear(Rgb565::WHITE);
    display.println(s.as_str(), 0, 0);
    display.flush();
    util::show_memory_status();

    #[cfg(feature = "with-wifi")]
    let maybe_wifi_driver = match wifi::init_wifi_client(
            peripherals.modem,
            wifi_config::DEFAULT_WIFI_SSID,
            wifi_config::DEFAULT_WIFI_PASSWORD) {
        Ok(wifi_driver) => {
            Some(wifi_driver)
        },
        Err(err) => {
            log::error!("bad bad Wifi: {}", err);
            None
        }
    };

    util::show_memory_status();

    #[cfg(feature = "with-wifi")]
    let mut have_wifi = false;
    let mut n = 0_i32;
    let mut s_display = s.clone();
    loop {
        FreeRtos::delay_ms(2000);

        let start = Instant::now();
        if n == 0 {
            display.clear(Rgb565::BLACK);
        } else {
            display.clear(Rgb565::WHITE);
        }
        n = (n + 10) % 60;
        display.println(s_display.as_str(), n, n);
        display.flush();
        log::info!("Update took {} ms", start.elapsed().as_millis());
        util::show_memory_status();

        #[cfg(feature = "with-wifi")]
        if let Some(wifi_driver) = maybe_wifi_driver.as_ref() {
            match wifi_driver.is_connected() {
                Ok(true) => {
                    if !have_wifi {
                        have_wifi = true;
                        println!("IP info: {:?}", wifi_driver.sta_netif().get_ip_info().unwrap());
                        if let Some(body) = wifi::http_get(wifi_config::HUD_URL) {
                            s_display = format!("{}\nWIFI:{}\nBODY:{}", s, wifi_driver.sta_netif().get_ip_info().unwrap().ip, body);
                        } else {
                            s_display = format!("{}\nWIFI:{}", s, wifi_driver.sta_netif().get_ip_info().unwrap().ip);
                        }
                    }
                },
                Ok(false) => {
                    if have_wifi {
                        have_wifi = false;
                        println!("Lost wifi :(");
                        s_display = format!("{}\nWIFI:false", s);
                    }
                },
                Err(err) => {
                    if have_wifi {
                        have_wifi = false;
                        println!("Lost wifi because: {:?}", err);
                        s_display = format!("{}\nWIFI:error", s);
                    }
                },
            }
        }
    }
}
