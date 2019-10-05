use std::error::Error;

use crate::Exception;

/// View type as an error object.
pub trait AsError {
    /// View type as an error object.
    fn as_error(&self) -> &(dyn Error + Send + Sync + 'static);
}

impl<T: Error + Send + Sync + 'static> AsError for T {
    fn as_error(&self) -> &(dyn Error + Send + Sync + 'static) {
        self
    }
}

impl AsError for dyn Error + Send + Sync + 'static {
    fn as_error(&self) -> &(dyn Error + Send + Sync + 'static) {
        self
    }
}

impl AsError for Exception {
    fn as_error(&self) -> &(dyn Error + Send + Sync + 'static) {
        &**self
    }
}
