#![allow(nonstandard_style)]
#![no_std]
pub use self::{consts::*, funcs::*, types::*};

pub mod types {
    use core::ffi::c_uint;

    pub type gpg_error_t = c_uint;
    pub type gpg_err_source_t = c_uint;
    pub type gpg_err_code_t = c_uint;
}

pub mod consts {
    use crate::types::{gpg_err_code_t, gpg_err_source_t, gpg_error_t};

    pub const GPG_ERR_SOURCE_DIM: gpg_err_source_t = 128;
    pub const GPG_ERR_SOURCE_MASK: gpg_error_t = (GPG_ERR_SOURCE_DIM as gpg_error_t) - 1;
    pub const GPG_ERR_SOURCE_SHIFT: gpg_error_t = 24;

    pub const GPG_ERR_SYSTEM_ERROR: gpg_err_code_t = 1 << 15;
    pub const GPG_ERR_CODE_DIM: gpg_err_code_t = 65536;
    pub const GPG_ERR_CODE_MASK: gpg_error_t = (GPG_ERR_CODE_DIM as gpg_error_t) - 1;

    include!("consts.rs");
}

pub mod funcs {
    use core::ffi::{c_char, c_int};

    use crate::types::{gpg_err_code_t, gpg_err_source_t, gpg_error_t};

    use crate::consts::*;

    #[inline]
    pub const fn gpg_err_make(source: gpg_err_source_t, code: gpg_err_code_t) -> gpg_error_t {
        let code = code & GPG_ERR_CODE_MASK;
        let source = source & GPG_ERR_SOURCE_MASK;
        if code == GPG_ERR_NO_ERROR {
            code
        } else {
            code | (source << GPG_ERR_SOURCE_SHIFT)
        }
    }

    #[inline]
    pub const fn gpg_err_code(err: gpg_error_t) -> gpg_err_code_t {
        err & GPG_ERR_CODE_MASK
    }

    #[inline]
    pub const fn gpg_err_source(err: gpg_error_t) -> gpg_err_source_t {
        (err >> GPG_ERR_SOURCE_SHIFT) & GPG_ERR_SOURCE_MASK
    }

    #[inline]
    pub unsafe fn gpg_err_make_from_errno(source: gpg_err_source_t, err: c_int) -> gpg_error_t {
        gpg_err_make(source, gpg_err_code_from_errno(err))
    }

    #[inline]
    pub unsafe fn gpg_error_from_errno(err: c_int) -> gpg_error_t {
        gpg_err_make_from_errno(GPG_ERR_SOURCE_UNKNOWN, err)
    }

    #[inline]
    pub unsafe fn gpg_error_from_syserror() -> gpg_error_t {
        gpg_err_make(GPG_ERR_SOURCE_UNKNOWN, gpg_err_code_from_syserror())
    }

    #[cfg_attr(
        all(windows, feature = "windows_raw_dylib"),
        link(
            name = "libgpg-error-0.dll",
            kind = "raw-dylib",
            modifiers = "+verbatim"
        )
    )]
    extern "C" {
        pub fn gpg_err_init() -> gpg_error_t;
        pub fn gpg_err_deinit(mode: c_int);

        pub fn gpg_strerror(err: gpg_error_t) -> *const c_char;
        pub fn gpg_strerror_r(err: gpg_error_t, buf: *mut c_char, buflen: usize) -> c_int;

        pub fn gpg_strsource(err: gpg_error_t) -> *const c_char;

        pub fn gpg_err_code_from_errno(err: c_int) -> gpg_err_code_t;
        pub fn gpg_err_code_to_errno(code: gpg_err_code_t) -> c_int;
        pub fn gpg_err_code_from_syserror() -> gpg_err_code_t;

        pub fn gpg_err_set_errno(err: c_int);

        pub fn gpg_error_check_version(req_version: *const c_char) -> *const c_char;
    }
}
