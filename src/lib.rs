#![feature(backtrace)]

mod as_error;
mod context;
mod exception;

pub use crate::as_error::AsError;
pub use crate::context::Context;
pub use crate::exception::{Errors, Exception};

/// Throw an error.
///
/// This macro is equivalent to `Err($err)?`.
#[macro_export]
macro_rules! throw {
    ($err:expr) => {
        return std::result::Result::Err(std::convert::From::from($err));
    };
}

/// Construct an ad-hoc exception from a string.
///
/// This evaluates to an `Exception`. It can take either just a string, or a format string with
/// arguments. It also can take any custom type which implements `Debug` and `Display`.
#[macro_export]
macro_rules! error {
    ($e:expr)   => { $crate::Exception::new_adhoc($e) };
    ($($arg:tt)*) => { $crate::Exception::new_adhoc(format!($($arg)*)) };
}
