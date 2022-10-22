# libgpg-error

[![Build Status][build]][ci]
[![crates.io version][version]][crate]
[![LGPL-2.1 licensed][license]](./COPYING)
[![downloads][downloads]][crate]

libgpg-error bindings for Rust.

## Using

To use the crate, add it to your depedencies:
```sh
$ cargo add libgpg-error
```

### Requirements
These crates require the libgpg-error library and its development files to be
installed. The build script uses the [system-deps] crate to attempt to locate
them (or the registry on Windows).

On Debian/Ubuntu based systems:
```sh
$ sudo apt-get install libgpg-error-dev
```

On Fedora/RHEL based systems:
```sh
$ sudo dnf install libgpg-error-devel
```

On MacOS systems:
```sh
$ brew install gnupg
```

On Windows systems, download and install the official [Gpg4win] installer. Only
the `i686-pc-windows-gnu` target is supported.

## License
The `libgpg-error` and `libgpg-error-sys` crates are licensed under the [LGPL-2.1 license](./COPYING). Files under
upstream are part of libgpg-error and are licensed under LGPL-2.1-or-later.

[crate]: https://crates.io/crates/gpg-error
[ci]: https://github.com/gpg-rs/libgpg-error/workflows/ci
[build]: https://img.shields.io/github/workflow/status/gpg-rs/libgpg-error/ci?style=flat-square
[version]: https://img.shields.io/crates/v/gpg-error?style=flat-square
[license]: https://img.shields.io/crates/l/gpg-error?style=flat-square
[downloads]: https://img.shields.io/crates/d/gpg-error?style=flat-square
[system-deps]: https://crates.io/crates/system-deps
[Gpg4win]: https://www.gpg4win.org/
