#![no_std]
#![no_main]
#![feature(alloc)]
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

use core::fmt::Write;

use alloc::string::String;

mod error;
mod http;
pub mod log;
pub mod timer;
mod uart;

mod app;

use uart::Uart;

use log::Level;
use serde::{Deserialize, Serialize};

const TAG: &'static [u8] = b"app\0";

// should no longer be necessary
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::log(Level::ERROR, &TAG, format_args!("{}", info));
    loop {}
}

const BLINK_GPIO: gpio_num_t = gpio_num_t_GPIO_NUM_2;
const UART_NUM: uart_port_t = uart_port_t_UART_NUM_1;
//const ECHO_TEST_TXD: i32 = gpio_num_t_GPIO_NUM_17 as i32;
//const ECHO_TEST_RXD: i32 = gpio_num_t_GPIO_NUM_16 as i32;
const ECHO_TEST_TXD: i32 = gpio_num_t_GPIO_NUM_1 as i32;
const ECHO_TEST_RXD: i32 = gpio_num_t_GPIO_NUM_3 as i32;
const ECHO_TEST_RTS: i32 = UART_PIN_NO_CHANGE;
const ECHO_TEST_CTS: i32 = UART_PIN_NO_CHANGE;

const BUF_SIZE: i32 = 1024;

static mut DEBUG: Uart = Uart { port: UART_NUM };

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
    client.send_json::<heapless::consts::U128, TelemetryPayload>(&TelemetryPayload { temp: temp })
}

#[no_mangle]
pub fn app_main() {
    log::log(Level::INFO, &TAG, format_args!("Hello World"));

    //    log::log(Level::INFO, &TAG, format_args!("Cert: {}", CERT));

    let cert = include_bytes!("letsencrypt.der");

    unsafe {
        esp_tls_set_global_ca_store(cert.as_ptr(), cert.len() as _);
    }

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

fn send_http() -> http::Result<http::Response> {
    let config = http::HttpClientConfig {
        url: "http://google.com",
        ..Default::default()
    };
    let mut client = http::HttpClient::new(&config)?;
    client.send()
}

unsafe fn rust_blink_and_write() {
    gpio_pad_select_gpio(BLINK_GPIO as u8);

    /* Set the GPIO as a push/pull output */
    gpio_set_direction(BLINK_GPIO, gpio_mode_t_GPIO_MODE_OUTPUT);

    /* Configure parameters of an UART driver,
     * communication pins and install the driver */
    /*
    let uart_config = uart_config_t {
        baud_rate: 115_200,
        data_bits: uart_word_length_t_UART_DATA_8_BITS,
        parity: uart_parity_t_UART_PARITY_DISABLE,
        stop_bits: uart_stop_bits_t_UART_STOP_BITS_1,
        flow_ctrl: uart_hw_flowcontrol_t_UART_HW_FLOWCTRL_DISABLE,
        rx_flow_ctrl_thresh: 0,
        use_ref_tick: false,
    };

    uart_param_config(UART_NUM, &uart_config);
    uart_set_pin(
        UART_NUM,
        ECHO_TEST_TXD,
        ECHO_TEST_RXD,
        ECHO_TEST_RTS,
        ECHO_TEST_CTS,
    );
    uart_driver_install(UART_NUM, BUF_SIZE * 2, 0, 0, ptr::null_mut(), 0);
    */

    let mut i = 0;

    loop {
        /* Blink off (output low) */
        gpio_set_level(BLINK_GPIO, 0);

        //vTaskDelay(1000 / portTICK_PERIOD_MS);
        ets_delay_us(1_000_000);

        i += 1;

        // Write data to UART.
        // let test_str = "This is a test string.\n";
        let mut output = String::new();
        // writeln!(&mut output, "Hello {}\r", i).expect("");

        // uart_write_bytes(UART_NUM, test_str.as_ptr() as *const _, test_str.len());
        uart_write_bytes(UART_NUM, output.as_ptr() as *const _, output.len());

        /* Blink on (output high) */
        gpio_set_level(BLINK_GPIO, 1);

        writeln!(DEBUG, "Hello World!\r").expect("");

        // do http call

        do_http();

        // vTaskDelay(1000 / portTICK_PERIOD_MS);
        ets_delay_us(1_000_000);
    }
}

fn do_http() {
    let response = send_http();
    match response {
        Err(e) => log::log(Level::INFO, &TAG, format_args!("Error: {}", e)),
        Ok(r) => log::log(
            Level::INFO,
            &TAG,
            format_args!("Response - Code: {}", r.status_code),
        ),
    };
}
