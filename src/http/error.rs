use core::fmt;

use esp32_sys::*;

use cstr_core;

#[derive(Debug)]
pub struct Error {
    message: &'static str,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl From<&'static str> for Error {
    fn from(e: &'static str) -> Self {
        Error { message: e }
    }
}

impl From<cstr_core::NulError> for Error {
    fn from(_e: cstr_core::NulError) -> Self {
        Error {
            message: "Null byte in string",
        }
    }
}

impl From<esp_err_t> for Error {
    fn from(e: esp_err_t) -> Self {
        Error {
            message: err_to_str(e),
        }
    }
}

fn err_to_str(err: esp_err_t) -> &'static str {
    unsafe {
        let err = esp_err_to_name(err);

        return cstr_core::CStr::from_ptr(err).to_str().unwrap();
    }
}
