use std::{
    borrow::Cow,
    convert::Infallible,
    error,
    ffi::{CStr, NulError},
    fmt::{self, Write},
    io::{self, ErrorKind},
    num::TryFromIntError,
    os::raw::c_int,
    result, str,
};

pub type ErrorSource = ffi::gpg_err_source_t;
pub type ErrorCode = ffi::gpg_err_code_t;

/// A type wrapping errors produced by GPG libraries.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Error(ffi::gpg_error_t);

include!("consts.rs");

impl Error {
    /// Creates a new error from a raw error value.
    #[inline]
    pub const fn new(err: ffi::gpg_error_t) -> Self {
        Self(err)
    }

    /// Returns the raw error value that this error wraps.
    #[inline]
    pub const fn raw(&self) -> ffi::gpg_error_t {
        self.0
    }

    /// Creates a new error from an error source and an error code.
    #[inline]
    pub fn from_source(source: ErrorSource, code: ErrorCode) -> Self {
        Error::new(ffi::gpg_err_make(source, code))
    }

    /// Creates a new error from an error code using the default
    /// error source `SOURCE_UNKNOWN`.
    #[inline]
    pub fn from_code(code: ErrorCode) -> Self {
        Error::from_source(Self::SOURCE_UNKNOWN, code)
    }

    /// Returns an error representing the last OS error that occurred.
    #[inline]
    pub fn last_os_error() -> Self {
        unsafe { Error::new(ffi::gpg_error_from_syserror()) }
    }

    /// Creates a new error from an OS error code.
    #[inline]
    pub fn from_errno(code: i32) -> Self {
        unsafe { Error::new(ffi::gpg_error_from_errno(code as c_int)) }
    }

    /// Returns the OS error that this error represents.
    #[inline]
    pub fn to_errno(&self) -> i32 {
        unsafe { ffi::gpg_err_code_to_errno(self.code()) }
    }

    /// Returns the error code.
    #[inline]
    pub const fn code(&self) -> ErrorCode {
        ffi::gpg_err_code(self.0)
    }

    /// Returns a description of the source of the error as a UTF-8 string.
    #[inline]
    pub fn source(&self) -> Option<&'static str> {
        self.raw_source().and_then(|s| str::from_utf8(s).ok())
    }

    /// Returns an `Error` with the same code from the provided source.
    #[inline]
    pub fn with_source(&self, src: ErrorSource) -> Self {
        Error::from_source(src, self.code())
    }

    /// Returns a description of the source of the error as a slice of bytes.
    #[inline]
    pub fn raw_source(&self) -> Option<&'static [u8]> {
        unsafe {
            ffi::gpg_strsource(self.0)
                .as_ref()
                .map(|s| CStr::from_ptr(s).to_bytes())
        }
    }

    /// Returns a printable description of the error.
    #[inline]
    pub fn description(&self) -> Cow<'static, str> {
        let mut buf = [0; 1024];
        match self.write_description(&mut buf) {
            Ok(b) => Cow::Owned(String::from_utf8_lossy(b).into_owned()),
            Err(_) => Cow::Borrowed("Unknown error"),
        }
    }

    /// Returns a description of the error as a slice of bytes.
    #[inline]
    pub fn raw_description(&self) -> Cow<'static, [u8]> {
        let mut buf = [0; 1024];
        match self.write_description(&mut buf) {
            Ok(b) => Cow::Owned(b.to_owned()),
            Err(_) => Cow::Borrowed(b"Unknown error"),
        }
    }

    /// Writes a description of the error to the provided buffer
    /// and returns a slice of the buffer containing the description.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided buffer is not long enough or
    /// if the error is not recognized.
    #[inline]
    pub fn write_description<'r>(&self, buf: &'r mut [u8]) -> result::Result<&'r mut [u8], ()> {
        let p = buf.as_mut_ptr();
        unsafe {
            if ffi::gpg_strerror_r(self.0, p as *mut _, buf.len()) == 0 {
                match buf.iter().position(|&b| b == b'\0') {
                    Some(x) => Ok(&mut buf[..x]),
                    None => Ok(buf),
                }
            } else {
                Err(())
            }
        }
    }
}

impl From<ffi::gpg_error_t> for Error {
    #[inline]
    fn from(e: ffi::gpg_error_t) -> Self {
        Self::new(e)
    }
}

impl error::Error for Error {
    #[inline]
    fn description(&self) -> &str {
        "gpg error"
    }
}

struct Escaped<'a>(&'a [u8]);
impl fmt::Debug for Escaped<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('"')?;
        for b in self.0.iter().flat_map(|&b| ::std::ascii::escape_default(b)) {
            f.write_char(b as char)?;
        }
        f.write_char('"')
    }
}

