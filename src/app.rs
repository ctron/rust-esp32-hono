use crate::error::{err, err_check, EspError};
use crate::log;
use crate::log::Level;
use crate::timer;

use esp32_sys::*;

use core::ffi::c_void;

pub struct Application {
    timer: timer::Timer,

    ssid: &'static str,
    password: &'static str,

    event_group: EventGroupHandle_t,

    retry_num: u32,
}

fn nvs_init() -> Result<(), EspError> {
    unsafe {
        let mut ret = nvs_flash_init();
        if ret == ESP_ERR_NVS_NO_FREE_PAGES as i32 || ret == ESP_ERR_NVS_NEW_VERSION_FOUND as i32 {
            log::log(
                Level::INFO,
                &TAG,
                format_args!("Need to erase flash: rc = {}", ret),
            );
            err(nvs_flash_erase())?;
            ret = nvs_flash_init();
        }
        err(ret)
    }
}

const WIFI_CONNECTED_BIT: u32 = BIT0;
const APP_ESP_MAXIMUM_RETRY: u32 = 10u32;
const TAG: &'static [u8] = b"app\0";

unsafe fn default_config() -> wifi_init_config_t {
    wifi_init_config_t {
        event_handler: Some(esp_event_send),
        osi_funcs: &mut g_wifi_osi_funcs,
        wpa_crypto_funcs: g_wifi_default_wpa_crypto_funcs,
        static_rx_buf_num: CONFIG_ESP32_WIFI_STATIC_RX_BUFFER_NUM as i32,
        dynamic_rx_buf_num: CONFIG_ESP32_WIFI_DYNAMIC_RX_BUFFER_NUM as i32,
        tx_buf_type: CONFIG_ESP32_WIFI_TX_BUFFER_TYPE as i32,
        static_tx_buf_num: WIFI_STATIC_TX_BUFFER_NUM as i32,
        dynamic_tx_buf_num: WIFI_DYNAMIC_TX_BUFFER_NUM as i32,
        csi_enable: WIFI_CSI_ENABLED as i32,
        ampdu_rx_enable: WIFI_AMPDU_RX_ENABLED as i32,
        ampdu_tx_enable: WIFI_AMPDU_TX_ENABLED as i32,
        nvs_enable: WIFI_NVS_ENABLED as i32,
        nano_enable: WIFI_NANO_FORMAT_ENABLED as i32,
        tx_ba_win: WIFI_DEFAULT_TX_BA_WIN as i32,
        rx_ba_win: WIFI_DEFAULT_RX_BA_WIN as i32,
        wifi_task_core_id: WIFI_TASK_CORE_ID as i32,
        beacon_max_len: WIFI_SOFTAP_BEACON_MAX_LEN as i32,
        mgmt_sbuf_num: WIFI_MGMT_SBUF_NUM as i32,
        magic: WIFI_INIT_CONFIG_MAGIC as i32,
    }
}

unsafe extern "C" fn event_handler(ctx: *mut cty::c_void, event: *mut system_event_t) -> esp_err_t {
    log::log(
        Level::INFO,
        &TAG,
        format_args!("event_handler - {}", (*event).event_id),
    );

    let app: *mut Application = ctx as *mut Application;
    let app: &mut Application = &mut *app;

    app.event_handler(&*event)
        .map_or_else(|e| e.code(), |_| ESP_OK as i32)
}


const SNTP_SERVER_NAME: &'static [u8] = b"0.pool.ntp.org\0";

