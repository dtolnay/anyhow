mod common;

use self::common::*;
use anyhow::{ensure, bail};

#[test]
fn test_messages() {
    assert_eq!("oh no!", bail_literal().unwrap_err().to_string());
    assert_eq!("oh no!", bail_fmt().unwrap_err().to_string());
    assert_eq!("oh no!", bail_error().unwrap_err().to_string());
}

#[test]
fn test_ensure() {
    let f = || {
        ensure!(1 + 1 == 2, "This is correct");
        Ok(())
    };
    assert!(f().is_ok());

    let v = 1;
    let f = || {
        ensure!(v + v == 2, "This is correct, v: {}", v);
        Ok(())
    };
    assert!(f().is_ok());

    let f = || {
        ensure!(v + v == 1, "This is not correct, v: {}", v);
        Ok(())
    };
    assert!(f().is_err());
}

#[test]
fn test_boxed_ensure() {
    fn inner() -> Result<(), Box<dyn std::error::Error>> {
        ensure!(true, "error");
        Ok(())
    }
}

#[test]
fn test_boxed_bail() {
    fn inner() -> Result<(), Box<dyn std::error::Error>> {
        bail!("error");
    }
}
