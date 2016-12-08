use std::env;
use std::process::Command;
use std::str;

fn parse_config_output(output: &str) {
    let parts = output.split(|c: char| c.is_whitespace()).filter_map(|p| {
        if p.len() > 2 {
            Some(p.split_at(2))
        } else {
            None
        }
    });

    for (flag, val) in parts {
        match flag {
            "-L" => {
                println!("cargo:rustc-link-search=native={}", val);
            },
            "-F" => {
                println!("cargo:rustc-link-search=framework={}", val);
            },
            "-l" => {
                println!("cargo:rustc-link-lib={}", val);
            },
            _ => ()
        }
    }
}

fn main() {
    if let Ok(lib) = env::var("LIBGPG_ERROR_LIB") {
        let mode = match env::var_os("LIBGPG_ERROR_STATIC") {
            Some(_) => "static",
            None => "dylib",
        };
        println!("cargo:rustc-flags=-l {0}={1}", mode, lib);
    } else {
        let mut command = Command::new(env::var_os("GPG_ERROR_CONFIG")
                .unwrap_or("gpg-error-config".into()));
        command.args(&["--mt", "--libs"]);

        let output = command.output().unwrap();
        if !output.status.success() {
            panic!("`{:?}` did not exit successfully: {}", command, output.status);
        }
        parse_config_output(&str::from_utf8(&output.stdout).unwrap());
    }
}

