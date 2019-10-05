use std::error::Error as StdError;

use crate::Error;

/// View error as a `&dyn std::error::Error`.
pub trait AsDynError {
    /// View type as an error object.
    fn as_dyn_error(&self) -> &(dyn StdError + Send + Sync + 'static);
}

impl<T> AsDynError for T
where
    T: StdError + Send + Sync + 'static,
{
    fn as_dyn_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }
}

impl AsDynError for dyn StdError + Send + Sync + 'static {
    fn as_dyn_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }
}

impl AsDynError for Error {
    fn as_dyn_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        &**self
    }
}
