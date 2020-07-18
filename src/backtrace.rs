#[cfg(backtrace)]
macro_rules! backtrace_if_absent {
    ($err:expr) => {
        match $err.backtrace() {
            Some(_) => None,
            None => Some(Backtrace::capture()),
        }
    };
}
