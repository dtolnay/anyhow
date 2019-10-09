use crate::Error;
use std::convert::Infallible;
use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};

#[cfg(backtrace)]
use std::backtrace::Backtrace;

/// Provides the `context` method for `Result`.
///
/// This trait is sealed and cannot be implemented for types outside of
/// `anyhow`.
///
/// # Example
///
/// ```
/// use anyhow::{Context, Result};
/// use std::fs;
/// use std::path::PathBuf;
///
/// pub struct ImportantThing {
///     path: PathBuf,
/// }
///
/// impl ImportantThing {
///     # const IGNORE: &'static str = stringify! {
///     pub fn detach(&mut self) -> Result<()> {...}
///     # };
///     # fn detach(&mut self) -> Result<()> {
///     #     unimplemented!()
///     # }
/// }
///
/// pub fn do_it(mut it: ImportantThing) -> Result<Vec<u8>> {
///     it.detach().context("failed to detach the important thing")?;
///
///     let path = &it.path;
///     let content = fs::read(path)
///         .with_context(|| format!("failed to read instrs from {}", path.display()))?;
///
///     Ok(content)
/// }
/// ```
///
/// When printed, the outermost context would be printed first and the lower
/// level underlying causes would be enumerated below.
///
/// ```console
/// Error: failed to read instrs from ./path/to/instrs.jsox
///
/// caused by:
///     No such file or directory (os error 2)
/// ```
pub trait Context<T, E>: private::Sealed {
    /// Wrap the error value with additional context.
    fn context<C>(self, context: C) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static;

    /// Wrap the error value with additional context that is evaluated lazily
    /// only once an error does occur.
    fn with_context<C, F>(self, f: F) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> Context<T, E> for Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|error| Error::new(ContextError { error, context }))
    }

    fn with_context<C, F>(self, context: F) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|error| {
            Error::new(ContextError {
                error,
                context: context(),
            })
        })
    }
}

impl<T> Context<T, Error> for Result<T, Error> {
    fn context<C>(self, context: C) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|error| error.context(context))
    }

    fn with_context<C, F>(self, context: F) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|error| error.context(context()))
    }
}

/// ```
/// # type T = ();
/// #
/// use anyhow::{Context, Result};
///
/// fn maybe_get() -> Option<T> {
///     # const IGNORE: &str = stringify! {
///     ...
///     # };
///     # unimplemented!()
/// }
///
/// fn demo() -> Result<()> {
///     let t = maybe_get().context("there is no T")?;
///     # const IGNORE: &str = stringify! {
///     ...
///     # };
///     # unimplemented!()
/// }
/// ```
impl<T> Context<T, Infallible> for Option<T> {
    fn context<C>(self, context: C) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
    {
        self.ok_or_else(|| Error::from_display(context, backtrace!()))
    }

    fn with_context<C, F>(self, context: F) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.ok_or_else(|| Error::from_display(context(), backtrace!()))
    }
}

pub(crate) struct ContextError<E, C> {
    pub error: E,
    pub context: C,
}

impl<E, C> Debug for ContextError<E, C>
where
    E: Debug,
    C: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}\n\n{}", self.error, self.context)
    }
}

impl<E, C> Display for ContextError<E, C>
where
    C: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.context, f)
    }
}

impl<E, C> StdError for ContextError<E, C>
where
    E: StdError + 'static,
    C: Display,
{
    #[cfg(backtrace)]
    fn backtrace(&self) -> Option<&Backtrace> {
        self.error.backtrace()
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.error)
    }
}

impl<C> StdError for ContextError<Error, C>
where
    C: Display,
{
    #[cfg(backtrace)]
    fn backtrace(&self) -> Option<&Backtrace> {
        Some(self.error.backtrace())
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&*self.error)
    }
}

mod private {
    use crate::Error;
    use std::error::Error as StdError;

    pub trait Sealed {}

    impl<T, E> Sealed for Result<T, E> where E: StdError + Send + Sync + 'static {}
    impl<T> Sealed for Result<T, Error> {}
    impl<T> Sealed for Option<T> {}
}
