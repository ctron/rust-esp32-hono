use core::fmt;

use esp32_sys::*;

#[derive(Debug)]
pub struct EspError {
    code: i32,
}

impl EspError {
    pub fn code(&self) -> i32 {
        self.code
    }
}

impl fmt::Display for EspError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "code: {}", self.code)
    }
}

impl From<i32> for EspError {
    fn from(e: i32) -> Self {
        EspError { code: e }
    }
}

impl From<cstr_core::NulError> for EspError {
    fn from(_e: cstr_core::NulError) -> Self {
        EspError { code: 1 }
    }
}

impl From<serde_json_core::ser::Error> for EspError {
    fn from(_e: serde_json_core::ser::Error) -> Self {
        EspError { code: 2 }
    }
}

impl Into<i32> for EspError {
    fn into(self) -> i32 {
        self.code
    }
}

pub fn err_check(err: i32) {
    if err != ESP_OK as i32 {
        panic!("Failed - rc = {}", err);
    }
}

pub fn err(err: i32) -> Result<(), EspError> {
    if err != ESP_OK as i32 {
        return Err(err.into());
    } else {
        return Ok(());
    }
}

pub fn err_to_str(err: esp_err_t) -> &'static str {
    unsafe {
        let err = esp_err_to_name(err);

        return cstr_core::CStr::from_ptr(err).to_str().unwrap();
    }
}
