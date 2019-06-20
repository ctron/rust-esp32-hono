use core::time::Duration;

use core::default::Default;

#[derive(Default)]
pub struct HttpClientConfig<'a> {
    pub url: &'a str,
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub method: Method,
    pub timeout: Duration,
    pub max_redirect: u8,
    pub authentication_type: Option<AuthenticationType>,
    pub authentication_header: Option<&'a str>,
    pub cert_pem: Option<&'a str>,
}

// method

pub enum Method {
    GET,
    PUT,
    POST,
    HEAD,
    DELETE,
    PATCH,
}

impl Default for Method {
    fn default() -> Self {
        Method::GET
    }
}

// auth type

pub enum AuthenticationType {
    BASIC,
    DIGEST,
}
