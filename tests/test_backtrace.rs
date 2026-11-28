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

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[test]
fn test_debug_without_backtrace_support() {
    use anyhow::anyhow;
    use std::backtrace::BacktraceStatus;

    let error = anyhow!("cause").context("outer");
    assert_ne!(error.backtrace().status(), BacktraceStatus::Captured);
    assert_eq!(format!("{error:?}"), "outer\n\nCaused by:\n    cause");
}
