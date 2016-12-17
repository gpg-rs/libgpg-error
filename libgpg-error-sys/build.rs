extern crate gcc;

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Child, Stdio};
use std::str;

fn main() {
    if let Ok(lib) = env::var("LIBGPG_ERROR_LIB") {
        let mode = match env::var_os("LIBGPG_ERROR_STATIC") {
            Some(_) => "static",
            None => "dylib",
        };
        println!("cargo:rustc-link-lib={0}={1}", mode, lib);
        return;
    } else if let Some(path) = env::var_os("GPG_ERROR_CONFIG") {
        if !try_config(path) {
            process::exit(1);
        }
        return;
    }

    if !Path::new("libgpg-error/.git").exists() {
        run(Command::new("git").args(&["submodule", "update", "--init"]));
    }

    if try_build() || try_config("gpg-error-config") {
        return;
    }
    process::exit(1);
}

fn try_config<S: AsRef<OsStr>>(path: S) -> bool {
    let path = path.as_ref();
    if let Some(output) = output(Command::new(&path).arg("--prefix")) {
        println!("cargo:root={}", output);
    }

    if let Some(output) = output(Command::new(&path).args(&["--mt", "--libs"])) {
        parse_config_output(&output);
        return true;
    }

    if let Some(output) = output(Command::new(&path).arg("--libs")) {
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
    let dst = env::var("OUT_DIR").unwrap();
    let build = PathBuf::from(&dst).join("build");
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
        .arg(src.join("configure"))
        .args(&["--enable-maintainer-mode",
                "--build", &host,
                "--host", &target,
                "--enable-static",
                "--disable-shared",
                "--with-pic",
                "--prefix", &dst])) {
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
    println!("cargo:root={}", &dst);
    println!("cargo:rustc-link-lib=static=gpg-error");
    println!("cargo:rustc-link-search=native={}",
             PathBuf::from(dst).join("lib").display());
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
