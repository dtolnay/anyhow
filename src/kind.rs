use crate::Error;
use std::error::Error as StdError;
use std::fmt::{Debug, Display};

#[cfg(backtrace)]
use std::backtrace::Backtrace;

pub struct Adhoc;

pub trait AdhocKind: Sized {
    #[inline]
    fn anyhow_kind(&self) -> Adhoc {
        Adhoc
    }
}

impl<T> AdhocKind for &T where T: ?Sized + Display + Debug + Send + Sync + 'static {}

impl Adhoc {
    pub fn new<M>(self, message: M) -> Error
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Error::from_adhoc(message, backtrace!())
    }
}

pub struct Trait;

pub trait TraitKind: Sized {
    #[inline]
    fn anyhow_kind(&self) -> Trait {
        Trait
    }
}

impl<T> TraitKind for T where T: StdError + Send + Sync + 'static {}

impl Trait {
    pub fn new<E>(self, error: E) -> Error
    where
        E: StdError + Send + Sync + 'static,
    {
        Error::from_std(error, backtrace!())
    }
}
