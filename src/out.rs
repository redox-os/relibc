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

use core::{cell::UnsafeCell, fmt, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

/// Wrapper for write-only "out pointers" that are safe to write to
// TODO: We may want to change this to &mut MaybeUninit, or have a generic parameter deciding
// whether it should be noalias or not
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Out<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _marker: PhantomData<&'a UnsafeCell<T>>,
}
impl<'a, T: ?Sized> Out<'a, T> {
    /// # Safety
    ///
    /// - pointer must either be NULL, or be valid for the duration of lifetime `'a`
    #[inline]
    pub unsafe fn nullable(ptr: *mut T) -> Option<Self> {
        Some(Self {
            ptr: NonNull::new(ptr)?,
            _marker: PhantomData,
        })
    }
    /// # Safety
    ///
    /// - pointer must be valid for the duration of lifetime `'a`
    #[inline]
    pub unsafe fn nonnull(ptr: *mut T) -> Self {
        if cfg!(debug_assertions) {
            assert!(!ptr.is_null());
        }
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _marker: PhantomData,
        }
    }
    #[inline]
    pub fn from_mut(r: &'a mut T) -> Self {
        // SAFETY:
        //
        // - `r` will obviously have the same lifetime as Self
        // - a Rust reference is obviously valid as a pointer, and the lifetime is tied to that of
        // this struct
        unsafe { Self::nonnull(r) }
    }
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }
}
impl<'a, T> Out<'a, T> {
    #[inline]
    pub fn from_uninit_mut(r: &'a mut MaybeUninit<T>) -> Self {
        // SAFETY:
        //
        // Same as for from_mut. It's fine if *r is uninitialized, as this wrapper only allows
        // writes.
        unsafe { Self::nonnull(r.as_mut_ptr()) }
    }
    #[inline]
    pub fn write(&mut self, t: T) {
        unsafe {
            self.ptr.as_ptr().write(t);
        }
    }
}
impl<'a, T, const N: usize> Out<'a, [T; N]> {
    #[inline]
    pub fn as_slice_mut<'b>(&'b mut self) -> Out<'b, [T]> {
        unsafe {
            let ptr: *mut [T; N] = self.as_mut_ptr();
            Out::from_raw_parts(ptr.cast::<T>(), N)
        }
    }
}
impl<'a, T> Out<'a, [T]> {
    /// # Safety
    ///
    /// If `len > 0`, `ptr` be valid for `len` elements of `T`, during lifetime `'a`.
    pub unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> Self {
        // Empty slices must be non-NULL in Rust, but C typically does not force this for
        // pointer-length pairs.
        let ptr = if len == 0 {
            core::ptr::dangling_mut::<T>()
        } else {
            ptr
        };
        unsafe { Self::nonnull(core::slice::from_raw_parts_mut(ptr, len)) }
    }
    pub fn len(&self) -> usize {
        self.ptr.as_ptr().len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
    pub fn copy_common_length_from_slice(&mut self, src: &[T]) -> usize
    where
        T: Copy,
    {
        let l = src.len().min(self.len());
        self.split_at_checked(l).unwrap()[0].copy_from_slice(&src[..l]);
        l
    }
    // TODO: better API, impl RangeBounds, also fn get(usize) -> Out<T>
    pub fn subslice<'b>(&'b mut self, start: usize, end: usize) -> Out<[T]> {
        assert!(start <= end);
        assert!(end <= self.len());
        unsafe { Self::from_raw_parts(self.as_mut_ptr().as_mut_ptr().add(start), end - start) }
    }
    pub fn index<'b>(&'b mut self, i: usize) -> Out<T> {
        assert!(i <= self.len());
        unsafe { Out::nonnull(self.as_mut_ptr().as_mut_ptr().add(i)) }
    }
}
// TODO: use bytemuck
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
    #[inline]
    pub fn cast_slice_to<'b, U>(mut self) -> Out<'b, [U]>
    where
        T: CastSlice<U>,
    {
        assert_eq!(self.as_mut_ptr().as_mut_ptr() as usize % align_of::<U>(), 0);

        let byte_length = self.as_mut_ptr().len() * size_of::<T>();

        unsafe {
            Out::from_raw_parts(
                self.as_mut_ptr().as_mut_ptr().cast(),
                byte_length / size_of::<U>(),
            )
        }
    }
}
// TODO: use bytemuck
pub unsafe trait CastSlice<U> {}
unsafe impl CastSlice<i8> for u8 {}
unsafe impl CastSlice<u8> for i8 {}
unsafe impl CastSlice<u8> for u8 {}
unsafe impl CastSlice<i8> for i8 {}

impl<T: ?Sized> fmt::Pointer for Out<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.ptr)
    }
}
impl<T: ?Sized> fmt::Debug for Out<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Out: {:p}]", self.ptr)
    }
}
/// Marker trait for types where it is sound to turn `Out<struct { ... }>` into `struct { ...:
/// Out<...> }` by simply referencing fields. This is safe for any struct but must not be
/// implemented for `Deref` types so that `Out<&struct { ... }>` is never projected in a way that
/// adds mutability.
pub unsafe trait OutProject {}

impl<'a, T: ?Sized> Out<'a, T> {
    pub unsafe fn with_lifetime_of<'b, U: ?Sized>(mut self, u: &'b U) -> Out<'b, T> {
        unsafe { Out::nonnull(self.as_mut_ptr()) }
    }
}

// TODO: unit tests
