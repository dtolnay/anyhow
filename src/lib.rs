#![feature(backtrace)]

mod as_error;
mod context;
mod error;

pub use crate::as_error::AsError;
pub use crate::context::Context;
pub use crate::error::{Error, Errors};

/// Return early with an error.
///
/// This macro is equivalent to `return Err(From::from($err))`.
#[macro_export]
macro_rules! bail {
    ($err:expr) => {
        return std::result::Result::Err(std::convert::From::from($err));
    };
}

/// Construct an ad-hoc exception from a string.
///
/// This evaluates to an `Error`. It can take either just a string, or a format string with
/// arguments. It also can take any custom type which implements `Debug` and `Display`.
#[macro_export]
macro_rules! error {
    ($e:expr)   => { $crate::Error::new_adhoc($e) };
    ($($arg:tt)*) => { $crate::Error::new_adhoc(format!($($arg)*)) };
}
