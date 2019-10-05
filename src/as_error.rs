use std::error::Error as StdError;

use crate::Error;

/// View type as an error object.
pub trait AsError {
    /// View type as an error object.
    fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static);
}

impl<T: StdError + Send + Sync + 'static> AsError for T {
    fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }
}

impl AsError for dyn StdError + Send + Sync + 'static {
    fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }
}

impl AsError for Error {
    fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        &**self
    }
}
