use core::{marker::PhantomData, ptr::NonNull, str::Utf8Error};

use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
};

use crate::{header::string::strlen, platform::types::c_char};

/// C string wrapper, guaranteed to be
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
    pub fn as_ptr(self) -> *const c_char {
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
}

unsafe impl Send for CStr<'_> {}
unsafe impl Sync for CStr<'_> {}

#[derive(Debug)]
pub struct FromBytesWithNulError;

#[derive(Debug)]
pub struct FromBytesUntilNulError;

pub use alloc::ffi::CString;
