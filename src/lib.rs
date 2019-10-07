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
//!   # pub trait Deserialize {}
//!   #
//!   # mod serde_json {
//!   #     use super::Deserialize;
//!   #     use std::io;
//!   #
//!   #     pub fn from_str<T: Deserialize>(json: &str) -> io::Result<T> {
//!   #         unimplemented!()
//!   #     }
//!   # }
//!   #
//!   # struct ClusterMap;
//!   #
//!   # impl Deserialize for ClusterMap {}
//!   #
//!   use anyhow::Result;
//!
//!   fn get_cluster_info() -> Result<ClusterMap> {
//!       let config = std::fs::read_to_string("cluster.json")?;
//!       let map: ClusterMap = serde_json::from_str(&config)?;
//!       Ok(map)
//!   }
//!   #
//!   # fn main() {}
//!   ```
//!
//! - Attach context to help the person troubleshooting the error understand
//!   where things went wrong. A low-level error like "No such file or
//!   directory" can be annoying to debug without more context about what higher
//!   level step the application was in the middle of.
//!
//!   ```
//!   # struct It;
//!   #
//!   # impl It {
//!   #     fn detach(&self) -> Result<()> {
//!   #         unimplemented!()
//!   #     }
//!   # }
//!   #
//!   use anyhow::{Context, Result};
//!
//!   fn main() -> Result<()> {
//!       # return Ok(());
//!       #
//!       # const _: &str = stringify! {
//!       ...
//!       # };
//!       #
//!       # let it = It;
//!       # let path = "./path/to/instrs.jsox";
//!       #
//!       it.detach().context("failed to detach the important thing")?;
//!
//!       let content = std::fs::read(path)
//!           .with_context(|| format!("failed to read instrs from {}", path))?;
//!       #
//!       # const _: &str = stringify! {
//!       ...
//!       # };
//!       #
//!       # Ok(())
//!   }
//!   ```
//!
//!   ```console
//!   Error: failed to read instrs from ./path/to/instrs.jsox
//!
//!   Caused by:
//!       No such file or directory (os error 2)
//!   ```
//!
//! - Downcasting is supported and can be by value, by shared reference, or by
//!   mutable reference as needed.
//!
//!   ```
//!   # use anyhow::anyhow;
//!   # use std::fmt::{self, Display};
//!   # use std::task::Poll;
//!   #
//!   # #[derive(Debug)]
//!   # enum DataStoreError {
//!   #     Censored(()),
//!   # }
//!   #
//!   # impl Display for DataStoreError {
//!   #     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//!   #         unimplemented!()
//!   #     }
//!   # }
//!   #
//!   # impl std::error::Error for DataStoreError {}
//!   #
//!   # const REDACTED_CONTENT: () = ();
//!   #
//!   # let error = anyhow!("...");
//!   # let root_cause = &error;
//!   #
//!   # let ret =
//!   // If the error was caused by redaction, then return a
//!   // tombstone instead of the content.
//!   match root_cause.downcast_ref::<DataStoreError>() {
//!       Some(DataStoreError::Censored(_)) => Ok(Poll::Ready(REDACTED_CONTENT)),
//!       None => Err(error),
//!   }
//!   # ;
//!   ```
//!
//! - A backtrace is captured and printed with the error if the underlying error
//!   type does not already provide its own. In order to see backtraces, the
//!   `RUST_LIB_BACKTRACE=1` environment variable must be defined.
//!
//! - Anyhow works with any error type that has an impl of `std::error::Error`,
//!   including ones defined in your crate. We do not bundle a `derive(Error)`
//!   macro but you can write the impls yourself or use a standalone macro like
//!   [err-derive].
//!
//!   [err-derive]: https://crates.io/crates/err-derive
//!
//!   ```
//!   # const IGNORE: &str = stringify! {
//!   #[derive(Error, Debug)]
//!   pub enum FormatError {
//!       #[error(display = "invalid header (expected {:?}, got {:?})", expected, found)]
//!       InvalidHeader {
//!           expected: String,
//!           found: String,
//!       },
//!       #[error(display = "missing attribute: {}", _0)]
//!       MissingAttribute(String),
//!   }
//!   # };
//!   ```
//!
//! - One-off error messages can be constructed using the `anyhow!` macro, which
//!   supports string interpolation and produces an `anyhow::Error`.
//!
//!   ```
//!   # use anyhow::{anyhow, Result};
//!   #
//!   # fn demo() -> Result<()> {
//!   #     let missing = "...";
//!   return Err(anyhow!("missing attribute: {}", missing));
//!   #     Ok(())
//!   # }
//!   ```
//!
//! <br>
//!
//! # Acknowledgements
//!
//! The implementation of the `anyhow::Error` type is forked from
//! `fehler::Exception` ([https://github.com/withoutboats/fehler][fehler]). This
//! library just exposes it under the more standard `Error` / `Result`
//! terminology rather than the `throw!` / `#[throws]` / `Exception` language of
//! exceptions.
//!
//! [fehler]: https://github.com/withoutboats/fehler

