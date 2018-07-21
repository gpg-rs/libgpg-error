extern crate ctest;

use std::env;

fn main() {
    let mut cfg = ctest::TestGenerator::new();
    if let Some(paths) = env::var_os("DEP_GPG_ERROR_INCLUDE") {
        for p in env::split_paths(&paths) {
            cfg.include(p);
        }
    }
    cfg.header("gpg-error.h");
    cfg.cfg("ctest", None);
    cfg.generate("../libgpg-error-sys/lib.rs", "all.rs");
}
