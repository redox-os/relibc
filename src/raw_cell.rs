use core::cell::UnsafeCell;

/// Wrapper over `UnsafeCell` that can directly be used in statics, where all modifications require
/// unsafe.
#[repr(transparent)]
pub struct RawCell<T> {
    inner: UnsafeCell<T>,
}
impl<T> RawCell<T> {
    #[inline]
    pub const fn new(t: T) -> Self {
        Self {
            inner: UnsafeCell::new(t),
        }
    }
    #[inline]
    pub fn as_mut_ptr(&self) -> *mut T {
        self.inner.get()
    }
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
    #[inline]
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
    #[inline]
    pub unsafe fn unsafe_ref(&self) -> &T {
        &*self.inner.get()
    }
    #[inline]
    pub unsafe fn unsafe_set(&self, t: T) {
        *self.inner.get() = t;
    }
    #[inline]
    pub unsafe fn unsafe_mut(&self) -> &mut T {
        &mut *self.inner.get()
    }
}

// SAFETY: Sync requires that no safe interface be allowed to act on &self in a way that is
// undefined behavior when accessed concurrently. The interface above only allows get, set, and
// as_mut_ptr, where the former two that access memory are unsafe anyway.
unsafe impl<T> Sync for RawCell<T> {}

const _: () = {
    // Check that RawCell works for non-Sync types.
    static X: RawCell<*mut ()> = RawCell::new(core::ptr::null_mut());
};
