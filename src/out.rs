//! Wrapper for the "out pointer" pattern.
//!
//! This is functionally equivalent to `&Cell<MaybeUninit<T>>` except the only allowed operation is
//! to write a `T`. Using `MaybeUninit` directly would not have been equally general; a
//! `&mut MaybeUninit<T>` could never then be created from a `&mut T` and passed to safe code,
//! which can safely replace it with `MaybeUninit::uninit` and make the existence of `&mut T` UB.
//!
//! As for the "`&Cell<...>`", this is to be slightly weaker than Rust's normally strict
//! requirement that `&mut` references are never aliased, which can typically not be assumed when
//! getting pointers from C.

use core::{cell::UnsafeCell, fmt, marker::PhantomData, ptr::NonNull};

/// Wrapper for write-only "out pointers" that are safe to write to
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Out<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _marker: PhantomData<&'a UnsafeCell<T>>,
}
impl<'a, T: ?Sized> Out<'a, T> {
    #[inline]
    pub unsafe fn new(ptr: *mut T) -> Option<Self> {
        Some(Self::from_nonnull(NonNull::new(ptr)?))
    }
    /// # Safety
    ///
    /// - pointer must be valid for the duration of lifetime `'a`
    #[inline]
    pub unsafe fn from_nonnull(ptr: NonNull<T>) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }
    #[inline]
    pub fn from_mut(r: &'a mut T) -> Self {
        // SAFETY:
        //
        // - `r` will obviously have the same lifetime as Self
        // - having a mut ref is stronger than
        unsafe { Self::from_nonnull(r.into()) }
    }
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }
}
impl<'a, T> Out<'a, T> {
    #[inline]
    pub fn write(&mut self, t: T) {
        unsafe {
            self.ptr.as_ptr().write(t);
        }
    }
}
impl<T> fmt::Pointer for Out<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.ptr)
    }
}
