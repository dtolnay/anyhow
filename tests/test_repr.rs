use anyhow::Error;
use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::marker::Unpin;
use std::mem;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;

#[test]
fn size_of_error() {
    assert_eq!(mem::size_of::<Error>(), mem::size_of::<usize>());
}

#[test]
fn error_autotraits() {
    fn assert<E: Unpin + Send + Sync + 'static>() {}
    assert::<Error>();
}

#[test]
fn drop_works() {
    #[derive(Debug)]
    struct DetectDrop {
        has_dropped: Arc<AtomicBool>,
    }

    impl StdError for DetectDrop {}

    impl Display for DetectDrop {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "does something")
        }
    }

    impl Drop for DetectDrop {
        fn drop(&mut self) {
            let already_dropped = self.has_dropped.swap(true, SeqCst);
            assert!(!already_dropped);
        }
    }

    let has_dropped = Arc::new(AtomicBool::new(false));

    drop(Error::new(DetectDrop {
        has_dropped: has_dropped.clone(),
    }));

    assert!(has_dropped.load(SeqCst));
}
