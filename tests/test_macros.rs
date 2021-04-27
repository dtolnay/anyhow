#![allow(clippy::eq_op, clippy::shadow_unrelated, clippy::wildcard_imports)]

mod common;

use self::common::*;
use anyhow::{ensure, ensure_eq, ensure_ne};

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

    let f = || {
        ensure!(v + v == 1);
        Ok(())
    };
    assert_eq!(
        f().unwrap_err().to_string(),
        "Condition failed: `v + v == 1`",
    );
}

#[test]
fn test_ensure_eq() {
    let f = || {
        ensure_eq!(1 + 1, 2, "This is correct");
        Ok(())
    };
    assert!(f().is_ok());

    let v = 1;
    let f = || {
        ensure_eq!(v + v, 2, "This is correct, v: {}", v);
        Ok(())
    };
    assert!(f().is_ok());

    let f = || {
        ensure_eq!(v + v, 1, "This is not correct, v: {}", v);
        Ok(())
    };
    assert!(f().is_err());

    let f = || {
        ensure_eq!(v + v, 1);
        Ok(())
    };
    assert_eq!(
        f().unwrap_err().to_string(),
        r#"Condition failed: `(left == right)`
    left: `2`,
   right: `1`"#,
    );
}

#[test]
fn test_ensure_ne() {
    let f = || {
        ensure_ne!(1 + 2, 2, "This is correct");
        Ok(())
    };
    assert!(f().is_ok());

    let v = 1;
    let f = || {
        ensure_ne!(v + v, 3, "This is correct, v: {}", v);
        Ok(())
    };
    assert!(f().is_ok());

    let f = || {
        ensure_ne!(v + v, 2, "This is not correct, v: {}", v);
        Ok(())
    };
    assert!(f().is_err());

    let f = || {
        ensure_ne!(v + v, 2);
        Ok(())
    };
    assert_eq!(
        f().unwrap_err().to_string(),
        r#"Condition failed: `(left != right)`
    left: `2`,
   right: `2`"#,
    );
}
