use anyhow::{anyhow, Error};
use heapless::String;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{
    ClientConfiguration as WifiClientConfiguration,
    Configuration as WifiConfiguration,
    EspWifi,
};

use crate::util;


pub fn init_wifi_client<'a>(modem: Modem, ssid: &str, password: &str) -> Result<EspWifi<'a>, Error> {
    let sys_loop = EspSystemEventLoop::take()?;
    util::show_memory_status();
    let nvs = EspDefaultNvsPartition::take()?;
    util::show_memory_status();

    let mut wifi_driver = EspWifi::new(
        modem,
        sys_loop,
        Some(nvs)
    ).unwrap();
    util::show_memory_status();

    wifi_driver.set_configuration(&WifiConfiguration::Client(WifiClientConfiguration{
        ssid: String::<32>::try_from(ssid).map_err(|_| anyhow!("ssid conversion failed"))?,
        password: String::<64>::try_from(password).map_err(|_| anyhow!("password conversion failed"))?,
        ..Default::default()
    })).unwrap();
    util::show_memory_status();

    wifi_driver.start().unwrap();
    util::show_memory_status();
    wifi_driver.connect().unwrap();
    util::show_memory_status();

    log::info!("MCH Wifi inited");

    Ok(wifi_driver)
}
