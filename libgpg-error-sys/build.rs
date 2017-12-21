extern crate gcc;

use std::env;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

mod build_helper;

use build_helper::*;

fn main() {
    if let Err(_) = configure() {
        process::exit(1);
    }
}

fn configure() -> Result<()> {
    generate_codes();

    let path = get_env("GPG_ERROR_LIB_DIR");
    let libs = get_env("GPG_ERROR_LIBS");
    if path.is_some() || libs.is_some() {
        let mode = match get_env("GPG_ERROR_STATIC") {
            Some(_) => "static=",
            _ => "",
        };

        for path in path.iter().flat_map(env::split_paths) {
            println!("cargo:rustc-link-search=native={}", path.display());
        }
        for lib in env::split_paths(libs.as_ref().map(|s| &**s).unwrap_or("gpg-error".as_ref())) {
            println!("cargo:rustc-link-lib={}{}", mode, lib.display());
        }
        return Ok(());
    }

    if let Some(path) = get_env("GPG_ERROR_CONFIG") {
        return try_config(path);
    }

    if !Path::new("libgpg-error/autogen.sh").exists() {
        let _ = run(Command::new("git").args(&["submodule", "update", "--init"]));
    }

    try_build().or_else(|_| try_config("gpg-error-config"))
}

macro_rules! scan {
    ($string:expr, $sep:expr; $($x:ty),+) => ({
        let mut iter = $string.split($sep);
        ($(iter.next().and_then(|word| word.parse::<$x>().ok()),)*)
    });
    ($string:expr; $($x:ty),+) => (
        scan!($string, char::is_whitespace; $($x),+)
    );
}

fn generate_codes() {
    let src = PathBuf::from(env::current_dir().unwrap()).join("libgpg-error/src");
    let dst = out_dir();
    let mut output = File::create(dst.join("constants.rs")).unwrap();
    fs::copy(src.join("err-sources.h.in"), dst.join("err-sources.h.in")).unwrap();
    for_each_line(src.join("err-sources.h.in"), |l| {
        if let (Some(code), Some(name)) = scan!(l; u32, String) {
            writeln!(output, "pub const {}: gpg_err_source_t = {};", name, code).unwrap();
        }
    }).unwrap();
    fs::copy(src.join("err-codes.h.in"), dst.join("err-codes.h.in")).unwrap();
    for_each_line(src.join("err-codes.h.in"), |l| {
        if let (Some(code), Some(name)) = scan!(l; u32, String) {
            writeln!(output, "pub const {}: gpg_err_code_t = {};", name, code).unwrap();
        }
    }).unwrap();
    fs::copy(src.join("errnos.in"), dst.join("errnos.in")).unwrap();
    for_each_line(src.join("errnos.in"), |l| {
        if let (Some(code), Some(name)) = scan!(l; u32, String) {
            writeln!(
                output,
                "pub const GPG_ERR_{}: gpg_err_code_t = GPG_ERR_SYSTEM_ERROR | {};",
                name, code
            ).unwrap();
        }
    }).unwrap();
    println!("cargo:gen={}", dst.display());
}

fn try_config<S: Into<OsString>>(path: S) -> Result<()> {
    let path = path.into();
    let mut cmd = path.clone();
    cmd.push(" --mt --libs");
    if let Ok(output) = output(Command::new("sh").arg("-c").arg(cmd)) {
        parse_linker_flags(&output);
        return Ok(());
    }

    let mut cmd = path;
    cmd.push(" --libs");
    let output = output(Command::new("sh").arg("-c").arg(cmd))?;
    parse_linker_flags(&output);
    Ok(())
}

fn try_build() -> Result<()> {
    if target().contains("msvc") {
        return Err(());
    }

    let config = Config::new("libgpg-error")?;
    run(Command::new("sh")
        .current_dir(&config.src)
        .arg("autogen.sh"))?;
    let mut cmd = config.configure()?;
    cmd.arg("--disable-doc");
    run(cmd)?;
    run(config.make())?;
    run(config.make().arg("install"))?;

    parse_libtool_file(config.dst.join("lib/libgpg-error.la"))?;
    println!(
        "cargo:rustc-link-search={}",
        config.dst.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=gpg-error");
    println!("cargo:root={}", config.dst.display());
    Ok(())
}
