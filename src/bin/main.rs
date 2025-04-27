#![no_std]
#![no_main]

// Pull in alloc even though we're using no_std.
extern crate alloc;
use alloc::format;

use esp_hal::main;

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::time::{Duration, Instant};
use esp_hal::timer::timg::TimerGroup;

use esp_println::println;

use hellomch_mchdisplay::mchdisplay::{Display, Rgb565, RgbColor};

const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");
#[cfg(feature = "version-from-env")]
const BUILD_VERSION: &str = env!("BUILD_VERSION");
#[cfg(not(feature = "version-from-env"))]
const BUILD_VERSION: &str = git_version::git_version!();


#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[main]
fn main() -> ! {
    // generator version: 0.3.1
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    println!("Starting.. waiting 2500ms for debug");
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(2500) {}
    println!("Started {} ts {}", BUILD_VERSION, BUILD_TIMESTAMP);

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

    let mode = peripherals.GPIO26;
    let _mode_output = Output::new(mode, Level::Low, OutputConfig::default());

    let dc = peripherals.GPIO33;
    let mosi = peripherals.GPIO23; // sdo -> MOSI
    let sclk = peripherals.GPIO18;
    let miso = peripherals.GPIO21; // sdi -> MISO
    let cs = peripherals.GPIO32; // or also 21 ??
    let rst = peripherals.GPIO25;

    let mut tft = Display::new(peripherals.SPI3, sclk, miso, mosi, cs, rst, dc);
    let s = format!("Hello MCH build {}", BUILD_TIMESTAMP);
    tft.clear(Rgb565::WHITE);
    tft.println(s.as_str(), 0, 0);
    println!("MCH Badge Display inited");

    let mut n = 0_i32; 
    loop {
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(1000) {}
        println!("bla bla bla");
        tft.clear(Rgb565::WHITE);
        tft.println(s.as_str(), 140 + n, 40 + n);
        n = (n + 10) % 60;
    }

    // for inspiration have a look at the examples at
    // https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.0/examples/src/bin
}