impl fmt::Display for Escaped<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = self.0;
        loop {
            match str::from_utf8(buf) {
                Ok(s) => {
                    f.write_str(s)?;
                    break;
                }
                Err(e) => {
                    let (valid, broken) = buf.split_at(e.valid_up_to());
                    f.write_str(unsafe { str::from_utf8_unchecked(valid) })?;
                    f.write_char(::std::char::REPLACEMENT_CHARACTER)?;
                    match e.error_len() {
                        Some(l) => buf = &broken[l..],
                        None => break,
                    }
                }
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0; 1024];
        let desc = self
            .write_description(&mut buf)
            .map(|x| &*x)
            .unwrap_or(b"Unknown error");
        f.debug_struct("Error")
            .field("source", &self.source())
            .field("code", &self.code())
            .field("description", &Escaped(desc))
            .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0; 1024];
        let desc = self
            .write_description(&mut buf)
            .map(|x| &*x)
            .unwrap_or(b"Unknown error");
        write!(fmt, "{} (gpg error {})", Escaped(desc), self.code())
    }
}

impl From<Infallible> for Error {
    #[inline]
    fn from(x: Infallible) -> Self {
        match x {}
    }
}

impl From<NulError> for Error {
    #[inline]
    fn from(_: NulError) -> Self {
        Self::EINVAL
    }
}

impl From<TryFromIntError> for Error {
    #[inline]
    fn from(_: TryFromIntError) -> Self {
        Self::EINVAL
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        let kind = err.kind();
        if let Some(Ok(err)) = err.into_inner().map(|e| e.downcast::<Self>()) {
            *err
        } else {
            match kind {
                ErrorKind::NotFound => Self::ENOENT,
                ErrorKind::PermissionDenied => Self::EACCES,
                ErrorKind::ConnectionRefused => Self::ECONNREFUSED,
                ErrorKind::ConnectionReset => Self::ECONNRESET,
                ErrorKind::ConnectionAborted => Self::ECONNABORTED,
                ErrorKind::NotConnected => Self::ENOTCONN,
                ErrorKind::AddrInUse => Self::EADDRINUSE,
                ErrorKind::AddrNotAvailable => Self::EADDRNOTAVAIL,
                ErrorKind::BrokenPipe => Self::EPIPE,
                ErrorKind::AlreadyExists => Self::EEXIST,
                ErrorKind::WouldBlock => Self::EWOULDBLOCK,
                ErrorKind::InvalidInput => Self::EINVAL,
                ErrorKind::TimedOut => Self::ETIMEDOUT,
                ErrorKind::Interrupted => Self::EINTR,
                _ => Error::EIO,
            }
        }
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        let kind = match err.with_source(Error::SOURCE_UNKNOWN) {
            Error::ECONNREFUSED => ErrorKind::ConnectionRefused,
            Error::ECONNRESET => ErrorKind::ConnectionReset,
            Error::EPERM | Error::EACCES => ErrorKind::PermissionDenied,
            Error::EPIPE => ErrorKind::BrokenPipe,
            Error::ENOTCONN => ErrorKind::NotConnected,
            Error::ECONNABORTED => ErrorKind::ConnectionAborted,
            Error::EADDRNOTAVAIL => ErrorKind::AddrNotAvailable,
            Error::EADDRINUSE => ErrorKind::AddrInUse,
            Error::ENOENT => ErrorKind::NotFound,
            Error::EINTR => ErrorKind::Interrupted,
            Error::EINVAL => ErrorKind::InvalidInput,
            Error::ETIMEDOUT => ErrorKind::TimedOut,
            Error::EEXIST => ErrorKind::AlreadyExists,
            x if x == Error::EAGAIN || x == Error::EWOULDBLOCK => ErrorKind::WouldBlock,
            _ => ErrorKind::Other,
        };
        Self::new(kind, err)
    }
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[macro_export]
macro_rules! return_err {
    ($e:expr) => {
        match $crate::Error::from($e) {
            $crate::Error::NO_ERROR => (),
            err => return Err(From::from(err)),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn test_errno() {
        let e = Error::from_errno(0);
        assert_eq!(e.to_errno(), 0);
        assert_eq!(e.code(), 0);
        assert_eq!(e, Error::NO_ERROR);
    }

    #[test]
    fn test_syserror() {
        unsafe {
            ffi::gpg_err_set_errno(0);
        }
        let e = Error::last_os_error();
        assert_eq!(e.to_errno(), 0);
        assert_eq!(e, Error::MISSING_ERRNO);
    }
}
