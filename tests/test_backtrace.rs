#![allow(clippy::let_underscore_untyped)]

#[rustversion::not(nightly)]
#[ignore = "requires nightly"]
#[test]
fn test_backtrace() {}

#[rustversion::nightly]
#[test]
fn test_backtrace() {
    use anyhow::anyhow;

    let error = anyhow!("oh no!");
    let _ = error.backtrace();
}

#[cfg(feature = "std")]
#[test]
fn test_new_with_backtrace() {
    use anyhow::Error;
    use std::backtrace::Backtrace;
    use std::io;

    let backtrace = Backtrace::force_capture();
    let expected = format!("{:?}", backtrace);
    let error =
        Error::new_with_backtrace(io::Error::new(io::ErrorKind::Other, "oh no!"), backtrace);
    assert_eq!(expected, format!("{:?}", error.backtrace()));
}
