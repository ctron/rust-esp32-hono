#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate cty;

#[repr(C)]
pub struct _reent {
    d: * const cty::c_void
}

include!("bindings.rs");
