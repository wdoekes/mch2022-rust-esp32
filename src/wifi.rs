use anyhow::{anyhow, Error};

use embedded_svc::{
    http::{client::Client as HttpClient, Method},
    utils::io,
};

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::http::client::EspHttpConnection;
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
        ssid: heapless::String::<32>::try_from(ssid).map_err(|_| anyhow!("ssid conversion failed"))?,
        password: heapless::String::<64>::try_from(password).map_err(|_| anyhow!("password conversion failed"))?,
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

pub fn http_get(url: &str) -> Option<String> {
    let Ok(esp_conn) = EspHttpConnection::new(&Default::default()) else {
        log::error!("GET {} failed to init EspHttpConnection", url);
        return None;
    };
    let mut client = HttpClient::wrap(esp_conn);

    // Prepare headers and URL
    let headers = [
        ("accept", "text/plain"),
        ("connection", "close"),
    ];

    // Note: If you don't want to pass in any headers, you can also use `client.get(url, headers)`.
    let Ok(request) = client.request(Method::Get, url, &headers) else {
        log::error!("GET {} failed to make GET request", url);
        return None;
    };
    let Ok(mut response) = request.submit() else {
        log::error!("GET {} failed to submit", url);
        return None;
    };

    // Process response
    let status = response.status();
    let mut buf = [0u8; 1024];
    let Ok(bytes_read) = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0) else {
        log::error!("GET {} status {} failed to get body", url, status);
        return None;
    };

    let Ok(body_string) = std::str::from_utf8(&buf[0..bytes_read]) else {
        log::error!(
            "GET {} got {} with {} (possibly truncated) bytes (decode error)",
            url, status, bytes_read);
        return None;
    };

    if status != 200 {
        log::error!(
            "GET {} got {} with {} (possibly truncated) bytes: {:?}",
            url, status, buf.len(), body_string);
        return None;
    }

    log::info!(
        "GET {} got {} with {} (possibly truncated) bytes: {:?}",
        url, status, buf.len(), body_string);

    Some(body_string.to_string())
}
