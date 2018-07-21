#![allow(bad_style, unused_imports, unused_macros)]
extern crate libc;
extern crate libgpg_error_sys;

use libc::*;
use libgpg_error_sys::*;

include!(concat!(env!("OUT_DIR"), "/all.rs"));
