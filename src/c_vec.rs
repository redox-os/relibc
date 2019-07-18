use core::iter::IntoIterator;
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};
use core::{cmp, mem, slice};
use crate::platform::types::*;
use crate::platform;

/// Error that occurs when an allocation fails
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub struct AllocError;

/// A normal vector allocated in Rust needs to be dropped from Rust
/// too, in order to avoid UB. This CVec is an abstraction that works
/// using only C allocations functions and can therefore be dropped
/// from C. Just like the Rust Vec, this does bounds checks to assure
/// you never reach isize::MAX. Unless you need to drop something from
/// C, prefer Rust's builtin Vec.
pub struct CVec<T> {
    ptr: NonNull<T>,
    len: usize,
    cap: usize
}
impl<T> CVec<T> {
    pub fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0
        }
    }
    fn check_bounds(i: usize) -> Result<usize, AllocError> {
        if i > core::isize::MAX as usize {
            Err(AllocError)
        } else {
            Ok(i)
        }
    }
    fn check_mul(x: usize, y: usize) -> Result<usize, AllocError> {
        x.checked_mul(y).ok_or(AllocError)
            .and_then(Self::check_bounds)
    }
    pub fn with_capacity(cap: usize) -> Result<Self, AllocError> {
        if cap == 0 {
            return Ok(Self::new());
        }
        let size = Self::check_mul(cap, mem::size_of::<T>())?;
        let ptr = NonNull::new(unsafe { platform::alloc(size) as *mut T }).ok_or(AllocError)?;
        Ok(Self {
            ptr,
            len: 0,
            cap
        })
    }
    unsafe fn resize(&mut self, cap: usize) -> Result<(), AllocError> {
        let size = Self::check_mul(cap, mem::size_of::<T>())?;
        let ptr = NonNull::new(platform::realloc(self.ptr.as_ptr() as *mut c_void, size) as *mut T).ok_or(AllocError)?;
        self.ptr = ptr;
        self.cap = cap;
        Ok(())
    }
    unsafe fn drop_range(&mut self, start: usize, end: usize) {
        let mut start = self.ptr.as_ptr().add(start);
        let end = self.ptr.as_ptr().add(end);
        while start < end {
            ptr::drop_in_place(start);
            start = start.add(1);
        }
    }
    pub fn reserve(&mut self, required: usize) -> Result<(), AllocError> {
        let reserved_len = self.len.checked_add(required)
            .ok_or(AllocError)
            .and_then(Self::check_bounds)?;
        let new_cap = cmp::min(reserved_len.next_power_of_two(), core::isize::MAX as usize);
        if new_cap > self.cap {
            unsafe {
                self.resize(new_cap)?;
            }
        }
        Ok(())
    }
    pub fn push(&mut self, elem: T) -> Result<(), AllocError> {
        unsafe {
            self.reserve(1)?;
            ptr::write(self.ptr.as_ptr().add(self.len), elem);
        }
        self.len += 1; // no need to bounds check, as new len <= cap
        Ok(())
    }
    pub fn extend_from_slice(&mut self, elems: &[T]) -> Result<(), AllocError>
        where T: Copy
    {
        unsafe {
            self.reserve(elems.len())?;
            ptr::copy_nonoverlapping(elems.as_ptr(), self.ptr.as_ptr().add(self.len), elems.len());
        }
        self.len += elems.len(); // no need to bounds check, as new len <= cap
        Ok(())
    }
    pub fn append(&mut self, other: &mut Self) -> Result<(), AllocError> {
        unsafe {
            self.reserve(other.len())?;
            ptr::copy_nonoverlapping(other.as_ptr(), self.ptr.as_ptr().add(self.len), other.len());
        }
        self.len += other.len(); // no need to bounds check, as new len <= cap
        Ok(())
    }
    pub fn truncate(&mut self, len: usize) {
        if len < self.len {
            unsafe {
                let old_len = self.len;
                self.drop_range(len, old_len);
            }
            self.len = len;
        }
    }
    pub fn shrink_to_fit(&mut self) -> Result<(), AllocError> {
        if self.len < self.cap {
            unsafe {
                let new_cap = self.len;
                self.resize(new_cap)?;
            }
        }
        Ok(())
    }
    pub fn capacity(&self) -> usize {
        self.cap
    }
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }
    /// Leaks the inner data. This is safe to drop from C!
    pub fn leak(mut self) -> *mut T {
        let ptr = self.as_mut_ptr();
        mem::forget(self);
        ptr
    }
}
impl<T> Deref for CVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }
}
impl<T> DerefMut for CVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
        }
    }
}
impl<T> Drop for CVec<T> {
    fn drop(&mut self) {
        unsafe {
            let len = self.len;
            self.drop_range(0, len);
        }
    }
}
impl<'a, T> IntoIterator for &'a CVec<T> {
    type Item = <&'a [T] as IntoIterator>::Item;
    type IntoIter = <&'a [T] as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        <&[T]>::into_iter(&*self)
    }
}
impl<'a, T> IntoIterator for &'a mut CVec<T> {
    type Item = <&'a mut [T] as IntoIterator>::Item;
    type IntoIter = <&'a mut [T] as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        <&mut [T]>::into_iter(&mut *self)
    }
}
