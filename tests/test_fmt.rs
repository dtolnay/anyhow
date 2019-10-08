use anyhow::{bail, Context, Result};

fn f() -> Result<()> {
    bail!("oh no!");
}

fn g() -> Result<()> {
    f().context("f failed")
}

fn h() -> Result<()> {
    g().context("g failed")
}

const EXPECTED_DEBUG_F: &str = "\
oh no!

Backtrace disabled; run with RUST_LIB_BACKTRACE=1 environment variable to display a backtrace
";

const EXPECTED_DEBUG_G: &str = "\
f failed

Caused by:
    oh no!

Backtrace disabled; run with RUST_LIB_BACKTRACE=1 environment variable to display a backtrace
";

const EXPECTED_DEBUG_H: &str = "\
g failed

Caused by:
    0: f failed
    1: oh no!

Backtrace disabled; run with RUST_LIB_BACKTRACE=1 environment variable to display a backtrace
";

#[test]
fn test_display() {
    assert_eq!("g failed", h().unwrap_err().to_string());
}

#[test]
#[cfg_attr(not(backtrace), ignore)]
fn test_debug() {
    assert_eq!(EXPECTED_DEBUG_F, format!("{:?}", f().unwrap_err()));
    assert_eq!(EXPECTED_DEBUG_G, format!("{:?}", g().unwrap_err()));
    assert_eq!(EXPECTED_DEBUG_H, format!("{:?}", h().unwrap_err()));
}
