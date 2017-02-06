extern crate libgpg_error_sys as ffi;

use std::borrow::Cow;
use std::error;
use std::ffi::{CStr, NulError};
use std::fmt;
use std::io::{self, ErrorKind};
use std::os::raw::{c_char, c_int};
use std::result;
use std::str;

pub use ffi::consts::*;

pub type ErrorSource = ffi::gpg_err_source_t;
pub type ErrorCode = ffi::gpg_err_code_t;

/// A type wrapping errors produced by GPG libraries.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Error {
    err: ffi::gpg_error_t,
}

impl Error {
    /// Creates a new error from a raw error value.
    #[inline]
    pub fn new(err: ffi::gpg_error_t) -> Error {
        Error { err: err }
    }

    /// Returns the raw error value that this error wraps.
    #[inline]
    pub fn raw(&self) -> ffi::gpg_error_t {
        self.err
    }

    /// Creates a new error from an error source and an error code.
    #[inline]
    pub fn from_source(source: ErrorSource, code: ErrorCode) -> Error {
        Error::new(ffi::gpg_err_make(source, code))
    }

    /// Creates a new error from an error code using the default
    /// error source `GPG_ERR_SOURCE_USER_1`.
    #[inline]
    pub fn from_code(code: ErrorCode) -> Error {
        Error::from_source(ffi::GPG_ERR_SOURCE_UNKNOWN, code)
    }

    /// Returns an error representing the last OS error that occurred.
    #[inline]
    pub fn last_os_error() -> Error {
        unsafe { Error::new(ffi::gpg_error_from_syserror()) }
    }

    /// Creates a new error from an OS error code.
    #[inline]
    pub fn from_errno(code: i32) -> Error {
        unsafe { Error::new(ffi::gpg_error_from_errno(code as c_int)) }
    }

    /// Returns the OS error that this error represents.
    #[inline]
    pub fn to_errno(&self) -> i32 {
        unsafe { ffi::gpg_err_code_to_errno(self.code()) }
    }

    /// Returns the error code.
    #[inline]
    pub fn code(&self) -> ErrorCode {
        ffi::gpg_err_code(self.err)
    }

    /// Returns a description of the source of the error as a UTF-8 string.
    #[inline]
    pub fn source(&self) -> Option<&'static str> {
        self.raw_source().and_then(|s| str::from_utf8(s).ok())
    }

    /// Returns a description of the source of the error as a slice of bytes.
    #[inline]
    pub fn raw_source(&self) -> Option<&'static [u8]> {
        unsafe {
            let source = ffi::gpg_strsource(self.err);
            if !source.is_null() {
                Some(CStr::from_ptr(source).to_bytes())
            } else {
                None
            }
        }
    }

    /// Returns a printable description of the error.
    #[inline]
    pub fn description(&self) -> Cow<'static, str> {
        let mut buf = [0 as c_char; 0x0400];
        let p = buf.as_mut_ptr();
        unsafe {
            if ffi::gpg_strerror_r(self.err, p, buf.len()) == 0 {
                Cow::Owned(CStr::from_ptr(p).to_string_lossy().into_owned())
            } else {
                Cow::Borrowed("Unknown error")
            }
        }
    }

    /// Returns a description of the error as a slice of bytes.
    #[inline]
    pub fn raw_description(&self) -> Cow<'static, [u8]> {
        let mut buf = [0 as c_char; 0x0400];
        let p = buf.as_mut_ptr();
        unsafe {
            if ffi::gpg_strerror_r(self.err, p, buf.len()) == 0 {
                Cow::Owned(CStr::from_ptr(p).to_bytes().to_owned())
            } else {
                Cow::Borrowed(b"Unknown error")
            }
        }
    }
}

impl error::Error for Error {
    #[inline]
    fn description(&self) -> &str {
        "gpg error"
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Error")
            .field("source", &self.source())
            .field("code", &self.code())
            .field("description", &&(*self.description()))
            .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} (gpg error {})", self.description(), self.code())
    }
}

impl From<NulError> for Error {
    #[inline]
    fn from(_: NulError) -> Error {
        Error::from_code(ffi::GPG_ERR_INV_VALUE)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        let kind = err.kind();
        if let Some(Ok(err)) = err.into_inner().map(|e| e.downcast::<Error>()) {
            *err
        } else {
            let code = match kind {
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
            x if x == GPG_ERR_EAGAIN || x == GPG_ERR_EWOULDBLOCK => ErrorKind::WouldBlock,
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
