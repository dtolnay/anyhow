#![allow(clippy::eq_op, clippy::shadow_unrelated, clippy::wildcard_imports)]

mod common;

use self::common::*;
use anyhow::{anyhow, ensure};
use std::future;

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
fn test_temporaries() {
    fn require_send_sync(_: impl Send + Sync) {}

    require_send_sync(async {
        // If anyhow hasn't dropped any temporary format_args it creates by the
        // time it's done evaluating, those will stick around until the
        // semicolon, which is on the other side of the await point, making the
        // enclosing future non-Send.
        future::ready(anyhow!("...")).await;
    });
}
