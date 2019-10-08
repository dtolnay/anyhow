mod drop;

use self::drop::DetectDrop;
use anyhow::Error;
use std::marker::Unpin;
use std::mem;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;

#[test]
fn test_error_size() {
    assert_eq!(mem::size_of::<Error>(), mem::size_of::<usize>());
}

#[test]
fn test_autotraits() {
    fn assert<E: Unpin + Send + Sync + 'static>() {}
    assert::<Error>();
}

#[test]
fn test_drop() {
    let has_dropped = Arc::new(AtomicBool::new(false));
    drop(Error::new(DetectDrop::new(&has_dropped)));
    assert!(has_dropped.load(SeqCst));
}
