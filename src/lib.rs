#![cfg_attr(backtrace, feature(backtrace))]

mod context;
mod error;

#[cfg(not(feature = "std"))]
compile_error!("no_std support is not implemented yet");

pub use crate::context::Context;
pub use crate::error::{Chain, Error};

/// `Result<T, Error>`
pub type Result<T> = std::result::Result<T, Error>;

/// Return early with an error.
///
/// This macro is equivalent to `return Err(From::from($err))`.
#[macro_export]
macro_rules! bail {
    ($err:expr) => {
        return std::result::Result::Err(std::convert::From::from($err));
    };
    ($err:expr,) => {
        $crate::bail!($err);
    };
}

/// Construct an ad-hoc error from a string.
///
/// This evaluates to an `Error`. It can take either just a string, or a format
/// string with arguments. It also can take any custom type which implements
/// `Debug` and `Display`.
#[macro_export]
macro_rules! anyhow {
    ($msg:expr) => {
        $crate::Error::new_adhoc($msg)
    };
    ($msg:expr,) => {
        $crate::anyhow!($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::Error::new_adhoc(format!($fmt, $($arg)*))
    };
}
