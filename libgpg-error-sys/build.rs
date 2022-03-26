use std::{
    env,
    ffi::OsString,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::Command,
};

#[macro_use]
mod build_helper;

use self::build_helper::*;

fn main() -> Result<()> {
    fn configure() -> Result<Config> {
        let proj = Project::default();
        generate_codes(&proj);

        if let r @ Ok(_) = proj.try_env() {
            return r;
        }

        if let Some(path) = get_env(proj.prefix.clone() + "_CONFIG") {
            return try_config(&proj, path);
        }

        if let r @ Ok(_) = try_registry(&proj) {
            return r;
        }

        try_config(&proj, "gpg-error-config")
    }
    let mut config = configure()?;
    if config.version.is_none() {
        config.try_detect_version("gpg-error.h", "GPG_ERROR_VERSION")?;
    }
    config.write_version_macro("gpg_err");
    config.print();
    Ok(())
}

fn generate_codes(proj: &Project) {
    fn for_each_line(path: impl AsRef<Path>, mut f: impl FnMut(&str)) {
        let path = path.as_ref();
        println!("scanning file: {}", path.display());
        let mut file = File::open(path)
            .map(BufReader::new)
            .expect("failed to open file");
        let mut line = String::new();
        loop {
            line.clear();
            if file.read_line(&mut line).expect("failed to read file") == 0 {
                return;
            }
            f(&line);
        }
    }

    let src = PathBuf::from(env::current_dir().unwrap());
    let dst = &proj.out_dir;
    let mut output = File::create(dst.join("constants.rs")).unwrap();
    fs::copy(src.join("err-sources.h.in"), dst.join("err-sources.h.in")).unwrap();
    for_each_line(src.join("err-sources.h.in"), |l| {
        if let (Some(code), Some(name)) = scan!(l; u32, String) {
            writeln!(output, "pub const {}: gpg_err_source_t = {};", name, code).unwrap();
        }
    });
    fs::copy(src.join("err-codes.h.in"), dst.join("err-codes.h.in")).unwrap();
    for_each_line(src.join("err-codes.h.in"), |l| {
        if let (Some(code), Some(name)) = scan!(l; u32, String) {
            writeln!(output, "pub const {}: gpg_err_code_t = {};", name, code).unwrap();
        }
    });
    fs::copy(src.join("errnos.in"), dst.join("errnos.in")).unwrap();
    for_each_line(src.join("errnos.in"), |l| {
        if let (Some(code), Some(name)) = scan!(l; u32, String) {
            writeln!(
                output,
                "pub const GPG_ERR_{}: gpg_err_code_t = GPG_ERR_SYSTEM_ERROR | {};",
                name, code
            )
            .unwrap();
        }
    });
    println!("cargo:generated={}", dst.display());
}

fn try_config<S: Into<OsString>>(proj: &Project, path: S) -> Result<Config> {
    let path = path.into();
    let mut cmd = path.clone();
    cmd.push(" --version");
    let version = output(Command::new("sh").arg("-c").arg(cmd))?;

    let mut cmd = path.clone();
    cmd.push(" --cflags --mt --libs");
    if let Ok(mut cfg) = proj.try_config(Command::new("sh").arg("-c").arg(cmd)) {
        cfg.version = Some(version.trim().into());
        return Ok(cfg);
    }

    let mut cmd = path;
    cmd.push(" --cflags --libs");
    proj.try_config(Command::new("sh").arg("-c").arg(cmd))
        .map(|mut cfg| {
            cfg.version = Some(version.trim().into());
            cfg
        })
}

#[cfg(not(windows))]
fn try_registry(_: &Project) -> Result<Config> {
    Err(())
}

#[cfg(windows)]
fn try_registry(proj: &Project) -> Result<Config> {
    use winreg::{enums::*, RegKey};

    if !proj.target.contains("windows") {
        eprintln!("cross compiling. disabling registry detection.");
        return Err(());
    }

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = match hklm.open_subkey("SOFTWARE\\GnuPG") {
        Ok(k) => k,
        Err(_) => {
            //check if we are on 64bit windows but have 32bit GnuPG installed
            match hklm.open_subkey("SOFTWARE\\WOW6432Node\\GnuPG") {
                Ok(k) => {
                    // found 32bit library
                    if !proj.target.contains("i586") && !proj.target.contains("i686") {
                        eprintln!("Compile using i586/686 target.");
                        return Err(());
                    } else {
                        k
                    }
                }
                Err(_) => {
                    eprintln!("unable to retrieve install location");
                    return Err(());
                }
            }
        }
    };
    let root = PathBuf::from(
        key.get_value::<String, _>("Install Directory")
           .warn_err("unable to retrieve install location")?,
    );
    println!("detected install via registry: {}", root.display());
    if root.join("lib/libgpg-error.imp").exists() {
        fs::copy(
            root.join("lib/libgpg-error.imp"),
            proj.out_dir.join("libgpg-error.a"),
        )
        .warn_err("unable to rename library")?;
    }

    let mut config = Config::default();
    config.include_dir.insert(root.join("include"));
    config.lib_dir.insert(proj.out_dir.clone());
    config.libs.insert(proj.links.clone().into());
    config.prefix = Some(root);
    Ok(config)
}
