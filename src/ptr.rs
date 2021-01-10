use crate::alloc::Box;
use core::marker::PhantomData;
use core::ptr::NonNull;

#[repr(transparent)]
pub struct Own<T> {
    pub ptr: NonNull<T>,
}

unsafe impl<T> Send for Own<T> {}
unsafe impl<T> Sync for Own<T> {}
impl<T> Copy for Own<T> {}
impl<T> Clone for Own<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Own<T> {
    pub fn new(ptr: Box<T>) -> Self {
        Own {
            ptr: unsafe { NonNull::new_unchecked(Box::into_raw(ptr)) },
        }
    }

    pub fn cast<U>(self) -> Own<U> {
        Own {
            ptr: self.ptr.cast(),
        }
    }

    pub unsafe fn boxed(self) -> Box<T> {
        Box::from_raw(self.ptr.as_ptr())
    }

    pub fn by_ref(&self) -> Ref<T> {
        Ref {
            ptr: self.ptr,
            lifetime: PhantomData,
        }
    }

    pub fn by_mut(&mut self) -> Mut<T> {
        Mut {
            ptr: self.ptr,
            lifetime: PhantomData,
        }
    }
}

#[repr(transparent)]
pub struct Ref<'a, T> {
    pub ptr: NonNull<T>,
    lifetime: PhantomData<&'a T>,
}

impl<'a, T> Copy for Ref<'a, T> {}
impl<'a, T> Clone for Ref<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Ref<'a, T> {
    pub fn new(ptr: &'a T) -> Self {
        Ref {
            ptr: NonNull::from(ptr),
            lifetime: PhantomData,
        }
    }

    pub fn cast<U>(self) -> Ref<'a, U> {
        Ref {
            ptr: self.ptr.cast(),
            lifetime: PhantomData,
        }
    }

    pub unsafe fn deref(self) -> &'a T {
        &*self.ptr.as_ptr()
    }
}

#[repr(transparent)]
pub struct Mut<'a, T> {
    pub ptr: NonNull<T>,
    lifetime: PhantomData<&'a mut T>,
}

impl<'a, T> Copy for Mut<'a, T> {}
impl<'a, T> Clone for Mut<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Mut<'a, T> {
    pub fn new(ptr: &'a mut T) -> Self {
        Mut {
            ptr: NonNull::from(ptr),
            lifetime: PhantomData,
        }
    }

    pub fn cast<U>(self) -> Mut<'a, U> {
        Mut {
            ptr: self.ptr.cast(),
            lifetime: PhantomData,
        }
    }

    pub fn extend<'b>(self) -> Mut<'b, T> {
        Mut {
            ptr: self.ptr,
            lifetime: PhantomData,
        }
    }

    pub unsafe fn deref_mut(self) -> &'a mut T {
        &mut *self.ptr.as_ptr()
    }

    pub unsafe fn read(self) -> T {
        self.ptr.as_ptr().read()
    }
}
