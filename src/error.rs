use std::any::TypeId;
use std::backtrace::{Backtrace, BacktraceStatus};
use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;

/// The `Error` type, a wrapper around a dynamic error type.
///
/// `Error` functions a lot like `Box<dyn std::error::Error>`, with these
/// differences:
///
/// - `Error` requires that the error is `Send`, `Sync`, and `'static`
/// - `Error` guarantees that a backtrace will exist, even if the error type
///   did not provide one
/// - `Error` is represented as a narrow pointer - exactly one word in size,
///   instead of two.
pub struct Error {
    inner: Box<ErrorImpl<()>>,
}

impl Error {
    /// Create a new error object from any error type.
    ///
    /// The error type must be threadsafe and `'static`, so that the `Error`
    /// will be as well.
    ///
    /// If the error type does not provide a backtrace, a backtrace will be
    /// created here to ensure that a backtrace exists.
    pub fn new<E>(error: E) -> Error
    where
        E: StdError + Send + Sync + 'static,
    {
        Error::construct(error, TypeId::of::<E>())
    }

    #[doc(hidden)]
    pub fn new_adhoc<M>(message: M) -> Error
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Error::construct(MessageError(message), TypeId::of::<M>())
    }

    fn construct<E>(error: E, type_id: TypeId) -> Error
    where
        E: StdError + Send + Sync + 'static,
    {
        unsafe {
            let backtrace = match error.backtrace() {
                Some(_) => None,
                None => Some(Backtrace::capture()),
            };
            let obj: TraitObject = mem::transmute(&error as &dyn StdError);
            let vtable = obj.vtable;
            let inner = ErrorImpl {
                vtable,
                type_id,
                backtrace,
                error,
            };
            Error {
                inner: mem::transmute(Box::new(inner)),
            }
        }
    }

    /// View this error object as the underlying error.
    pub fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        &**self
    }

    /// View this error object as the underlying error, mutably.
    pub fn as_error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
        &mut **self
    }

    /// Get the backtrace for this Error.
    pub fn backtrace(&self) -> &Backtrace {
        // NB: this unwrap can only fail if the underlying error's backtrace
        // method is nondeterministic, which would only happen in maliciously
        // constructed code
        self.inner
            .backtrace
            .as_ref()
            .or_else(|| self.inner.error().backtrace())
            .expect("backtrace capture failed")
    }

    /// An iterator of errors contained by this Error.
    ///
    /// This iterator will visit every error in the "cause chain" of this error
    /// object, beginning with the error that this error object was created
    /// from.
    pub fn errors(&self) -> Errors<'_> {
        Errors {
            next: Some(self.inner.error()),
        }
    }

    /// Returns `true` if `E` is the type wrapped by this error object.
    pub fn is<E>(&self) -> bool
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        TypeId::of::<E>() == self.inner.type_id
    }

    /// Attempt to downcast the error object to a concrete type.
    pub fn downcast<E>(self) -> Result<E, Error>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        if let Some(error) = self.downcast_ref::<E>() {
            unsafe {
                let error = ptr::read(error);
                drop(ptr::read(&self.inner));
                mem::forget(self);
                Ok(error)
            }
        } else {
            Err(self)
        }
    }

    /// Downcast this error object by reference.
    pub fn downcast_ref<E>(&self) -> Option<&E>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        if self.is::<E>() {
            unsafe { Some(&*(self.inner.error() as *const dyn StdError as *const E)) }
        } else {
            None
        }
    }

    /// Downcast this error object by mutable reference.
    pub fn downcast_mut<E>(&mut self) -> Option<&mut E>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        if self.is::<E>() {
            unsafe { Some(&mut *(self.inner.error_mut() as *mut dyn StdError as *mut E)) }
        } else {
            None
        }
    }
}

impl<E> From<E> for Error
where
    E: StdError + Send + Sync + 'static,
{
    fn from(error: E) -> Error {
        Error::new(error)
    }
}

impl Deref for Error {
    type Target = dyn StdError + Send + Sync + 'static;
    fn deref(&self) -> &Self::Target {
        self.inner.error()
    }
}

impl DerefMut for Error {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.error_mut()
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.inner.error())?;

        let mut errors = self.errors().skip(1).enumerate();

        if let Some((n, error)) = errors.next() {
            writeln!(f, "\ncaused by:")?;
            writeln!(f, "\t{}: {}", n, error)?;
            for (n, error) in errors {
                writeln!(f, "\t{}: {}", n, error)?;
            }
        }

        let backtrace = self.backtrace();

        match backtrace.status() {
            BacktraceStatus::Captured => {
                writeln!(f, "\n{}", backtrace)?;
            }
            BacktraceStatus::Disabled => {
                writeln!(
                    f,
                    "\nbacktrace disabled; run with RUST_BACKTRACE=1 environment variable to display a backtrace"
                )?;
            }
            _ => {}
        }

        Ok(())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner.error())
    }
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl Drop for Error {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.inner.error_mut()) }
    }
}

// repr C to ensure that `E` remains in the final position
#[repr(C)]
struct ErrorImpl<E> {
    vtable: *const (),
    type_id: TypeId,
    backtrace: Option<Backtrace>,
    error: E,
}

// repr C to ensure that transmuting from trait objects is safe
#[repr(C)]
struct TraitObject {
    data: *const (),
    vtable: *const (),
}

#[repr(transparent)]
struct MessageError<M>(M)
where
    M: Display + Debug;

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

impl ErrorImpl<()> {
    fn error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        unsafe {
            mem::transmute(TraitObject {
                data: &self.error,
                vtable: self.vtable,
            })
        }
    }

    fn error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
        unsafe {
            mem::transmute(TraitObject {
                data: &mut self.error,
                vtable: self.vtable,
            })
        }
    }
}

/// Iterator of errors in an `Error`.
pub struct Errors<'a> {
    next: Option<&'a (dyn StdError + 'static)>,
}

impl<'a> Iterator for Errors<'a> {
    type Item = &'a (dyn StdError + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take()?;
        self.next = next.source();
        Some(next)
    }
}

#[cfg(test)]
mod repr_correctness {
    use super::*;

    use std::marker::Unpin;
    use std::mem;

    #[test]
    fn size_of_error() {
        assert_eq!(mem::size_of::<Error>(), mem::size_of::<usize>());
    }

    #[allow(dead_code)]
    fn assert_error_autotraits()
    where
        Error: Unpin + Send + Sync + 'static,
    {
    }

    #[test]
    fn destructors_work() {
        use std::sync::*;

        #[derive(Debug)]
        struct HasDrop(Box<Arc<Mutex<bool>>>);
        impl StdError for HasDrop {}
        impl Display for HasDrop {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "does something")
            }
        }
        impl Drop for HasDrop {
            fn drop(&mut self) {
                let mut has_dropped = self.0.lock().unwrap();
                assert!(!*has_dropped);
                *has_dropped = true;
            }
        }

        let has_dropped = Arc::new(Mutex::new(false));

        drop(Error::from(HasDrop(Box::new(has_dropped.clone()))));

        assert!(*has_dropped.lock().unwrap());
    }
}
