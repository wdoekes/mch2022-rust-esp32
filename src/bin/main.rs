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

    let mode = peripherals.pins.gpio26; // FPGA vs. ILI
    let mut mode_output = PinDriver::output(mode).unwrap();
    mode_output.set_low().unwrap();

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

    let s = format!("Hello MCH build {}", BUILD_TIMESTAMP);
    display.clear(Rgb565::WHITE);
    display.println(s.as_str(), 0, 0);
    display.flush();
    util::show_memory_status();

    let mut n = 0_i32;
    loop {
        FreeRtos::delay_ms(2000);

        let start = Instant::now();
        if n == 0 {
            display.clear(Rgb565::BLACK);
        } else {
            display.clear(Rgb565::WHITE);
        }
        n = (n + 10) % 60;
        display.println(s.as_str(), n, n);
        display.flush();
        log::info!("Update took {} ms", start.elapsed().as_millis());
        util::show_memory_status();
    }
}
