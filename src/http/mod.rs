use core::ptr::*;
use core::result;

use cstr_core::CString;

use esp32_sys::*;

mod config;
mod response;

pub use self::config::*;
pub use self::response::*;

use crate::error::*;
use crate::log;
use crate::log::Level;

use generic_array;

const TAG: &'static [u8] = b"http\0";

pub type Result<T> = result::Result<T, EspError>;

struct State {
    url: CString,
    username: Option<CString>,
    password: Option<CString>,
    cert_pem: Option<CString>,
    authentication_header: Option<CString>,
}

pub struct HttpClient {
    client: esp_http_client_handle_t,
    http_config: esp_http_client_config_t,
    state: State,
}

impl HttpClient {
    pub fn new(config: &HttpClientConfig) -> Result<HttpClient> {
        let url = CString::new(config.url)?;

        let username = config.username.map(CString::new).transpose()?;
        let password = config.password.map(CString::new).transpose()?;
        let authentication_header = config.authentication_header.map(CString::new).transpose()?;

        let cert_pem = config.cert_pem.map(CString::new).transpose()?;

        let state = State {
            url,
            username,
            password,
            cert_pem,
            authentication_header,
        };

        let (client, http_config) = Self::create(&state, config);

        Ok(HttpClient {
            client,
            http_config,
            state,
        })
    }

    fn create(
        state: &State,
        config: &HttpClientConfig,
    ) -> (esp_http_client_handle_t, esp_http_client_config_t) {
        let method = match config.method {
            Method::GET => esp_http_client_method_t_HTTP_METHOD_GET,
            Method::POST => esp_http_client_method_t_HTTP_METHOD_POST,
            Method::PUT => esp_http_client_method_t_HTTP_METHOD_PUT,
            Method::DELETE => esp_http_client_method_t_HTTP_METHOD_DELETE,
            Method::HEAD => esp_http_client_method_t_HTTP_METHOD_HEAD,
            Method::PATCH => esp_http_client_method_t_HTTP_METHOD_PATCH,
        };

        let auth_type = match config.authentication_type {
            None => esp_http_client_auth_type_t_HTTP_AUTH_TYPE_NONE,
            Some(AuthenticationType::BASIC) => esp_http_client_auth_type_t_HTTP_AUTH_TYPE_BASIC,
            Some(AuthenticationType::DIGEST) => esp_http_client_auth_type_t_HTTP_AUTH_TYPE_DIGEST,
        };

        let username = match &state.password {
            Some(username) => username.as_ptr(),
            None => null(),
        };

        let password = match &state.password {
            Some(password) => password.as_ptr(),
            None => null(),
        };

        let use_global_ca_store = state.cert_pem.is_some();
        let cert_pem = match &state.cert_pem {
            Some(cert_pem) => cert_pem.as_ptr(),
            None => null(),
        };

        let http_config = esp_http_client_config_t {
            url: state.url.as_ptr(),
            host: null(),
            port: 0,
            username: username,
            password: password,
            auth_type,
            path: null(),
            query: null(),
            cert_pem: cert_pem,
            client_cert_pem: null(),
            client_key_pem: null(),
            method,
            timeout_ms: config.timeout.as_millis() as i32,
            disable_auto_redirect: false,
            max_redirection_count: config.max_redirect as i32,
            event_handler: None,
            transport_type: 0,
            buffer_size: 16 * 1024,
            user_data: null_mut(),
            is_async: false,
            use_global_ca_store,
        };

        unsafe {
            return (esp_http_client_init(&http_config), http_config);
        }
    }

    pub fn send(&mut self) -> Result<Response> {
        unsafe {
            let err = esp_http_client_perform(self.client);

            if err == ESP_OK as _ {
                Ok(Response {
                    status_code: esp_http_client_get_status_code(self.client) as _,
                })
            } else {
                Err(err.into())
            }
        }
    }

    pub fn send_json<B: generic_array::ArrayLength<u8>, T>(
        &mut self,
        payload: &T,
    ) -> Result<Response>
    where
        B: generic_array::ArrayLength<u8>,
        T: serde::ser::Serialize,
    {
        log::log(Level::INFO, &TAG, format_args!("Sending payload"));

        let payload = serde_json_core::ser::to_vec::<B, T>(payload)?;

        log::log(Level::INFO, &TAG, format_args!("Payload encoded"));

        unsafe {
            log::log(Level::INFO, &TAG, format_args!("set content type header"));

            if let Some(authentication_header) = &self.state.authentication_header {
                esp_http_client_set_header(
                    self.client,
                    b"Authorization\0".as_ptr() as *const _,
                    authentication_header.as_ptr(),
                );
            }

            esp_http_client_set_header(
                self.client,
                b"Content-Type\0".as_ptr() as *const _,
                b"application/json\0".as_ptr() as *const _,
            );

            log::log(Level::INFO, &TAG, format_args!("Set post data"));

            err(esp_http_client_set_post_field(
                self.client,
                payload.as_ptr() as *const _,
                payload.len() as _,
            ))?;

            log::log(Level::INFO, &TAG, format_args!("Perform HTTP request"));

            let err = esp_http_client_perform(self.client);

            if err == ESP_OK as _ {
                Ok(Response {
                    status_code: esp_http_client_get_status_code(self.client) as _,
                })
            } else {
                Err(err.into())
            }
        }
    }
}

impl core::ops::Drop for HttpClient {
    fn drop(&mut self) {
        unsafe {
            esp_http_client_cleanup(self.client);
        }
    }
}
