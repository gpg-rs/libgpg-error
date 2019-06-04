# libgpg-error

[![LGPL-2.1 licensed](https://img.shields.io/crates/l/libgpg-errror.svg)](./COPYING)
[![Crates.io](https://img.shields.io/crates/v/libgpg-error.svg)](https://crates.io/crates/libgpg-error)

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
