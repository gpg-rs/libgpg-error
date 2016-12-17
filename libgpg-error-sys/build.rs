use std::env;
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
        if let Some(output) = output(Command::new(&path).arg("--prefix"), false) {
            println!("cargo:root={}", output);
            return;
        }

        if let Some(output) = output(Command::new(&path).args(&["--mt", "--libs"]), false) {
            parse_config_output(&output);
            return;
        }

        if let Some(output) = output(Command::new(&path).arg("--libs"), true) {
            parse_config_output(&output);
            return;
        }
    }

    if !Path::new("libgpg-error/.git").exists() {
        run(Command::new("git").args(&["submodule", "update", "--init"]),
            false);
    }

    build();
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

fn build() {
    let dst = env::var("OUT_DIR").unwrap();
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();

    if !Path::new("libgpg-error/configure").exists() {
        run(Command::new("sh").current_dir("libgpg-error").arg("autogen.sh"),
            true);
    }
    if !Path::new("libgpg-error/Makefile").exists() {
        run(Command::new("sh")
                .current_dir("libgpg-error")
                .args(&["configure",
                        "--enable-maintainer-mode",
                        "--build", &host,
                        "--host", &target,
                        "--enable-static",
                        "--disable-shared",
                        "--with-pic",
                        "--prefix", &dst]),
            true);
    }
    run(Command::new("make")
            .current_dir("libgpg-error")
            .arg("-j").arg(env::var("NUM_JOBS").unwrap()),
        true);
    run(Command::new("make")
            .current_dir("libgpg-error")
            .arg("install"),
        true);
    println!("cargo:root={}", &dst);
    println!("cargo:rustc-link-lib=static=gpg-error");
    println!("cargo:rustc-link-search=native={}",
             PathBuf::from(dst).join("lib").display());
}

fn spawn(cmd: &mut Command, abort: bool) -> Option<Child> {
    println!("running: {:?}", cmd);
    match cmd.stdin(Stdio::null()).spawn() {
        Ok(child) => Some(child),
        Err(e) => {
            println!("failed to execute command: {:?}\nerror: {}", cmd, e);
            if abort {
                process::exit(1);
            }
            None
        }
    }
}

fn run(cmd: &mut Command, abort: bool) {
    if let Some(mut child) = spawn(cmd, abort) {
        match child.wait() {
            Ok(status) => {
                if !status.success() {
                    println!("command did not execute successfully: {:?}\n\
                       expected success, got: {}", cmd, status);
                    if abort {
                        process::exit(1);
                    }
                }
            }
            Err(e) => {
                println!("failed to execute command: {:?}\nerror: {}", cmd, e);
                if abort {
                    process::exit(1);
                }
            }
        }
    }
}

fn output(cmd: &mut Command, abort: bool) -> Option<String> {
    if let Some(child) = spawn(cmd.stdout(Stdio::piped()), abort) {
        match child.wait_with_output() {
            Ok(output) => {
                if !output.status.success() {
                    println!("command did not execute successfully: {:?}\n\
                       expected success, got: {}", cmd, output.status);
                    if abort {
                        process::exit(1);
                    }
                }
                return String::from_utf8(output.stdout).ok();
            }
            Err(e) => {
                println!("failed to execute command: {:?}\nerror: {}", cmd, e);
                if abort {
                    process::exit(1);
                }
            }
        }
    }
    None
}
