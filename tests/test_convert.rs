mod drop;

use self::drop::DetectDrop;
use anyhow::Error;
use std::error::Error as StdError;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;

#[test]
fn test_convert() {
    let has_dropped = Arc::new(AtomicBool::new(false));
    let error = Error::new(DetectDrop::new(&has_dropped));
    let box_dyn = Box::<dyn StdError + Send + Sync>::from(error);
    assert_eq!("oh no!", box_dyn.to_string());
    drop(box_dyn);
    assert!(has_dropped.load(SeqCst));
}
