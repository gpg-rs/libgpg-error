extern crate gcc;

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Child, Stdio};
use std::str;

fn main() {
    let path = env::var_os("GPG_ERROR_LIB_PATH");
    let libs = env::var_os("GPG_ERROR_LIBS");
    if path.is_some() || libs.is_some() {
        let mode = match env::var_os("GPG_ERROR_STATIC") {
            Some(_) => "static",
            _ => "dylib",
        };

        for path in path.iter().flat_map(env::split_paths) {
            println!("cargo:rustc-link-search=native={}", path.display());
        }
        match libs {
            Some(libs) => {
                for lib in env::split_paths(&libs) {
                    println!("cargo:rustc-link-lib={0}={1}", mode, lib.display());
                }
            }
            None => {
                println!("cargo:rustc-link-lib={0}={1}", mode, "gpg-error");
            }
        }
        return;
    } else if let Some(path) = env::var_os("GPG_ERROR_CONFIG") {
        if !try_config(path) {
            process::exit(1);
        }
        return;
    }

    if !Path::new("libgpg-error/autogen.sh").exists() {
        run(Command::new("git").args(&["submodule", "update", "--init"]));
    }

    if try_build() || try_config("gpg-error-config") {
        return;
    }
    process::exit(1);
}

fn try_config<S: AsRef<OsStr>>(path: S) -> bool {
    let path = path.as_ref();

    let mut cmd = path.to_owned();
    cmd.push(" --prefix");
    if let Some(output) = output(Command::new("sh").arg("-c").arg(cmd)) {
        println!("cargo:root={}", output);
    }

    let mut cmd = path.to_owned();
    cmd.push(" --mt --libs");
    if let Some(output) = output(Command::new("sh").arg("-c").arg(cmd)) {
        parse_config_output(&output);
        return true;
    }

    let mut cmd = path.to_owned();
    cmd.push(" --libs");
    if let Some(output) = output(Command::new("sh").arg("-c").arg(cmd)) {
        parse_config_output(&output);
        return true;
    }
    false
}

fn parse_config_output(output: &str) {
    let parts = output.split(|c: char| c.is_whitespace()).filter_map(|p| if p.len() > 2 {
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

fn try_build() -> bool {
    let src = PathBuf::from(env::current_dir().unwrap()).join("libgpg-error");
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let build = dst.clone().join("build");
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();
    let compiler = gcc::Config::new().get_compiler();
    let cflags = compiler.args().iter().fold(OsString::new(), |mut c, a| {
        c.push(a);
        c.push(" ");
        c
    });

    let _ = fs::create_dir_all(&build);

    if !run(Command::new("sh").current_dir(&src).arg("autogen.sh")) {
        return false;
    }
    if !run(Command::new("sh")
        .current_dir(&build)
        .env("CC", compiler.path())
        .env("CFLAGS", cflags)
        .arg(msys_compatible(src.join("configure")))
        .args(&["--build", &gnu_target(&host),
                "--host", &gnu_target(&target),
                "--enable-static",
                "--disable-shared",
                "--disable-doc",
                "--prefix", &msys_compatible(&dst)])) {
        return false;
    }
    if !run(Command::new("make")
        .current_dir(&build)
        .arg("-j")
        .arg(env::var("NUM_JOBS").unwrap())) {
        return false;
    }
    if !run(Command::new("make")
        .current_dir(&build)
        .arg("install")) {
        return false;
    }

    println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
    println!("cargo:rustc-link-lib=static=gpg-error");
    println!("cargo:root={}", dst.display());
    true
}

fn spawn(cmd: &mut Command) -> Option<Child> {
    println!("running: {:?}", cmd);
    match cmd.stdin(Stdio::null()).spawn() {
        Ok(child) => Some(child),
        Err(e) => {
            println!("failed to execute command: {:?}\nerror: {}", cmd, e);
            None
        }
    }
}

fn run(cmd: &mut Command) -> bool {
    if let Some(mut child) = spawn(cmd) {
        match child.wait() {
            Ok(status) => {
                if !status.success() {
                    println!("command did not execute successfully: {:?}\n\
                       expected success, got: {}", cmd, status);
                } else {
                    return true;
                }
            }
            Err(e) => {
                println!("failed to execute command: {:?}\nerror: {}", cmd, e);
            }
        }
    }
    false
}

fn output(cmd: &mut Command) -> Option<String> {
    if let Some(child) = spawn(cmd.stdout(Stdio::piped())) {
        match child.wait_with_output() {
            Ok(output) => {
                if !output.status.success() {
                    println!("command did not execute successfully: {:?}\n\
                       expected success, got: {}", cmd, output.status);
                } else {
                    return String::from_utf8(output.stdout).ok();
                }
            }
            Err(e) => {
                println!("failed to execute command: {:?}\nerror: {}", cmd, e);
            }
        }
    }
    None
}

fn msys_compatible<P: AsRef<Path>>(path: P) -> String {
    use std::ascii::AsciiExt;

    let mut path = path.as_ref().to_string_lossy().into_owned();
    if !cfg!(windows) || Path::new(&path).is_relative() {
        return path;
    }

    if let Some(b'a'...b'z') = path.as_bytes().first().map(u8::to_ascii_lowercase) {
        if path.split_at(1).1.starts_with(":\\") {
            (&mut path[..1]).make_ascii_lowercase();
            path.remove(1);
            path.insert(0, '/');
        }
    }
    path.replace("\\", "/")
}

fn gnu_target(target: &str) -> String {
    match target {
        "i686-pc-windows-gnu" => "i686-w64-mingw32".to_string(),
        "x86_64-pc-windows-gnu" => "x86_64-w64-mingw32".to_string(),
        s => s.to_string(),
    }
}
