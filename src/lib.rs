extern crate libc;
extern crate libgpg_error_sys as ffi;

use std::borrow::Cow;
use std::error;
use std::ffi::{CStr, NulError};
use std::fmt;
use std::io::{self, ErrorKind};
use std::result;
use std::str;

pub use ffi::consts::*;

pub type ErrorSource = ffi::gpg_err_source_t;
pub type ErrorCode = ffi::gpg_err_code_t;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Error {
    err: ffi::gpg_error_t,
}

impl Error {
    pub fn new(err: ffi::gpg_error_t) -> Error {
        Error { err: err }
    }

    pub fn raw(&self) -> ffi::gpg_error_t {
        self.err
    }

    pub fn from_source(source: ErrorSource, code: ErrorCode) -> Error {
        Error::new(ffi::gpg_err_make(source, code))
    }

    pub fn from_code(code: ErrorCode) -> Error {
        Error::from_source(ffi::GPG_ERR_SOURCE_USER_1, code)
    }

    pub fn last_os_error() -> Error {
        unsafe {
            Error::new(ffi::gpg_error_from_syserror())
        }
    }

    pub fn from_errno(code: i32) -> Error {
        unsafe {
            Error::new(ffi::gpg_error_from_errno(code as libc::c_int))
        }
    }

    pub fn to_errno(&self) -> i32 {
        unsafe {
            ffi::gpg_err_code_to_errno(self.code())
        }
    }

    pub fn code(&self) -> ErrorCode {
        ffi::gpg_err_code(self.err)
    }

    pub fn source(&self) -> Option<&'static str> {
        unsafe {
            let source = ffi::gpg_strsource(self.err);
            if !source.is_null() {
                str::from_utf8(CStr::from_ptr(source).to_bytes()).ok()
            } else {
                None
            }
        }
    }

    pub fn description(&self) -> Cow<'static, str> {
        let mut buf = [0 as libc::c_char; 0x0400];
        let p = buf.as_mut_ptr();
        unsafe {
            let result = if ffi::gpg_strerror_r(self.err, p, buf.len() as libc::size_t) == 0 {
                str::from_utf8(CStr::from_ptr(p).to_bytes()).ok()
            } else {
                None
            };
            result.map_or("Unknown error".into(), |s| s.to_owned().into())
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "gpg error"
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} (gpg error {})", self.description(), self.code())
    }
}

impl From<NulError> for Error {
    fn from(_: NulError) -> Error {
        Error::from_code(ffi::GPG_ERR_INV_VALUE)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        let code = match err.kind() {
            ErrorKind::NotFound => GPG_ERR_ENOENT,
            ErrorKind::PermissionDenied => GPG_ERR_EACCES,
            ErrorKind::ConnectionRefused => GPG_ERR_ECONNREFUSED,
            ErrorKind::ConnectionReset => GPG_ERR_ECONNRESET,
            ErrorKind::ConnectionAborted => GPG_ERR_ECONNABORTED,
            ErrorKind::NotConnected => GPG_ERR_ENOTCONN,
            ErrorKind::AddrInUse => GPG_ERR_EADDRINUSE,
            ErrorKind::AddrNotAvailable => GPG_ERR_EADDRNOTAVAIL,
            ErrorKind::BrokenPipe => GPG_ERR_EPIPE,
            ErrorKind::AlreadyExists => GPG_ERR_EEXIST,
            ErrorKind::WouldBlock => GPG_ERR_EWOULDBLOCK,
            ErrorKind::InvalidInput => GPG_ERR_EINVAL,
            ErrorKind::TimedOut => GPG_ERR_ETIMEDOUT,
            ErrorKind::Interrupted => GPG_ERR_EINTR,
            _ => GPG_ERR_EIO,

        };
        Error::from_code(code)
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        let kind = match err.code() {
            GPG_ERR_ECONNREFUSED => ErrorKind::ConnectionRefused,
            GPG_ERR_ECONNRESET => ErrorKind::ConnectionReset,
            GPG_ERR_EPERM | GPG_ERR_EACCES => ErrorKind::PermissionDenied,
            GPG_ERR_EPIPE => ErrorKind::BrokenPipe,
            GPG_ERR_ENOTCONN => ErrorKind::NotConnected,
            GPG_ERR_ECONNABORTED => ErrorKind::ConnectionAborted,
            GPG_ERR_EADDRNOTAVAIL => ErrorKind::AddrNotAvailable,
            GPG_ERR_EADDRINUSE => ErrorKind::AddrInUse,
            GPG_ERR_ENOENT => ErrorKind::NotFound,
            GPG_ERR_EINTR => ErrorKind::Interrupted,
            GPG_ERR_EINVAL => ErrorKind::InvalidInput,
            GPG_ERR_ETIMEDOUT => ErrorKind::TimedOut,
            GPG_ERR_EEXIST => ErrorKind::AlreadyExists,
            x if x == GPG_ERR_EAGAIN || x == GPG_ERR_EWOULDBLOCK =>
                ErrorKind::WouldBlock,
            _ => ErrorKind::Other,
        };
        io::Error::new(kind, err)
    }
}

pub type Result<T> = result::Result<T, Error>;

#[macro_export]
macro_rules! return_err {
    ($e:expr) => (match $e {
        $crate::GPG_ERR_NO_ERROR => (),
        err => return Err(From::from($crate::Error::new(err))),
    });
}
