#![allow(non_camel_case_types)]

pub use self::consts::*;
pub use self::funcs::*;
pub use self::types::*;

pub mod types {
    use std::os::raw::c_uint;
    pub type gpg_error_t = c_uint;
    pub type gpg_err_source_t = c_uint;
    pub type gpg_err_code_t = c_uint;
}

pub mod consts {
    use types::gpg_err_code_t;
    use types::gpg_err_source_t;
    use types::gpg_error_t;

    pub const GPG_ERR_SOURCE_DIM: gpg_err_source_t = 128;
    pub const GPG_ERR_SOURCE_MASK: gpg_error_t = (GPG_ERR_SOURCE_DIM as gpg_error_t) - 1;
    pub const GPG_ERR_SOURCE_SHIFT: gpg_error_t = 24;

    pub const GPG_ERR_SYSTEM_ERROR: gpg_err_code_t = 1 << 15;
    pub const GPG_ERR_CODE_DIM: gpg_err_code_t = 65536;
    pub const GPG_ERR_CODE_MASK: gpg_error_t = (GPG_ERR_CODE_DIM as gpg_error_t) - 1;

    include!(concat!(env!("OUT_DIR"), "/constants.rs"));
}

pub mod funcs {
    use std::os::raw::{c_char, c_int};

    use types::gpg_err_code_t;
    use types::gpg_err_source_t;
    use types::gpg_error_t;

    use consts::*;

    #[inline]
    pub fn gpg_err_make(source: gpg_err_source_t, code: gpg_err_code_t) -> gpg_error_t {
        if code == GPG_ERR_NO_ERROR {
            GPG_ERR_NO_ERROR
        } else {
            ((source & GPG_ERR_SOURCE_MASK) << GPG_ERR_SOURCE_SHIFT) | (code & GPG_ERR_CODE_MASK)
        }
    }

    #[inline]
    pub fn gpg_err_code(err: gpg_error_t) -> gpg_err_code_t {
        err & GPG_ERR_CODE_MASK
    }

    #[inline]
    pub fn gpg_err_source(err: gpg_error_t) -> gpg_err_source_t {
        (err >> GPG_ERR_SOURCE_SHIFT) & GPG_ERR_SOURCE_MASK
    }

    #[inline]
    pub unsafe fn gpg_err_make_from_errno(source: gpg_err_source_t, err: c_int) -> gpg_error_t {
        gpg_err_make(source, gpg_error_from_errno(err))
    }

    #[inline]
    pub unsafe fn gpg_error_from_errno(err: c_int) -> gpg_error_t {
        gpg_err_make_from_errno(GPG_ERR_SOURCE_UNKNOWN, err)
    }

    #[inline]
    pub unsafe fn gpg_error_from_syserror() -> gpg_error_t {
        gpg_err_make(GPG_ERR_SOURCE_UNKNOWN, gpg_err_code_from_syserror())
    }

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
