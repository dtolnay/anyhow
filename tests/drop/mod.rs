use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;

#[derive(Debug)]
pub struct DetectDrop {
    has_dropped: Arc<AtomicBool>,
}

impl DetectDrop {
    pub fn new(has_dropped: &Arc<AtomicBool>) -> Self {
        DetectDrop {
            has_dropped: Arc::clone(has_dropped),
        }
    }
}

impl StdError for DetectDrop {}

impl Display for DetectDrop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "oh no!")
    }
}

impl Drop for DetectDrop {
    fn drop(&mut self) {
        let already_dropped = self.has_dropped.swap(true, SeqCst);
        assert!(!already_dropped);
    }
}
