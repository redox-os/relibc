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
// TODO: We may want to change this to &mut MaybeUninit, or have a generic parameter deciding
// whether it should be noalias or not
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
impl<'a, T> Out<'a, [T]> {
    /// # Safety
    ///
    /// Must be valid for lifetime `'a` and writable.
    pub unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> Self {
        Self::from_nonnull(NonNull::new_unchecked(core::slice::from_raw_parts_mut(
            ptr, len,
        )))
    }
    pub fn len(&self) -> usize {
        self.ptr.as_ptr().len()
    }
    // TODO: Maybe strengthen lifetimes?
    #[inline]
    pub fn split_at_checked<'b>(&'b mut self, n: usize) -> Option<[Out<'b, [T]>; 2]> {
        let l = self.ptr.len();
        if n > l {
            return None;
        }
        Some([
            Out {
                ptr: unsafe {
                    NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(
                        self.ptr.as_mut_ptr(),
                        n,
                    ))
                },
                _marker: PhantomData,
            },
            Out {
                ptr: unsafe {
                    NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(
                        self.ptr.as_mut_ptr().add(n),
                        l - n,
                    ))
                },
                _marker: PhantomData,
            },
        ])
    }
    #[inline]
    pub fn copy_from_slice(&mut self, src: &[T])
    where
        T: Copy,
    {
        assert_eq!(
            self.ptr.len(),
            src.len(),
            "Out::copy_from_slice size mismatch"
        );
        unsafe {
            // SAFETY:
            //
            // - we have already know from the existence of self that the slice is a valid writable
            // pointer
            // - src is similarly also a valid readable pointer of the same type
            // - because of `T: Copy`, it is valid to copy bytes directly
            // - although self.ptr may alias, src must not alias with any writable pointer, and the
            // Copy bound ensures T cannot have interior mutability since `UnsafeCell: !Copy`
            self.ptr
                .as_mut_ptr()
                .copy_from_nonoverlapping(src.as_ptr(), src.len());
        }
    }
}
// TODO: different trait?
impl<T: plain::Plain> Out<'_, [T]> {
    pub fn zero(&mut self) {
        let l = self.ptr.len();
        unsafe {
            // SAFETY:
            // - already know the pointer is valid up to its length
            // - the Plain trait ensures zero is a valid bit pattern
            self.ptr.as_mut_ptr().write_bytes(0, l)
        }
    }
}
impl<T> fmt::Pointer for Out<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.ptr)
    }
}
