//! Utilities to help use Rust iterators on C strings.

use core::{iter::Iterator, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};
use core::mem;

use crate::platform::types::*;

/// A minimal alternative to the `Zero` trait from num-traits, for use in
/// `NulTerminated`.
///
/// May be replaced with the one from num-traits at a later time if so
/// desired.
pub unsafe trait Zero {
    fn is_zero(&self) -> bool;
}

unsafe impl Zero for c_char {
    fn is_zero(&self) -> bool {
        self == &0
    }
}

unsafe impl Zero for wchar_t {
    fn is_zero(&self) -> bool {
        self == &0
    }
}

/// An iterator over a nul-terminated buffer.
///
/// This is intended to allow safe, ergonomic iteration over C-style byte and
/// wide strings without first having to read through the string and construct
/// a slice. Assuming the safety requirements are upheld when constructing the
/// iterator, it allows for string iteration in safe Rust.
pub struct NulTerminated<'a, T: Zero> {
    ptr: NonNull<T>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: Zero> Iterator for NulTerminated<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: the caller is required to ensure a valid pointer to a
        // 0-terminated buffer is provided, and the zero-check below ensures
        // that iteration and pointer increments will stop in time.
        let val_ref = unsafe { self.ptr.as_ref() };
        if val_ref.is_zero() {
            None
        } else {
            // SAFETY: the caller is required to provide a 0-terminated
            // buffer, and this point will only be reached if the next element
            // is at most the terminating 0.
            self.ptr = unsafe { self.ptr.add(1) };
            Some(val_ref)
        }
    }
}

impl<'a, T: Zero> NulTerminated<'a, T> {
    /// Constructs a new iterator, starting at `ptr`, yielding elements of
    /// type `&T` up to (but not including) the terminating nul.
    ///
    /// The iterator returns `None` after the terminating nul has been
    /// encountered.
    ///
    /// # Safety
    /// The provided pointer must be a valid pointer to a buffer of contiguous
    /// elements of type `T`, and the value 0 must be present within the
    /// buffer at or after `ptr` (not necessarily at the end). The buffer must
    /// not be written to for the lifetime of the iterator.
    pub unsafe fn new(ptr: *const T) -> Self {
        NulTerminated {
            // NonNull can only wrap only *mut pointers...
            ptr: NonNull::new(ptr.cast_mut()).unwrap(),
            phantom: PhantomData,
        }
    }


    // Faster len implementation. Checks 8 characters at a time
    // TODO Maybe able to implement with normal iterator trait? 
    pub fn len(&self) -> usize {
        let mut length = 0;
        let mut c_ptr = self.ptr.as_ptr();
        // Must first align on long word boundary
        while (c_ptr as usize) % mem::size_of::<u64>() != 0 {
            let val_ref = unsafe { self.ptr.as_ref() };
            if val_ref.is_zero() {
                return length;
            }
            c_ptr = unsafe { c_ptr.add(1) };
            length += 1;
        }
        // Uses himagic and lomagic for NULL termination check
        // Look at glibc implementation:
        // https://github.com/lattera/glibc/blob/master/string/strlen.c
        // Works Similarly
        let mut himagic: u64 = 0x80808080;
        let mut lomagic: u64 = 0x01010101;
        himagic = ((himagic << 16) << 16) | himagic;
        lomagic = ((lomagic << 16) << 16) | lomagic;
        let mut long_word_ptr = self.ptr.as_ptr() as *const u64;
        let mut long_word = unsafe { *long_word_ptr };
        while long_word.wrapping_sub(lomagic) & !long_word & himagic == 0 {
            long_word_ptr = unsafe { long_word_ptr.add(1) };
            long_word = unsafe { *long_word_ptr };
            length += 8;

        }
        let mut cp = long_word_ptr as *const c_char;
        for i in 0..8 {
            let val_ref = unsafe { cp.as_ref().unwrap() };
            if val_ref.is_zero() {
                length += i;
                break;
            }
            cp = unsafe { cp.add(1) };
        }
        return length;
    }
}

/// A zipped iterator mapping an input iterator to an "out" pointer.
///
/// This is intended to allow safe, iterative writing to an "out pointer".
/// Special care needs to be taken to avoid creating references past the end
/// of the output buffer, thus the output is zipped with an "input" iterator
/// to ensure up-front control of the range of memory on which we create
/// references.
pub struct SrcDstPtrIter<'a, I: Iterator, U: Copy> {
    src_iter: I,
    dst_ptr: *mut U,
    phantom: PhantomData<&'a mut U>,
}

impl<'a, I: Iterator, U: Copy> Iterator for SrcDstPtrIter<'a, I, U> {
    type Item = (I::Item, &'a mut MaybeUninit<U>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(src_item) = self.src_iter.next() {
            let old_dst_ptr = self.dst_ptr;

            // SAFETY: due to the caller requirements on `I` upon
            // construction, the new pointer here may be either valid to turn
            // into a reference or "one past the end". The latter is okay as
            // long as it is only represented as a raw pointer.
            self.dst_ptr = unsafe { self.dst_ptr.add(1) };

            // SAFETY: self.dst_ptr may point "one past the end", but the
            // caller is required upon construction to ensure that `I` does
            // not over-iterate, and thus old_dst_ptr is always okay to
            // dereference.
            let out_mut_ref = unsafe { old_dst_ptr.as_uninit_mut() }.unwrap();

            Some((src_item, out_mut_ref))
        } else {
            None
        }
    }
}

impl<'a, I: Iterator, U: Copy> SrcDstPtrIter<'a, I, U> {
    /// Constructs a new iterator of "zipped" input and output.
    ///
    /// The caller must provide an "input" iterator `I` and an "out pointer"
    /// `ptr`. Assuming `I` has item type `T`, the new iterator will have
    /// `type Item = (T, &mut MaybeUninit<U>)`.
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to a writable buffer of contiguous (but
    /// possibly uninitialized) elements of type `U`. The caller must ensure
    /// that `I` does not return `Some` any more times than there are elements
    /// in the output buffer. The caller must ensure that the iterator has
    /// exclusive access to that buffer for the entire lifetime of the
    /// iterator.
    pub unsafe fn new(iter: I, ptr: *mut U) -> Self {
        SrcDstPtrIter {
            src_iter: iter,
            dst_ptr: ptr,
            phantom: PhantomData,
        }
    }
}
