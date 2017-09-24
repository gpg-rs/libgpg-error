extern crate gcc;

use std::env;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::str;

#[path = "../build_helper/mod.rs"]
mod build_helper;

use build_helper::*;

fn main() {
    if let Err(_) = configure() {
        process::exit(1);
    }
}

fn configure() -> Result<()> {
    generate_codes();

    println!("cargo:rerun-if-env-changed=GPG_ERROR_LIB_DIR");
    let path = env::var_os("GPG_ERROR_LIB_DIR");
    println!("cargo:rerun-if-env-changed=GPG_ERROR_LIBS");
    let libs = env::var_os("GPG_ERROR_LIBS");
    if path.is_some() || libs.is_some() {
        println!("cargo:rerun-if-env-changed=GPG_ERROR_STATIC");
        let mode = match env::var_os("GPG_ERROR_STATIC") {
            Some(_) => "static",
            _ => "dylib",
        };

        for path in path.iter().flat_map(env::split_paths) {
            println!("cargo:rustc-link-search=native={}", path.display());
        }
        for lib in env::split_paths(libs.as_ref().map(|s| &**s).unwrap_or("gpg-error".as_ref())) {
            println!("cargo:rustc-link-lib={0}={1}", mode, lib.display());
        }
        return Ok(());
    }

    println!("cargo:rerun-if-env-changed=GPG_ERROR_CONFIG");
    if let Some(path) = env::var_os("GPG_ERROR_CONFIG") {
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
    fn each_line<P: AsRef<Path>, F: FnMut(&str)>(path: P, mut f: F) {
        let mut file = BufReader::new(File::open(path).unwrap());
        let mut line = String::new();
        loop {
            line.clear();
            if file.read_line(&mut line).unwrap() == 0 {
                break;
            }
            f(&line);
        }
    }

    let src = PathBuf::from(env::current_dir().unwrap()).join("libgpg-error/src");
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut output = File::create(dst.join("constants.rs")).unwrap();
    fs::copy(src.join("err-sources.h.in"), dst.join("err-sources.h.in")).unwrap();
    each_line(src.join("err-sources.h.in"), |l| {
        if let (Some(code), Some(name)) = scan!(l; u32, String) {
            writeln!(output, "pub const {}: gpg_err_source_t = {};", name, code).unwrap();
        }
    });
    fs::copy(src.join("err-codes.h.in"), dst.join("err-codes.h.in")).unwrap();
    each_line(src.join("err-codes.h.in"), |l| {
        if let (Some(code), Some(name)) = scan!(l; u32, String) {
            writeln!(output, "pub const {}: gpg_err_code_t = {};", name, code).unwrap();
        }
    });
    fs::copy(src.join("errnos.in"), dst.join("errnos.in")).unwrap();
    each_line(src.join("errnos.in"), |l| {
        if let (Some(code), Some(name)) = scan!(l; u32, String) {
            writeln!(
                output,
                "pub const GPG_ERR_{}: gpg_err_code_t = GPG_ERR_SYSTEM_ERROR | {};",
                name,
                code
            ).unwrap();
        }
    });
    println!("cargo:root={}", dst.display());
}

fn try_config<S: Into<OsString>>(path: S) -> Result<()> {
    let path = path.into();
    let mut cmd = path.clone();
    cmd.push(" --mt --libs");
    if let Ok(output) = output(Command::new("sh").arg("-c").arg(cmd)) {
        parse_config_output(&output);
        return Ok(());
    }

    let mut cmd = path;
    cmd.push(" --libs");
    let output = output(Command::new("sh").arg("-c").arg(cmd))?;
    parse_config_output(&output);
    Ok(())
}

fn parse_config_output(output: &str) {
    let parts = output
        .split(|c: char| c.is_whitespace())
        .filter_map(|p| if p.len() > 2 {
            Some(p.split_at(2))
        } else {
            None
        });

    for (flag, val) in parts {
        match flag {
            "-L" => {
                println!("cargo:rustc-link-search=native={}", val);
            }
            "-F" => {
                println!("cargo:rustc-link-search=framework={}", val);
            }
            "-l" => {
                println!("cargo:rustc-link-lib={}", val);
            }
            _ => (),
        }
    }
}

fn try_build() -> Result<()> {
    let config = Config::new("libgpg-error")?;

    if config.target.contains("msvc") {
        return Err(());
    }

    run(
        Command::new("sh")
            .current_dir(&config.src)
            .arg("autogen.sh"),
    )?;
    let mut cmd = config.configure()?;
    cmd.arg("--disable-doc");
    run(&mut cmd)?;
    run(&mut config.make())?;
    run(&mut config.make().arg("install"))?;

    println!(
        "cargo:rustc-link-search=native={}",
        config.dst.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=gpg-error");
    Ok(())
}
