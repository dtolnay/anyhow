use std::backtrace::Backtrace;
use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};

use crate::Error;

/// Provides the `context` method for `Result`.
pub trait Context<T, E> {
    /// Wrap the error value with additional context.
    fn context<C: Display + Send + Sync + 'static>(self, context: C) -> Result<T, Error>;

    /// Wrap the error value with additional context lazily.
    fn with_context<C, F>(self, f: F) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce(&E) -> C;
}

impl<T, E: StdError + Send + Sync + 'static> Context<T, E> for Result<T, E> {
    fn context<C: Display + Send + Sync + 'static>(self, context: C) -> Result<T, Error> {
        self.map_err(|error| Error::from(ContextError { error, context }))
    }

    fn with_context<C, F>(self, f: F) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce(&E) -> C,
    {
        self.map_err(|error| {
            Error::from(ContextError {
                context: f(&error),
                error,
            })
        })
    }
}

impl<T> Context<T, Error> for Result<T, Error> {
    fn context<C: Display + Send + Sync + 'static>(self, context: C) -> Result<T, Error> {
        self.map_err(|error| Error::from(ContextError { error, context }))
    }

    fn with_context<C, F>(self, f: F) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce(&Error) -> C,
    {
        self.map_err(|error| {
            Error::from(ContextError {
                context: f(&error),
                error,
            })
        })
    }
}

struct ContextError<E, C> {
    error: E,
    context: C,
}

impl<E: Debug, C: Display> Debug for ContextError<E, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}\n\n{}", self.error, self.context)
    }
}

impl<E, C: Display> Display for ContextError<E, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.context, f)
    }
}

impl<E: StdError + 'static, C: Display> StdError for ContextError<E, C> {
    fn backtrace(&self) -> Option<&Backtrace> {
        self.error.backtrace()
    }

    fn cause(&self) -> Option<&dyn StdError> {
        Some(&self.error)
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.error)
    }
}

impl<C: Display> StdError for ContextError<Error, C> {
    fn backtrace(&self) -> Option<&Backtrace> {
        Some(self.error.backtrace())
    }

    fn cause(&self) -> Option<&dyn StdError> {
        Some(&*self.error)
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&*self.error)
    }
}
