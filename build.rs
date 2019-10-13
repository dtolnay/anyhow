use std::env;
use std::process::Command;
use std::str::{self, FromStr};

fn main() {
    let compiler = match rustc_version() {
        Some(compiler) => compiler,
        None => return,
    };

    if compiler.minor >= 40 && compiler.nightly {
        println!("cargo:rustc-cfg=backtrace");
    }
}

struct Compiler {
    minor: u32,
    nightly: bool,
}

fn rustc_version() -> Option<Compiler> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let version = str::from_utf8(&output.stdout).ok()?;

    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }

    let next = pieces.next()?;
    let minor = u32::from_str(next).ok()?;
    let nightly = version.contains("nightly") || version.contains("dev");
    Some(Compiler { minor, nightly })
}
