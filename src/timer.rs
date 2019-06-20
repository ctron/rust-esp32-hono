use alloc::boxed::Box;

use core::ffi::c_void;
use core::ptr::null_mut;
use esp32_sys::*;

use crate::log;
use crate::log::Level;

const TAG: &'static [u8] = b"timer\0";

struct TimerBase {
    callback: Box<FnMut()>,
}

pub struct Timer {
    handle: TimerHandle_t,
}

impl Timer {
    pub fn new<F>(period_ticks: u32, reload: bool, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        let reload = match reload {
            true => 1u32,
            false => 0u32,
        };

        let base = Box::new(TimerBase {
            callback: Box::new(callback),
        });

        unsafe {
            let ptr = Box::into_raw(base) as *mut c_void;

            log::log(Level::DEBUG, &TAG, format_args!("Timer ptr: {:?}", ptr));

            let timer = xTimerCreate(
                b"Timer\0".as_ptr() as *const i8,
                period_ticks,
                reload,
                ptr,
                Some(Timer::handler),
            );

            log::log(
                Level::DEBUG,
                &TAG,
                format_args!("Created timer: {:?}", timer),
            );

            Timer { handle: timer }
        }
    }

    unsafe extern "C" fn handler(handle: *mut c_void) {
        log::log(
            Level::DEBUG,
            &TAG,
            format_args!("Executing timer - handle: {:?}", handle),
        );

        let ptr = pvTimerGetTimerID(handle);
        log::log(
            Level::DEBUG,
            &TAG,
            format_args!("Executing timer - ptr: {:?}", ptr),
        );

        let timer: *mut TimerBase = ptr as *mut TimerBase;
        let timer: &mut TimerBase = &mut *timer;

        (timer.callback)();

        log::log(Level::DEBUG, &TAG, format_args!("Timer executed"));
    }

    pub fn start(&mut self) {
        unsafe {
            xTimerGenericCommand(
                self.handle,
                1, /*START*/
                xTaskGetTickCount(),
                null_mut(),
                0,
            );
        }
    }

    pub fn stop_non_blocking(&mut self) {
        unsafe {
            xTimerGenericCommand(self.handle, 3 /*STOP*/, 0, null_mut(), 0);
        }
    }
}

impl core::ops::Drop for Timer {
    fn drop(&mut self) {
        log::log(Level::INFO, &TAG, format_args!("Dropping timer"));
        self.stop_non_blocking();
    }
}