unsafe fn wifi_init_sta(app: &mut Application, ssid: &'static str, password: &'static str) {
    log::log(
        Level::INFO,
        &TAG,
        format_args!("STA - SSID: {:?}, password: {:?}", ssid, password),
    );

    log::log(Level::INFO, &TAG, format_args!("Init TCP/IP"));

    tcpip_adapter_init();

    log::log(Level::INFO, &TAG, format_args!("Init event loop"));

    err_check(esp_event_loop_init(
        Some(event_handler),
        (app as *mut Application) as *mut c_void,
    ));

    let cfg = default_config();

    log::log(Level::INFO, &TAG, format_args!("Init wifi"));

    err_check(esp_wifi_init(&cfg));

    log::log(Level::INFO, &TAG, format_args!("Wifi initialized"));

    let mut fixed_ssid: [u8; 32] = [0; 32];
    {
        let b = ssid.as_bytes();
        fixed_ssid.split_at_mut(b.len()).0.copy_from_slice(b);
    }

    let mut fixed_password: [u8; 64] = [0; 64];
    {
        let b = password.as_bytes();
        fixed_password.split_at_mut(b.len()).0.copy_from_slice(b);
    }

    log::log(
        Level::DEBUG,
        &TAG,
        format_args!(
            "SSID: {:?} Password: {:?}",
            &fixed_ssid[..],
            &fixed_password[..]
        ),
    );

    let mut wifi_config = wifi_config_t {
        sta: wifi_sta_config_t {
            ssid: fixed_ssid,
            password: fixed_password,

            bssid_set: false,
            bssid: [0, 0, 0, 0, 0, 0],

            channel: 0,
            listen_interval: 0,
            scan_method: wifi_scan_method_t_WIFI_FAST_SCAN,
            sort_method: wifi_sort_method_t_WIFI_CONNECT_AP_BY_SIGNAL,
            threshold: wifi_fast_scan_threshold_t {
                rssi: 0,
                authmode: wifi_auth_mode_t_WIFI_AUTH_OPEN,
            },
        },
    };

    log::log(Level::INFO, &TAG, format_args!("Wifi set mode"));
    err_check(esp_wifi_set_mode(wifi_mode_t_WIFI_MODE_STA));
    log::log(Level::INFO, &TAG, format_args!("Wifi set config"));
    err_check(esp_wifi_set_config(
        esp_interface_t_ESP_IF_WIFI_STA,
        &mut wifi_config,
    ));

    log::log(Level::INFO, &TAG, format_args!("Start wifi"));

    err_check(esp_wifi_start());

    log::log(Level::INFO, &TAG, format_args!("Wifi started"));

    sntp_setoperatingmode(SNTP_OPMODE_POLL as _);
    sntp_setservername(0, SNTP_SERVER_NAME.as_ptr() as _);
    sntp_init();

}

impl Application {
    pub fn new<F>(ssid: &'static str, password: &'static str, ticked: F) -> Application
    where
        F: FnMut() + 'static,
    {
        let event_group = unsafe { xEventGroupCreate() };

        Application {
            ssid,
            password,
            event_group,
            retry_num: 0,
            timer: timer::Timer::new(200, true, ticked),
        }
    }

    pub fn run(&mut self) -> ! {
        nvs_init().expect("Failed to init NVS");
        unsafe {
            wifi_init_sta(self, self.ssid, self.password);
            loop {
                ets_delay_us(1_000_000);
            }
        }
    }

    fn event_handler(&mut self, event: &system_event_t) -> Result<(), EspError> {
        #[allow(non_upper_case_globals)]
        match event.event_id {
            system_event_id_t_SYSTEM_EVENT_STA_START => {
                log::log(Level::INFO, &TAG, format_args!("Started"));

                unsafe {
                    esp_wifi_connect();
                }
            }
            system_event_id_t_SYSTEM_EVENT_STA_GOT_IP => {
                log::log(Level::INFO, &TAG, format_args!("Got IP"));
                self.retry_num = 0;

                unsafe {
                    xEventGroupSetBits(self.event_group, WIFI_CONNECTED_BIT);
                }

                log::log(Level::DEBUG, &TAG, format_args!("Starting timer"));
                self.timer.start();
                log::log(Level::DEBUG, &TAG, format_args!("Timer started"));
            }
            system_event_id_t_SYSTEM_EVENT_STA_DISCONNECTED => {
                log::log(
                    Level::INFO,
                    &TAG,
                    format_args!("Disconnected - retry: {}", self.retry_num),
                );
                log::log(Level::DEBUG, &TAG, format_args!("Stopping timer"));
                self.timer.stop_non_blocking();
                log::log(Level::DEBUG, &TAG, format_args!("Timer stopped"));
                if self.retry_num < APP_ESP_MAXIMUM_RETRY {
                    unsafe {
                        esp_wifi_connect();
                        xEventGroupClearBits(self.event_group, WIFI_CONNECTED_BIT);
                    }
                    self.retry_num += 1;
                }
            }
            _ => {}
        };

        Ok(())
    }
}
