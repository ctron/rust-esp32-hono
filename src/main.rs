#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(result_map_or_else)]

extern crate cstr_core;
extern crate esp32_sys;
extern crate esp_idf_alloc;

extern crate alloc;

#[global_allocator]
static A: esp_idf_alloc::EspIdfAllocator = esp_idf_alloc::EspIdfAllocator;

use core::alloc::Layout;

extern "C" {
    fn abort() -> !;
}

#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    unsafe {
        abort();
    }
}

use core::panic::PanicInfo;
use esp32_sys::*;

mod error;
mod http;
pub mod log;
pub mod timer;
mod uart;

mod app;

use log::Level;
use serde::{Deserialize, Serialize};

const TAG: &'static [u8] = b"app\0";

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::log(Level::ERROR, &TAG, format_args!("{}", info));
    loop {}
}

extern "C" {
    fn temprature_sens_read() -> u8;
}

fn temperate_sensor_read() -> f32 {
    unsafe {
        // as f32, to DegC
        temprature_sens_read() as f32 - 32.0 / 1.8
    }
}

const WIFI_SSID: &str = "xx";
const WIFI_PASSWORD: &str = "xx";
const HONO_HTTP_ADAPTER_URL: &str =
    "https://iot-http-adapter-enmasse-infra.apps.wonderful.iot-playground.org/telemetry";
const HONO_DEVICE_AUTH_ID: &str = "xx@rhte2019.iot";
const HONO_DEVICE_PASSWORD: &str = "xx";
const HONO_AUTH_HEADER: &str = "Basic xx";

#[derive(Serialize, Deserialize, Debug)]
pub struct TelemetryPayload {
    temp: f32,
}

fn publish_telemetry(client: &mut http::HttpClient, temp: f32) -> http::Result<http::Response> {
    let payload = TelemetryPayload { temp: temp };
    client.send_json::<heapless::consts::U128, TelemetryPayload>(&payload)
}

/// Initialize the global CA store with the Let's Encrypt 
/// Certificate in the DER format.
fn init_global_ca_store() {
    let cert = include_bytes!("letsencrypt.der");

    unsafe {
        esp_tls_set_global_ca_store(cert.as_ptr(), cert.len() as _);
    }
}

#[no_mangle]
pub fn app_main() {
    log::log(Level::INFO, &TAG, format_args!("Hello World"));

    init_global_ca_store();

    let config = http::HttpClientConfig {
        url: HONO_HTTP_ADAPTER_URL,
        authentication_type: Some(http::AuthenticationType::BASIC),
        authentication_header: Some(HONO_AUTH_HEADER),
        method: http::Method::POST,
        ..Default::default()
    };

    let mut http = http::HttpClient::new(&config).expect("Failed to init HTTP client");

    let mut app = app::Application::new(WIFI_SSID, WIFI_PASSWORD, move || {
        let temp = temperate_sensor_read();
        log::log(Level::INFO, &TAG, format_args!("Ticked: {}", temp));
        if let Err(err) = publish_telemetry(&mut http, temp) {
            log::log(
                Level::ERROR,
                &TAG,
                format_args!("Failed to execute HTTP upload: {}", err),
            );
        }
    });

    app.run();
}
