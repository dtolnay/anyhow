#![allow(clippy::unnecessary_wraps)]

mod drop;

use self::drop::{DetectDrop, Flag};
use anyhow::{Error, Result};
use std::error::Error as StdError;

#[test]
fn test_convert() {
    let has_dropped = Flag::new();
    let error = Error::new(DetectDrop::new(&has_dropped));
    let box_dyn = Box::<dyn StdError>::from(error);
    assert_eq!("oh no!", box_dyn.to_string());
    drop(box_dyn);
    assert!(has_dropped.get());
}

#[test]
fn test_convert_send() {
    let has_dropped = Flag::new();
    let error = Error::new(DetectDrop::new(&has_dropped));
    let box_dyn = Box::<dyn StdError + Send>::from(error);
    assert_eq!("oh no!", box_dyn.to_string());
    drop(box_dyn);
    assert!(has_dropped.get());
}

#[test]
fn test_convert_send_sync() {
    let has_dropped = Flag::new();
    let error = Error::new(DetectDrop::new(&has_dropped));
    let box_dyn = Box::<dyn StdError + Send + Sync>::from(error);
    assert_eq!("oh no!", box_dyn.to_string());
    drop(box_dyn);
    assert!(has_dropped.get());
}

#[test]
fn test_question_mark() -> Result<(), Box<dyn StdError>> {
    fn f() -> Result<()> {
        Ok(())
    }
    f()?;
    Ok(())
}

#[test]
fn test_anyhow_from_arc_error_keeps_chain() {
    use std::sync::Arc;

    let error = Error::msg("error1").context("error2");
    let converted = anyhow::anyhow!(Arc::new(error));
    assert_eq!("error2", converted.to_string());
    assert_eq!("error2: error1", format!("{:#}", converted));
    assert_eq!("error1", converted.source().unwrap().to_string(),);
    assert_eq!(
        vec!["error2", "error1"],
        converted
            .chain()
            .map(ToString::to_string)
            .collect::<Vec<String>>(),
    );
    assert!(converted.downcast_ref::<Arc<Error>>().is_some());
}

#[test]
fn test_anyhow_from_arc_error_downcast_by_value() {
    use std::sync::Arc;

    let error = Error::msg("error1").context("error2");
    let converted = anyhow::anyhow!(Arc::new(error));
    let shared = converted.downcast::<Arc<Error>>().unwrap();
    assert_eq!("error2: error1", format!("{:#}", shared));
}
