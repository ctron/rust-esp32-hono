use esp32_sys::*;

use alloc::fmt;
use core::fmt::Arguments;
use cstr_core::{CStr, CString};

pub enum Level {
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

pub fn log(level: Level, tag: &[u8], args: Arguments) {
    let c = CString::new(fmt::format(args)).expect("");
    let (format, level) = match level {
        Level::ERROR => ("E (%d) %s: %s\n\0", esp_log_level_t_ESP_LOG_ERROR),
        Level::WARN => ("W (%d) %s: %s\n\0", esp_log_level_t_ESP_LOG_WARN),
        Level::INFO => ("I (%d) %s: %s\n\0", esp_log_level_t_ESP_LOG_INFO),
        Level::DEBUG => ("D (%d) %s: %s\n\0", esp_log_level_t_ESP_LOG_DEBUG),
    };
    unsafe {
        let tag = CStr::from_bytes_with_nul_unchecked(tag);
        esp_log_write(
            level,
            tag.as_ptr() as *const i8,
            format.as_ptr() as *const i8,
            esp_log_timestamp(),
            tag.as_ptr(),
            c.as_ptr(),
        );
    }
}
