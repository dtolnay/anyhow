use std::backtrace::Backtrace;
use std::fmt::{self, Debug, Display};
use std::error::Error;

use crate::Exception;

/// Provides the `context` method for `Result`.
pub trait Context<T, E> {
    /// Wrap the error value with additional context.
    fn context<C: Display + Send + Sync + 'static>(self, context: C) -> Result<T, Exception>;

    /// Wrap the error value with additional context lazily.
    fn with_context<C, F>(self, f: F) -> Result<T, Exception> where
        C: Display + Send + Sync + 'static,
        F: FnOnce(&E) -> C;
}

impl<T, E: Error + Send + Sync + 'static> Context<T, E> for Result<T, E> {
    fn context<C: Display + Send + Sync + 'static>(self, context: C) -> Result<T, Exception> {
        self.map_err(|error| Exception::from(ContextError { error, context }))
    }

    fn with_context<C, F>(self, f: F) -> Result<T, Exception> where
        C: Display + Send + Sync + 'static,
        F: FnOnce(&E) -> C
    {
        self.map_err(|error| Exception::from(ContextError { context: f(&error), error }))
    }
}

impl<T> Context<T, Exception> for Result<T, Exception> {
    fn context<C: Display + Send + Sync + 'static>(self, context: C) -> Result<T, Exception> {
        self.map_err(|error| Exception::from(ContextError { error, context }))
    }

    fn with_context<C, F>(self, f: F) -> Result<T, Exception> where
        C: Display + Send + Sync + 'static,
        F: FnOnce(&Exception) -> C
    {
        self.map_err(|error| Exception::from(ContextError { context: f(&error), error }))
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

impl<E: Error + 'static, C: Display> Error for ContextError<E, C> {
    fn backtrace(&self) -> Option<&Backtrace> {
        self.error.backtrace()
    }

    fn cause(&self) -> Option<&dyn Error> {
        Some(&self.error)
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

impl<C: Display> Error for ContextError<Exception, C> {
    fn backtrace(&self) -> Option<&Backtrace> {
        Some(self.error.backtrace())
    }

    fn cause(&self) -> Option<&dyn Error> {
        Some(&*self.error)
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.error)
    }
}
