use std::any::TypeId;
use std::backtrace::{Backtrace, BacktraceStatus};
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;

/// The `Exception type, a wrapper around a dynamic error type.
///
/// `Exception` functions a lot like `Box<dyn Error>`, with these differences:
///
/// - `Exception` requires that the error is `Send`, `Sync`, and `'static`
/// - `Exception` guarantees that a backtrace will exist, even if the error type
///   did not provide one
/// - `Exception` is represented as a narrow pointer - exactly one word in size,
///   instead of two.
pub struct Exception {
    inner: Box<InnerException<()>>,
}

impl Exception {
    /// Create a new exception from any error type.
    ///
    /// The error type must be threadsafe and `'static`, so that the `Exception` will be as well.
    ///
    /// If the error type does not provide a backtrace, a backtrace will be created here to ensure
    /// that a backtrace exists.
    pub fn new<E>(error: E) -> Exception where
        E: Error + Send + Sync + 'static
    {
        Exception::construct(error, TypeId::of::<E>())
    }

    #[doc(hidden)]
    pub fn new_adhoc<M>(message: M) -> Exception where
        M: Display + Debug + Send + Sync + 'static
    {
        Exception::construct(MessageError(message), TypeId::of::<M>())
    }

    fn construct<E>(error: E, type_id: TypeId) -> Exception where
        E: Error + Send + Sync + 'static,
    {
        unsafe {
            let backtrace = match error.backtrace() {
                Some(_) => None,
                None    => Some(Backtrace::capture()),
            };
            let obj: TraitObject = mem::transmute(&error as &dyn Error);
            let vtable = obj.vtable;
            let inner = InnerException { vtable, type_id, backtrace, error };
            Exception {
                inner: mem::transmute(Box::new(inner))
            }
        }
    }

    /// View this exception as the underlying error.
    pub fn as_error(&self) -> &(dyn Error + Send + Sync + 'static) {
        &**self
    }

    /// View this exception as the underlying error, mutably.
    pub fn as_error_mut(&mut self) -> &mut (dyn Error + Send + Sync + 'static) {
        &mut **self
    }

    /// Get the backtrace for this Exception.
    pub fn backtrace(&self) -> &Backtrace {
        // NB: this unwrap can only fail if the underlying error's backtrace method is
        // nondeterministic, which would only happen in maliciously constructed code
        self.inner.backtrace.as_ref().or_else(|| self.inner.error().backtrace())
            .expect("exception backtrace capture failed")
    }

    /// An iterator of errors contained by this Exception.
    ///
    /// This iterator will visit every error in the "cause chain" of this exception, beginning with
    /// the error that this exception was created from.
    pub fn errors(&self) -> Errors<'_> {
        Errors { next: Some(self.inner.error()) }
    }

    /// Returns `true` if `E` is the type wrapped by this exception.
    pub fn is<E: Display + Debug + Send + Sync + 'static>(&self) -> bool {
        TypeId::of::<E>() == self.inner.type_id
    }

    /// Attempt to downcast the exception to a concrete type.
    pub fn downcast<E: Display + Debug + Send + Sync + 'static>(self) -> Result<E, Exception> {
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

    /// Downcast this exception by reference.
    pub fn downcast_ref<E: Display + Debug + Send + Sync + 'static>(&self) -> Option<&E> {
        if self.is::<E>() {
            unsafe { Some(&*(self.inner.error() as *const dyn Error as *const E)) }
        } else { None }
    }

    /// Downcast this exception by mutable reference.
    pub fn downcast_mut<E: Display + Debug + Send + Sync + 'static>(&mut self) -> Option<&mut E> {
        if self.is::<E>() {
            unsafe { Some(&mut *(self.inner.error_mut() as *mut dyn Error as *mut E)) }
        } else { None }
    }
}

impl<E: Error + Send + Sync + 'static> From<E> for Exception {
    fn from(error: E) -> Exception {
        Exception::new(error)
    }
}

impl Deref for Exception {
    type Target = dyn Error + Send + Sync + 'static;
    fn deref(&self) -> &Self::Target {
        self.inner.error()
    }
}

impl DerefMut for Exception {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.error_mut()
    }
}

impl Debug for Exception {
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
            BacktraceStatus::Captured       => {
                writeln!(f, "\n{}", backtrace)?;
            }
            BacktraceStatus::Disabled       => {
                writeln!(f, "\nbacktrace disabled; run with RUST_BACKTRACE=1 environment variable \
                             to display a backtrace")?;
            }
            _                               => { }
        }

        Ok(())
    }
}

impl Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner.error())
    }
}

unsafe impl Send for Exception { }
unsafe impl Sync for Exception { }

impl Drop for Exception {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.inner.error_mut()) }
    }
}

// repr C to ensure that `E` remains in the final position
#[repr(C)]
struct InnerException<E> {
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
struct MessageError<M: Display + Debug>(M);

impl<M: Display + Debug> Debug for MessageError<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<M: Display + Debug> Display for MessageError<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<M: Display + Debug + 'static> Error for MessageError<M> { }

impl InnerException<()> {
    fn error(&self) -> &(dyn Error + Send + Sync + 'static) {
        unsafe {
            mem::transmute(TraitObject {
                data: &self.error,
                vtable: self.vtable,
            })
        }
    }

    fn error_mut(&mut self) -> &mut (dyn Error + Send + Sync + 'static) {
        unsafe {
            mem::transmute(TraitObject {
                data: &mut self.error,
                vtable: self.vtable,
            })
        }
    }
}

/// Iterator of errors in an `Exception`.
pub struct Errors<'a> {
    next: Option<&'a (dyn Error + 'static)>,
}

impl<'a> Iterator for Errors<'a> {
    type Item = &'a (dyn Error + 'static);
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take()?;
        self.next = next.source();
        Some(next)
    }
}

#[cfg(test)]
mod repr_correctness {
    use super::*;

    use std::mem;
    use std::marker::Unpin;

    #[test]
    fn size_of_exception() {
        assert_eq!(mem::size_of::<Exception>(), mem::size_of::<usize>());
    }

    #[allow(dead_code)] fn assert_exception_autotraits() where
        Exception: Unpin + Send + Sync + 'static
    { }

    #[test]
    fn destructors_work() {
        use std::sync::*;

        #[derive(Debug)] struct HasDrop(Box<Arc<Mutex<bool>>>);
        impl Error for HasDrop { }
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

        drop(Exception::from(HasDrop(Box::new(has_dropped.clone()))));

        assert!(*has_dropped.lock().unwrap());

    }
}
