//! This library provides [`anyhow::Error`][Error], a trait object based error
//! type for easy idiomatic error handling in Rust applications.
//!
//! <br>
//!
//! # Details
//!
//! - Use `Result<T, anyhow::Error>`, or equivalently `anyhow::Result<T>`, as
//!   the return type of any fallible function.
//!
//!   Within the function, use `?` to easily propagate any error that implements
//!   the `std::error::Error` trait.
//!
//!   ```
//!   use anyhow::Result;
//!
//!   fn get_cluster_info() -> Result<ClusterMap> {
//!       let config = std::fs::read_to_string("cluster.json")?;
//!       let map: ClusterMap = serde_json::from_str(&config)?;
//!       Ok(map)
//!   }
//!   ```
//!
//! - Attach context to help the person troubleshooting the error understand
//!   where things went wrong. A low-level error like "No such file or
//!   directory" can be annoying to debug without more context about what higher
//!   level step the application was in the middle of.
//!
//!   ```
//!   use anyhow::{Context, Result};
//!
//!   fn main() -> Result<()> {
//!       ...
//!       it.detach().context("failed to detach the important thing")?;
//!
//!       let content = std::fs::read(path)
//!           .with_context(|| format!("failed to read instrs from {}", path))?;
//!       ...
//!   }
//!   ```
//!
//!   ```console
//!   Error: failed to read instrs from ./path/to/instrs.jsox
//!
//!   caused by:
//!       0: No such file or directory (os error 2)
//!   ```
//!
//! - Downcasting is supported and can be by value, by shared reference, or by
//!   mutable reference as needed.
//!
//!   ```
//!   // If the error was caused by redaction, then return a
//!   // tombstone instead of the content.
//!   match root_cause.downcast_ref::<DataStoreError>() {
//!       Some(DataStoreError::Censored(_)) => Ok(Async::Ready(REDACTED_CONTENT)),
//!       None => Err(e),
//!   }
//!   ```
//!
//! - A backtrace is captured and printed with the error if the underlying error
//!   type does not already provide its own. In order to see backtraces, the
//!   `RUST_LIB_BACKTRACE=1` environment variable must be defined.
//!
//! <br>
//!
//! # Acknowledgements
//!
//! The implementation of the `anyhow::Error` type is identical to
//! `fehler::Exception` ([https://github.com/withoutboats/fehler][fehler]). This
//! library just exposes it under the more standard `Error` / `Result`
//! terminology rather than the `throw!` / `#[throws]` / `Exception` language of
//! exceptions.
//!
//! [fehler]: https://github.com/withoutboats/fehler

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
        $crate::private::new_adhoc($msg)
    };
    ($msg:expr,) => {
        $crate::anyhow!($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::private::new_adhoc(format!($fmt, $($arg)*))
    };
}

// Not public API.
#[doc(hidden)]
pub mod private {
    use crate::Error;
    use std::fmt::{Debug, Display};

    #[cfg(backtrace)]
    use std::backtrace::Backtrace;

    pub fn new_adhoc<M>(message: M) -> Error
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Error::new_adhoc(message, #[cfg(backtrace)] Some(Backtrace::capture()))
    }
}
