use crate::{Error, StdError};
use core::fmt::{self, Debug, Display};
use core::ops::Deref;

#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
use alloc::boxed::Box;

#[cfg(error_generic_member_access)]
use crate::nightly::{self, Request};

#[repr(transparent)]
pub struct MessageError<M>(pub M);

impl<M> Debug for MessageError<M>
where
    M: Display + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<M> Display for MessageError<M>
where
    M: Display + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<M> StdError for MessageError<M> where M: Display + Debug + 'static {}

#[repr(transparent)]
pub struct DisplayError<M>(pub M);

impl<M> Debug for DisplayError<M>
where
    M: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<M> Display for DisplayError<M>
where
    M: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<M> StdError for DisplayError<M> where M: Display + 'static {}

#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
#[repr(transparent)]
pub struct BoxedError(pub Box<dyn StdError + Send + Sync>);

#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
impl Debug for BoxedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
impl Display for BoxedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
impl StdError for BoxedError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }

    #[cfg(error_generic_member_access)]
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        nightly::provide(&*self.0, request);
    }
}

// An anyhow::Error held behind an Arc does not itself implement
// std::error::Error (anyhow::Error only derefs to a dyn Error), so
// converting one used to fall back to the message-only path and silently
// drop everything below the outermost context. Wrapping it transparently
// and delegating source() to the inner error's chain keeps the whole chain
// intact. repr(transparent) also means the vtable can keep downcasting to
// the wrapped value itself, as the message-only path did.
#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
#[repr(transparent)]
pub struct SharedError<T>(pub T);

#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
impl<T> Debug for SharedError<T>
where
    T: Deref<Target = Error>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&*self.0, f)
    }
}

#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
impl<T> Display for SharedError<T>
where
    T: Deref<Target = Error>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&*self.0, f)
    }
}

#[cfg(any(feature = "std", not(anyhow_no_core_error)))]
impl<T> StdError for SharedError<T>
where
    T: Deref<Target = Error>,
{
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        (**self.0).source()
    }

    #[cfg(error_generic_member_access)]
    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        nightly::provide(&**self.0, request);
    }
}
