extern crate cc;

use std::{
    env,
    ffi::OsString,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process::Command,
};

#[macro_use]
mod build_helper;

use self::build_helper::*;

fn main() {
    Project::default().configure(|proj| {
        generate_codes(&proj);

        if let r @ Ok(_) = proj.try_env() {
            return r;
        }

        if let Some(path) = get_env(proj.prefix.clone() + "_CONFIG") {
            return try_config(&proj, path);
        }

        if let r @ Ok(_) = proj.try_build(build) {
            return r;
        }

        try_config(&proj, "gpg-error-config")
    });
}

fn generate_codes(proj: &Project) {
    let src = PathBuf::from(env::current_dir().unwrap()).join("libgpg-error/src");
    let dst = &proj.out_dir;
    let mut output = File::create(dst.join("constants.rs")).unwrap();
    fs::copy(src.join("err-sources.h.in"), dst.join("err-sources.h.in")).unwrap();
    for_each_line(src.join("err-sources.h.in"), |l| {
        if let (Some(code), Some(name)) = scan!(l; u32, String) {
            writeln!(output, "pub const {}: gpg_err_source_t = {};", name, code).unwrap();
        }
        Ok(())
    })
    .unwrap();
    fs::copy(src.join("err-codes.h.in"), dst.join("err-codes.h.in")).unwrap();
    for_each_line(src.join("err-codes.h.in"), |l| {
        if let (Some(code), Some(name)) = scan!(l; u32, String) {
            writeln!(output, "pub const {}: gpg_err_code_t = {};", name, code).unwrap();
        }
        Ok(())
    })
    .unwrap();
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
        Ok(())
    })
    .unwrap();
    println!("cargo:generated={}", dst.display());
}

fn try_config<S: Into<OsString>>(proj: &Project, path: S) -> Result<Config> {
    let path = path.into();
    let mut cmd = path.clone();
    cmd.push(" --cflags --mt --libs");
    if let r @ Ok(_) = proj.try_config(Command::new("sh").arg("-c").arg(cmd)) {
        return r;
    }

    let mut cmd = path;
    cmd.push(" --cflags --libs");
    proj.try_config(Command::new("sh").arg("-c").arg(cmd))
}

fn build(proj: &Project) -> Result<Config> {
    if proj.target.contains("msvc") {
        return Err(());
    }

    let build = proj.new_build("libgpg-error")?;
    run(Command::new("sh").current_dir(&build.src).arg("autogen.sh"))?;
    let mut cmd = build.configure_cmd()?;
    cmd.arg("--disable-doc");
    cmd.arg("--disable-languages");
    cmd.arg("--disable-tests");
    run(cmd)?;
    run(build.make_cmd())?;
    run(build.make_cmd().arg("install"))?;

    let mut config = build.config();
    config.parse_libtool_file(proj.out_dir.join("lib/libgpg-error.la"))?;
    Ok(config)
}
