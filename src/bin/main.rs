use std::time::Instant;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::prelude::Peripherals;

use hellomch_mchdisplay::mchdisplay::{Display, Rgb565, RgbColor};

use hellomch::util;


const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");
#[cfg(feature = "version-from-env")]
const BUILD_VERSION: &str = env!("BUILD_VERSION");
#[cfg(not(feature = "version-from-env"))]
const BUILD_VERSION: &str = git_version::git_version!();


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

    /*
    esp_alloc::heap_allocator!(size: 72 * 1024);
    println!("Alloc inited");

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let _init = esp_wifi::init(
        timg0.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();
    println!("Wifi inited");
    */

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let mode = pins.gpio26; // FPGA vs. ILI
    let mut mode_output = PinDriver::output(mode).unwrap();
    mode_output.set_low().unwrap();

    let sclk = pins.gpio18;
    let mosi = pins.gpio23; // sdo
    let cs = pins.gpio32;
    let rst = pins.gpio25;
    let dc = pins.gpio33;

    let mut tft = Display::new(
        peripherals.spi3,
        sclk.into(),
        mosi.into(),
        cs.into(),
        rst.into(),
        dc.into(),
    );
    let s = format!("Hello MCH build {}", BUILD_TIMESTAMP);
    tft.clear(Rgb565::WHITE);
    tft.println(s.as_str(), 0, 0);
    tft.flush();
    log::info!("MCH Badge Display inited");
    util::show_memory_status();

    let mut n = 0_i32;
    loop {
        FreeRtos::delay_ms(2000);

        let start = Instant::now();
        if n == 0 {
            tft.clear(Rgb565::BLACK);
        } else {
            tft.clear(Rgb565::WHITE);
        }
        n = (n + 10) % 60;
        tft.println(s.as_str(), n, n);
        tft.flush();

        log::info!("Update took {} ms", start.elapsed().as_millis());
        util::show_memory_status();
    }
}
