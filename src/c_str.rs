//! Nul-terminated byte strings.

use core::{marker::PhantomData, ptr::NonNull, str::Utf8Error};

use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
};

use crate::{
    header::string::{strchr, strchrnul, strlen},
    platform::types::c_char,
};

/// Safe wrapper for immutable borrowed C strings, guaranteed to be the same layout as `*const u8`.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct CStr<'a> {
    ptr: NonNull<c_char>,
    _marker: PhantomData<&'a [u8]>,
}

impl<'a> CStr<'a> {
    /// Safety
    ///
    /// The ptr must be valid up to and including the first NUL byte from the base ptr.
    pub const unsafe fn from_ptr(ptr: *const c_char) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr as *mut c_char),
            _marker: PhantomData,
        }
    }
    pub unsafe fn from_nullable_ptr(ptr: *const c_char) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self::from_ptr(ptr))
        }
    }
    /// Look for the closest occurence of `c`, and if found, split the string into a slice up to
    /// that byte and a `CStr` starting at that byte.
    #[inline]
    #[doc(alias = "strchrnul")]
    pub fn find_get_subslice_or_all(self, c: u8) -> Result<(&'a [u8], Self), (&'a [u8], Self)> {
        // SAFETY: strchrnul expects self.as_ptr() to be valid up to and including its last NUL
        // byte
        let found = unsafe { strchrnul(self.as_ptr(), c.into()) };

        // SAFETY: the pointer returned from strchrnul is always a substring of this string, and
        // hence always valid as a CStr.
        let found = unsafe { Self::from_ptr(found) };
        let until = unsafe { self.slice_until_substr(found) };

        if found.first() == 0 {
            // The character was not found, and we got the end of the string instead.
            Err((until, found))
        } else {
            Ok((until, found))
        }
    }
    /// # Safety
    ///
    /// `substr` must be contained within `self`
    #[inline]
    pub unsafe fn slice_until_substr(self, substr: CStr<'_>) -> &'a [u8] {
        let index = unsafe {
            // SAFETY: the sub-pointer as returned by strchr must be derived from the same
            // allocation
            substr.as_ptr().offset_from(self.as_ptr()) as usize
        };
        unsafe { core::slice::from_raw_parts(self.as_ptr().cast::<u8>(), index) }
    }
    /// Look for the closest occurence of `c`, and if found, split the string into a slice up to
    /// that byte and a `CStr` starting at that byte.
    #[inline]
    pub fn find_get_subslice(self, c: u8) -> Option<(&'a [u8], Self)> {
        let rest = self.find(c)?;

        // SAFETY: the output of strchr is obviously a substring if it doesn't return NULL
        Some((unsafe { self.slice_until_substr(rest) }, rest))
    }
    /// Look for the closest occurence of `c`, and return a new string starting at that byte if
    /// found.
    #[doc(alias = "strchr")]
    #[inline]
    pub fn find(self, c: u8) -> Option<Self> {
        unsafe {
            // SAFETY: the only requirement is for self.as_ptr() to be valid up to and including
            // the nearest NUL byte, which this type requires
            let ret = strchr(self.as_ptr(), c.into());
            // SAFETY: strchr must either return NULL (not found) or a substring of self, which can
            // never exceed the nearest NUL byte of self
            Self::from_nullable_ptr(ret)
        }
    }
    // TODO: strrchr, strchrnul wrappers

    #[inline]
    pub fn contains(self, c: u8) -> bool {
        self.find(c).is_some()
    }
    #[inline]
    pub fn first(self) -> u8 {
        unsafe {
            // SAFETY: Self must be valid up to and including its nearest NUL byte, which certainly
            // implies its readable length is nonzero (string is empty if this first byte is 0).
            self.ptr.read() as u8
        }
    }
    /// Split this string into `Some((first_byte, string_after_that))` or `None` if empty.
    #[inline]
    pub fn split_first(self) -> Option<(u8, CStr<'a>)> {
        if self.first() == 0 {
            return None;
        }
        Some((self.first(), unsafe {
            CStr::from_ptr(self.as_ptr().add(1))
        }))
    }
    pub fn to_bytes_with_nul(self) -> &'a [u8] {
        unsafe {
            // SAFETY: The string must be valid at least until (and including) the NUL byte.
            let len = strlen(self.ptr.as_ptr());
            core::slice::from_raw_parts(self.ptr.as_ptr().cast(), len + 1)
        }
    }
    pub fn to_bytes(self) -> &'a [u8] {
        let s = self.to_bytes_with_nul();
        &s[..s.len() - 1]
    }
    pub fn to_str(self) -> Result<&'a str, Utf8Error> {
        core::str::from_utf8(self.to_bytes())
    }
    pub fn to_string_lossy(self) -> Cow<'a, str> {
        String::from_utf8_lossy(self.to_bytes())
    }
    pub const fn as_ptr(self) -> *const c_char {
        self.ptr.as_ptr()
    }
    pub const unsafe fn from_bytes_with_nul_unchecked(bytes: &'a [u8]) -> Self {
        Self::from_ptr(bytes.as_ptr().cast())
    }
    pub fn from_bytes_with_nul(bytes: &'a [u8]) -> Result<Self, FromBytesWithNulError> {
        if bytes.last() != Some(&b'\0') || bytes[..bytes.len() - 1].contains(&b'\0') {
            return Err(FromBytesWithNulError);
        }

        Ok(unsafe { Self::from_bytes_with_nul_unchecked(bytes) })
    }
    pub fn from_bytes_until_nul(bytes: &'a [u8]) -> Result<Self, FromBytesUntilNulError> {
        if !bytes.contains(&b'\0') {
            return Err(FromBytesUntilNulError);
        }

        Ok(unsafe { Self::from_bytes_with_nul_unchecked(bytes) })
    }
    pub fn to_owned_cstring(self) -> CString {
        CString::from(unsafe { core::ffi::CStr::from_ptr(self.ptr.as_ptr()) })
    }
    pub fn borrow(string: &'a CString) -> Self {
        unsafe { Self::from_ptr(string.as_ptr()) }
    }
    #[doc(alias = "strlen")]
    pub fn count_bytes(&self) -> usize {
        self.to_bytes().len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.first() == 0
    }
}

unsafe impl Send for CStr<'_> {}
unsafe impl Sync for CStr<'_> {}

impl From<&core::ffi::CStr> for CStr<'_> {
    fn from(s: &core::ffi::CStr) -> Self {
        // SAFETY:
        // * We can assume that `s` is valid because the caller should have upheld its
        // safety concerns when constructing it.
        unsafe { Self::from_ptr(s.as_ptr()) }
    }
}

#[derive(Debug)]
pub struct FromBytesWithNulError;

#[derive(Debug)]
pub struct FromBytesUntilNulError;

pub use alloc::ffi::CString;
