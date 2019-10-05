#![feature(backtrace)]

mod as_error;
mod exception;
mod context;

#[doc(inline)]
/// Annotations a function that "throws" a Result.
///
/// Inside functions tagged with `throws`, you can use `?` and the `throw!` macro to return errors,
/// but you don't need to wrap the successful return values in `Ok`.
///
/// `throws` can optionally take a type as an argument, which will be the error type returned by
/// this function. By default, the function will throw `Exception`.
pub use fehler_macros::throws;

#[doc(inline)]
/// Derive for the Error trait.
///
/// If the type contains a `Backtrace`, it will generate the `backtrace` method to find it.
///
/// If this wraps a different, underlying error, you can tag that field with `#[error::source]`,
/// which will generate the `cause` and `source` methods correctly.
pub use fehler_macros::Error;

pub use crate::as_error::AsError;
pub use crate::exception::{Exception, Errors};
pub use crate::context::Context;

/// Throw an error.
///
/// This macro is equivalent to `Err($err)?`.
#[macro_export]
macro_rules! throw {
    ($err:expr)   => (return std::result::Result::Err(std::convert::From::from($err)))
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
