mod common;
mod drop;

use self::common::*;
use self::drop::DetectDrop;
use anyhow::Error;
use std::io;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;

#[test]
fn test_downcast() {
    assert_eq!(
        "oh no!",
        bail_literal().unwrap_err().downcast::<&str>().unwrap(),
    );
    assert_eq!(
        "oh no!",
        bail_fmt().unwrap_err().downcast::<String>().unwrap(),
    );
    assert_eq!(
        "oh no!",
        bail_error()
            .unwrap_err()
            .downcast::<io::Error>()
            .unwrap()
            .to_string(),
    );
}

#[test]
fn test_downcast_ref() {
    assert_eq!(
        "oh no!",
        *bail_literal().unwrap_err().downcast_ref::<&str>().unwrap(),
    );
    assert_eq!(
        "oh no!",
        bail_fmt().unwrap_err().downcast_ref::<String>().unwrap(),
    );
    assert_eq!(
        "oh no!",
        bail_error()
            .unwrap_err()
            .downcast_ref::<io::Error>()
            .unwrap()
            .to_string(),
    );
}

#[test]
fn test_downcast_mut() {
    assert_eq!(
        "oh no!",
        *bail_literal().unwrap_err().downcast_mut::<&str>().unwrap(),
    );
    assert_eq!(
        "oh no!",
        bail_fmt().unwrap_err().downcast_mut::<String>().unwrap(),
    );
    assert_eq!(
        "oh no!",
        bail_error()
            .unwrap_err()
            .downcast_mut::<io::Error>()
            .unwrap()
            .to_string(),
    );
}

#[test]
fn test_drop() {
    let has_dropped = Arc::new(AtomicBool::new(false));
    let error = Error::new(DetectDrop::new(&has_dropped));
    drop(error.downcast::<DetectDrop>().unwrap());
    assert!(has_dropped.load(SeqCst));
}
