use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::str;

macro_rules! scan {
    ($string:expr, $sep:expr; $($x:ty),+) => ({
        let mut iter = $string.split($sep);
        ($(iter.next().and_then(|word| word.parse::<$x>().ok()),)*)
    });
    ($string:expr; $($x:ty),+) => (
        scan!($string, char::is_whitespace; $($x),+)
    );
}

fn main() {
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

    let src = PathBuf::from(env::var_os("DEP_GPG_ERROR_GEN").unwrap());
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut output = File::create(dst.join("constants.rs")).unwrap();
    writeln!(output, "impl Error {{").unwrap();
    each_line(src.join("err-sources.h.in"), |l| {
        if let (Some(_), Some(name)) = scan!(l; u32, String) {
            writeln!(
                output,
                "pub const {}: ErrorSource = ffi::{};",
                name.trim_left_matches("GPG_ERR_"),
                name
            ).unwrap();
        }
    });
    each_line(src.join("err-codes.h.in"), |l| {
        if let (Some(_), Some(name)) = scan!(l; u32, String) {
            writeln!(
                output,
                "pub const {}: Error = Error(ffi::{});",
                name.trim_left_matches("GPG_ERR_"),
                name
            ).unwrap();
        }
    });
    each_line(src.join("errnos.in"), |l| {
        if let (Some(_), Some(name)) = scan!(l; u32, String) {
            writeln!(
                output,
                "pub const {}: Error = Error(ffi::GPG_ERR_{});",
                name, name
            ).unwrap();
        }
    });
    writeln!(output, "}}").unwrap();
}
