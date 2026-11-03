// Tagged dispatch mechanism for resolving the behavior of `anyhow!($expr)`.
//
// When anyhow! is given a single expr argument to turn into anyhow::Error, we
// want the resulting Error to pick up the input's implementation of source()
// and backtrace() if it has a std::error::Error impl, otherwise require nothing
// more than Display and Debug.
//
// Expressed in terms of specialization, we want something like:
//
//     trait AnyhowNew {
//         fn new(self) -> Error;
//     }
//
//     impl<T> AnyhowNew for T
//     where
//         T: Display + Debug + Send + Sync + 'static,
//     {
//         default fn new(self) -> Error {
//             /* no std error impl */
//         }
//     }
//
//     impl<T> AnyhowNew for T
//     where
//         T: std::error::Error + Send + Sync + 'static,
//     {
//         fn new(self) -> Error {
//             /* use std error's source() and backtrace() */
//         }
//     }
//
// Since specialization is not stable yet, instead we rely on autoref behavior
// of method resolution to perform tagged dispatch. Here we have two traits
// AdhocKind and TraitKind that both have an anyhow_kind() method. AdhocKind is
// implemented whether or not the caller's type has a std error impl, while
// TraitKind is implemented only when a std error impl does exist. The ambiguity
// is resolved by AdhocKind requiring an extra autoref so that it has lower
// precedence.
//
// The anyhow! macro will set up the call in this form:
//
//     #[allow(unused_imports)]
//     use $crate::__private::{AdhocKind, TraitKind};
//     let error = $msg;
//     (&error).anyhow_kind().new(error)

use crate::Error;
use core::fmt::{Debug, Display};

#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
use crate::StdError;
#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
use alloc::boxed::Box;

pub struct Adhoc;

#[doc(hidden)]
pub trait AdhocKind: Sized {
    #[inline]
    fn anyhow_kind(&self) -> Adhoc {
        Adhoc
    }
}

impl<T> AdhocKind for &T where T: ?Sized + Display + Debug + Send + Sync + 'static {}

impl Adhoc {
    #[cold]
    pub fn new<M>(self, message: M) -> Error
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Error::construct_from_adhoc(message, backtrace!())
    }
}

pub struct Trait;

#[doc(hidden)]
pub trait TraitKind: Sized {
    #[inline]
    fn anyhow_kind(&self) -> Trait {
        Trait
    }
}

impl<E> TraitKind for E where E: Into<Error> {}

impl Trait {
    #[cold]
    pub fn new<E>(self, error: E) -> Error
    where
        E: Into<Error>,
    {
        error.into()
    }
}

#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
pub struct Boxed;

#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
#[doc(hidden)]
pub trait BoxedKind: Sized {
    #[inline]
    fn anyhow_kind(&self) -> Boxed {
        Boxed
    }
}

#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
impl BoxedKind for Box<dyn StdError + Send + Sync> {}

#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
impl Boxed {
    #[cold]
    pub fn new(self, error: Box<dyn StdError + Send + Sync>) -> Error {
        let backtrace = backtrace_if_absent!(&*error);
        Error::construct_from_boxed(error, backtrace)
    }
}

// An anyhow::Error held behind an Arc does not implement
// std::error::Error (anyhow::Error only derefs to a dyn Error), so it
// matches neither TraitKind nor BoxedKind and used to fall through to the
// message-only AdhocKind, which kept only the outermost context and dropped
// the rest of the chain. SharedKind catches that shape before AdhocKind
// (fewer autorefs) and routes it through the chain-preserving SharedError
// wrapper instead.
//
// There is deliberately no From<Arc<Error>> impl to go with this: the
// compiler rejects such an impl as conflicting with the blanket From impl,
// since std could one day provide an Error impl for Arc wrappers of
// anyhow::Error. A borrowed &'static Error has the same underlying problem
// but cannot be covered here: AdhocKind's blanket impl for &T already
// applies to it with the same receiver, so a SharedKind impl for references
// would make the macro call ambiguous.
pub struct Shared;

#[doc(hidden)]
pub trait SharedKind: Sized {
    #[inline]
    fn anyhow_kind(&self) -> Shared {
        Shared
    }
}

#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
impl SharedKind for alloc::sync::Arc<Error> {}

#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
impl Shared {
    #[cold]
    pub fn new<T>(self, error: T) -> Error
    where
        T: core::ops::Deref<Target = Error> + Send + Sync + 'static,
    {
        Error::construct_from_shared(error)
    }
}
