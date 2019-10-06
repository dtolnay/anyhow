use std::env;
use std::process::Command;
use std::str;

fn main() {
    let compiler = match rustc_version() {
        Some(compiler) => compiler,
        None => return,
    };

    if compiler.nightly {
        println!("cargo:rustc-cfg=backtrace");
    }
}

struct Compiler {
    nightly: bool,
}

fn rustc_version() -> Option<Compiler> {
    let rustc = match env::var_os("RUSTC") {
        Some(rustc) => rustc,
        None => return None,
    };

    let output = match Command::new(rustc).arg("--version").output() {
        Ok(output) => output,
        Err(_) => return None,
    };

    let version = match str::from_utf8(&output.stdout) {
        Ok(version) => version,
        Err(_) => return None,
    };

    Some(Compiler {
        nightly: version.contains("nightly") || version.contains("dev"),
    })
}