#![doc(html_root_url = "https://docs.rs/anyhow/1.0.0")]
#![cfg_attr(backtrace, feature(backtrace))]

mod context;
mod error;

#[cfg(not(feature = "std"))]
compile_error!("no_std support is not implemented yet");

pub use crate::context::Context;
pub use crate::error::{Chain, Error};

/// `Result<T, Error>`
///
/// This is a reasonable return type to use throughout your application but also
/// for `fn main`; if you do, failures will be printed along with any
/// [context][Context] and a backtrace if one was captured.
///
/// # Example
///
/// ```
/// # pub trait Deserialize {}
/// #
/// # mod serde_json {
/// #     use super::Deserialize;
/// #     use std::io;
/// #
/// #     pub fn from_str<T: Deserialize>(json: &str) -> io::Result<T> {
/// #         unimplemented!()
/// #     }
/// # }
/// #
/// # #[derive(Debug)]
/// # struct ClusterMap;
/// #
/// # impl Deserialize for ClusterMap {}
/// #
/// use anyhow::Result;
///
/// fn main() -> Result<()> {
///     # return Ok(());
///     let config = std::fs::read_to_string("cluster.json")?;
///     let map: ClusterMap = serde_json::from_str(&config)?;
///     println!("cluster info: {:#?}", map);
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

/// Return early with an error.
///
/// This macro is equivalent to `return Err(From::from($err))`.
///
/// # Example
///
/// ```
/// # use anyhow::{bail, Result};
/// #
/// # fn has_permission(user: usize, resource: usize) -> bool {
/// #     true
/// # }
/// #
/// # fn main() -> Result<()> {
/// #     let user = 0;
/// #     let resource = 0;
/// #
/// if !has_permission(user, resource) {
///     bail!("permission denied for accessing {}", resource);
/// }
/// #     Ok(())
/// # }
/// ```
///
/// ```
/// # use anyhow::{bail, Result};
/// # use std::fmt::{self, Display};
/// #
/// # #[derive(Debug)]
/// # enum ScienceError {
/// #     RecursionLimitExceeded,
/// # }
/// #
/// # impl std::error::Error for ScienceError {}
/// #
/// # impl Display for ScienceError {
/// #     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
/// #         unimplemented!()
/// #     }
/// # }
/// #
/// # const MAX_DEPTH: usize = 1;
/// #
/// # const IGNORE: &str = stringify! {
/// #[derive(Error, Debug)]
/// enum ScienceError {
///     #[error(display = "recursion limit exceeded")]
///     RecursionLimitExceeded,
///     ...
/// }
/// # };
///
/// # fn main() -> Result<()> {
/// #     let depth = 0;
/// #
/// if depth > MAX_DEPTH {
///     bail!(ScienceError::RecursionLimitExceeded);
/// }
/// #     Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return std::result::Result::Err($crate::anyhow!($msg));
    };
    ($err:expr $(,)?) => {
        return std::result::Result::Err(std::convert::From::from($err));
    };
    ($fmt:expr, $($arg:tt)*) => {
        return std::result::Result::Err($crate::anyhow!($fmt, $($arg)*));
    };
}

/// Construct an ad-hoc error from a string.
///
/// This evaluates to an `Error`. It can take either just a string, or a format
/// string with arguments. It also can take any custom type which implements
/// `Debug` and `Display`.
///
/// # Example
///
/// ```
/// # type V = ();
/// #
/// use anyhow::{anyhow, Result};
///
/// fn lookup(key: &str) -> Result<V> {
///     if key.len() != 16 {
///         return Err(anyhow!("key length must be 16 characters, got {:?}", key));
///     }
///     
///     // ...
///     # Ok(())
/// }
/// ```
#[macro_export]
macro_rules! anyhow {
    ($msg:expr $(,)?) => {
        $crate::private::new_adhoc($msg)
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
        #[cfg(backtrace)]
        let backtrace = Some(Backtrace::capture());

        #[cfg(not(backtrace))]
        let backtrace = None;

        Error::new_adhoc(message, backtrace)
    }
}
