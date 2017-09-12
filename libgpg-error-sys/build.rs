extern crate gcc;

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::result;
use std::str;

type Result<T> = result::Result<T, ()>;

fn main() {
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
        return;
    }

    println!("cargo:rerun-if-env-changed=GPG_ERROR_CONFIG");
    if let Some(path) = env::var_os("GPG_ERROR_CONFIG") {
        if try_config(path).is_err() {
            process::exit(1);
        }
        return;
    }

    if !Path::new("libgpg-error/autogen.sh").exists() {
        let _ = run(Command::new("git").args(&["submodule", "update", "--init"]));
    }

    if try_build().is_ok() || try_config("gpg-error-config").is_ok() {
        return;
    }
    process::exit(1);
}

fn try_config<S: Into<OsString>>(path: S) -> Result<()> {
    let path = path.into();
    let mut cmd = path.clone();
    cmd.push(" --prefix");
    if let Ok(output) = output(Command::new("sh").arg("-c").arg(cmd)) {
        println!("cargo:root={}", output);
    }

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
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();
    let src = PathBuf::from(env::current_dir().unwrap()).join("libgpg-error");
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let build = dst.clone().join("build");
    let compiler = gcc::Build::new().get_compiler();
    let cflags = compiler.args().iter().fold(OsString::new(), |mut c, a| {
        c.push(a);
        c.push(" ");
        c
    });

    if target.contains("msvc") {
        return Err(());
    }

    fs::create_dir_all(&build).map_err(|e| eprintln!("unable to create build directory: {}", e))?;

    run(Command::new("sh").current_dir(&src).arg("autogen.sh"))?;
    run(
        Command::new("sh")
            .current_dir(&build)
            .env("CC", msys_compatible(compiler.path())?)
            .env("CFLAGS", cflags)
            .arg(msys_compatible(src.join("configure"))?)
            .args(&[
                "--build",
                &gnu_target(&host),
                "--host",
                &gnu_target(&target),
                "--enable-static",
                "--disable-shared",
                "--disable-doc",
                "--prefix",
            ])
            .arg(msys_compatible(&dst)?),
    )?;
    run(
        make()
            .current_dir(&build)
            .arg("-j")
            .arg(env::var("NUM_JOBS").unwrap()),
    )?;
    run(Command::new("make").current_dir(&build).arg("install"))?;

    println!("cargo:root={}", dst.display());
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=gpg-error");
    Ok(())
}

fn make() -> Command {
    let name = if cfg!(any(target_os = "freebsd", target_os = "dragonfly")) {
        "gmake"
    } else {
        "make"
    };
    let mut cmd = Command::new(name);
    cmd.env_remove("DESTDIR");
    if cfg!(windows) {
        cmd.env_remove("MAKEFLAGS").env_remove("MFLAGS");
    }
    cmd
}

fn msys_compatible<P: AsRef<OsStr>>(path: P) -> Result<OsString> {
    use std::ascii::AsciiExt;

    if !cfg!(windows) || Path::new(path.as_ref()).is_relative() {
        return Ok(path.as_ref().to_owned());
    }

    let mut path = path.as_ref()
        .to_str()
        .ok_or_else(|| {
            eprintln!("path is not valid utf-8");
        })?
        .to_owned();
    if let Some(b'a'...b'z') = path.as_bytes().first().map(u8::to_ascii_lowercase) {
        if path.split_at(1).1.starts_with(":\\") {
            (&mut path[..1]).make_ascii_lowercase();
            path.remove(1);
            path.insert(0, '/');
        }
    }
    Ok(path.replace("\\", "/").into())
}

fn gnu_target(target: &str) -> String {
    match target {
        "i686-pc-windows-gnu" => "i686-w64-mingw32".to_string(),
        "x86_64-pc-windows-gnu" => "x86_64-w64-mingw32".to_string(),
        s => s.to_string(),
    }
}

fn run(cmd: &mut Command) -> Result<String> {
    eprintln!("running: {:?}", cmd);
    match cmd.stdin(Stdio::null())
        .spawn()
        .and_then(|c| c.wait_with_output()) {
        Ok(output) => if output.status.success() {
            String::from_utf8(output.stdout).or(Err(()))
        } else {
            eprintln!(
                "command did not execute successfully, got: {}",
                output.status
            );
            Err(())
        },
        Err(e) => {
            eprintln!("failed to execute command: {}", e);
            Err(())
        }
    }
}

fn output(cmd: &mut Command) -> Result<String> {
    run(cmd.stdout(Stdio::piped()))
}
