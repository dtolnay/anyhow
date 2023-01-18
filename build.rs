#![allow(clippy::option_if_let_else)]

use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::str;

#[cfg(all(feature = "backtrace", not(feature = "std")))]
compile_error! {
    "`backtrace` feature without `std` feature is not supported"
}

// This code exercises the surface area that we expect of the std Backtrace
// type. If the current toolchain is able to compile it, we go ahead and use
// backtrace in anyhow.
const PROBE: &str = r#"
    #![feature(error_generic_member_access, provide_any)]

    use std::any::{Demand, Provider};
    use std::backtrace::{Backtrace, BacktraceStatus};
    use std::error::Error;
    use std::fmt::{self, Display};

    #[derive(Debug)]
    struct E {
        backtrace: Backtrace,
    }

    impl Display for E {
        fn fmt(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }

    impl Error for E {
        fn provide<'a>(&'a self, demand: &mut Demand<'a>) {
            demand.provide_ref(&self.backtrace);
        }
    }

    struct P;

    impl Provider for P {
        fn provide<'a>(&'a self, _demand: &mut Demand<'a>) {}
    }

    const _: fn() = || {
        let backtrace: Backtrace = Backtrace::capture();
        let status: BacktraceStatus = backtrace.status();
        match status {
            BacktraceStatus::Captured | BacktraceStatus::Disabled | _ => {}
        }
    };

    const _: fn(&dyn Error) -> Option<&Backtrace> = |err| err.request_ref::<Backtrace>();
"#;

fn main() {
    // If the compile probe succeeds, i.e. if a nightly compiler is used and the
    // provide API exists, then `std::backtrace` is also usable. So `cfg
    // (provide_api)` implies `cfg(backtrace)`. We do set `cfg(backtrace)` in
    // this script explicitly anyway to make the `cfg()` attributes in the main
    // code less complex.
    let mut nightly_backtrace_support = false;
    if cfg!(feature = "std") {
        match compile_probe() {
            Some(status) if status.success() => {
                println!("cargo:rustc-cfg=provide_api");
                nightly_backtrace_support = true;
            }
            _ => {}
        }
    }

    let rustc = match rustc_minor_version() {
        Some(rustc) => rustc,
        None => return,
    };

    if rustc < 51 {
        println!("cargo:rustc-cfg=anyhow_no_ptr_addr_of");
    }

    if rustc < 52 {
        println!("cargo:rustc-cfg=anyhow_no_fmt_arguments_as_str");
    }

    if nightly_backtrace_support || (cfg!(feature = "std") && rustc >= 65) {
        println!("cargo:rustc-cfg=backtrace");
    }
}

fn compile_probe() -> Option<ExitStatus> {
    let rustc = env::var_os("RUSTC")?;
    let out_dir = env::var_os("OUT_DIR")?;
    let probefile = Path::new(&out_dir).join("probe.rs");
    fs::write(&probefile, PROBE).ok()?;

    // Make sure to pick up Cargo rustc configuration.
    let mut cmd = if let Some(wrapper) = env::var_os("RUSTC_WRAPPER") {
        let mut cmd = Command::new(wrapper);
        // The wrapper's first argument is supposed to be the path to rustc.
        cmd.arg(rustc);
        cmd
    } else {
        Command::new(rustc)
    };

    cmd.stderr(Stdio::null())
        .arg("--edition=2018")
        .arg("--crate-name=anyhow_build")
        .arg("--crate-type=lib")
        .arg("--emit=metadata")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(probefile);

    if let Some(target) = env::var_os("TARGET") {
        cmd.arg("--target").arg(target);
    }

    // If Cargo wants to set RUSTFLAGS, use that.
    if let Ok(rustflags) = env::var("CARGO_ENCODED_RUSTFLAGS") {
        if !rustflags.is_empty() {
            for arg in rustflags.split('\x1f') {
                cmd.arg(arg);
            }
        }
    }

    cmd.status().ok()
}

fn rustc_minor_version() -> Option<u32> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let version = str::from_utf8(&output.stdout).ok()?;
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    pieces.next()?.parse().ok()
}
