# libgpg-error

[![Build Status][build]][ci]
[![crates.io version][version]][crate]
[![LGPL-2.1 licensed][license]](./COPYING)
[![downloads][downloads]][crate]

libgpg-error bindings for Rust.

## Building
These crates require the libgpg-error library and its development files (e.g.,
headers, gpg-error-config) to be installed. The buildscript will attempt to
detect the necessary information using the `gpg-error-config` script
distributed with libgpg-error. If for whatever reason this does not work, the
required information can also be specified using one or more environment variables:
- `LIBGPG_ERROR_INCLUDE` specifies the path(s) where header files can be found.
- `LIBGPG_ERROR_LIB_DIR` specifies the path(s) where library files (e.g., *.so,
  *.a, *.dll, etc.) can be found.
- `LIBGPG_ERROR_LIBS` specifies the name(s) of all required libraries.
- `LIBGPG_ERROR_STATIC` controls whether libraries are linked to
  statically or dynamically by default. Individual libraries can have their
  linkage overridden by prefixing their names with either `static=` or
  `dynamic=` in `LIBGPG_ERROR_LIBS`.
- `LIBGPG_ERROR_CONFIG` specifies the path to the `gpg-error-config` script.

Each environment variable, with the exceptions of `LIBGPG_ERROR_STATIC` and
`LIBGPG_ERROR_CONFIG`, can take multiple values separated by the platform's path
separator.

**NOTE**: Previous versions of these crates bundled the sources of the libgpg-error library and attempted
to build them via the buildscript. This is no longer supported.

[crate]: https://crates.io/crates/gpg-error
[ci]: https://travis-ci.org/gpg-rs/libgpg-error
[build]: https://img.shields.io/travis/gpg-rs/libgpg-error/master?style=flat-square
[version]: https://img.shields.io/crates/v/gpg-error?style=flat-square
[license]: https://img.shields.io/crates/l/gpg-error?style=flat-square
[downloads]: https://img.shields.io/crates/d/gpg-error?style=flat-square
