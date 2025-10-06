//! Nul-terminated byte strings.

use core::{marker::PhantomData, ptr::NonNull, str::Utf8Error};

use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
};

use crate::platform::types::{c_char, c_int, wchar_t};

mod private {
    pub trait Sealed {}
}
#[derive(Clone, Copy, Debug)]
pub enum Thin {}

#[derive(Clone, Copy, Debug)]
pub enum Wide {}

impl private::Sealed for Thin {}
impl private::Sealed for Wide {}

pub trait Kind: private::Sealed + Copy + 'static {
    /// c_char or wchar_t
    type C: Copy + 'static;
    // u8 or u32
    type Char: Copy + From<u8> + Into<u32> + PartialEq + 'static;

    const NUL: Self::Char;

    const IS_THIN_NOT_WIDE: bool;

    fn r2c(c: Self::Char) -> Self::C;
    fn c2r(c: Self::C) -> Self::Char;

    fn chars_from_bytes(b: &[u8]) -> Option<&[Self::Char]>;

    unsafe fn strlen(s: *const Self::C) -> usize;
    unsafe fn strchr(s: *const Self::C, c: Self::C) -> *const Self::C;
    unsafe fn strchrnul(s: *const Self::C, c: Self::C) -> *const Self::C;
}
impl Kind for Thin {
    type C = c_char;
    type Char = u8;

    const NUL: Self::Char = 0;
    const IS_THIN_NOT_WIDE: bool = true;

    unsafe fn strlen(s: *const c_char) -> usize {
        crate::header::string::strlen(s)
    }
    unsafe fn strchr(s: *const c_char, c: c_char) -> *const c_char {
        crate::header::string::strchr(s, c.into())
    }
    unsafe fn strchrnul(s: *const c_char, c: c_char) -> *const c_char {
        crate::header::string::strchrnul(s, c.into())
    }
    fn r2c(c: u8) -> c_char {
        c as _
    }
    fn c2r(c: c_char) -> u8 {
        c as _
    }
    fn chars_from_bytes(b: &[u8]) -> Option<&[Self::Char]> {
        Some(b)
    }
}
impl Kind for Wide {
    type C = wchar_t;
    type Char = u32;

    const NUL: Self::Char = 0;
    const IS_THIN_NOT_WIDE: bool = false;

    unsafe fn strlen(s: *const Self::C) -> usize {
        crate::header::wchar::wcslen(s)
    }
    unsafe fn strchr(s: *const Self::C, c: Self::C) -> *const Self::C {
        crate::header::wchar::wcschr(s, c)
    }
    unsafe fn strchrnul(mut s: *const Self::C, c: Self::C) -> *const Self::C {
        // TODO: optimized function
        while s.read() != c && s.read() != 0 {
            s = s.add(1);
        }
        s
    }
    fn r2c(c: Self::Char) -> Self::C {
        c as _
    }
    fn c2r(c: Self::C) -> Self::Char {
        c as _
    }
    fn chars_from_bytes(b: &[u8]) -> Option<&[Self::Char]> {
        None
    }
}

/// Safe wrapper for immutable borrowed C strings, guaranteed to be the same layout as `*const u8`.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NulStr<'a, T: Kind> {
    ptr: NonNull<T::C>,
    _marker: PhantomData<&'a [u8]>,
}
pub type CStr<'a> = NulStr<'a, Thin>;
pub type WStr<'a> = NulStr<'a, Wide>;

impl<'a, T: Kind> NulStr<'a, T> {
    /// Safety
    ///
    /// The ptr must be valid up to and including the first NUL byte from the base ptr.
    pub const unsafe fn from_ptr(ptr: *const T::C) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr as *mut T::C),
            _marker: PhantomData,
        }
    }
    pub unsafe fn from_nullable_ptr(ptr: *const T::C) -> Option<Self> {
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
    pub fn find_get_subslice_or_all(
        self,
        c: impl Into<T::Char>,
    ) -> Result<(&'a [T::Char], Self), (&'a [T::Char], Self)> {
        let c = c.into();

        // SAFETY: strchrnul expects self.as_ptr() to be valid up to and including its last NUL
        // byte
        let found = unsafe { T::strchrnul(self.as_ptr(), T::r2c(c)) };

        // SAFETY: the pointer returned from strchrnul is always a substring of this string, and
        // hence always valid as a CStr.
        let found = unsafe { Self::from_ptr(found) };
        let until = unsafe { self.slice_until_substr(found) };

        if found.first() == T::NUL {
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
    pub unsafe fn slice_until_substr(self, substr: NulStr<'_, T>) -> &'a [T::Char] {
        let index = unsafe {
            // SAFETY: the sub-pointer as returned by strchr must be derived from the same
            // allocation
            substr.as_ptr().offset_from(self.as_ptr()) as usize
        };
        unsafe { core::slice::from_raw_parts(self.as_ptr().cast::<T::Char>(), index) }
    }
    /// Look for the closest occurence of `c`, and if found, split the string into a slice up to
    /// that byte and a `CStr` starting at that byte.
    #[inline]
    pub fn find_get_subslice(self, c: T::Char) -> Option<(&'a [T::Char], Self)> {
        let rest = self.find(c)?;

        // SAFETY: the output of strchr is obviously a substring if it doesn't return NULL
        Some((unsafe { self.slice_until_substr(rest) }, rest))
    }
    /// Look for the closest occurence of `c`, and return a new string starting at that byte if
    /// found.
    #[doc(alias = "strchr")]
    #[doc(alias = "wcschr")]
    #[inline]
    pub fn find(self, c: T::Char) -> Option<Self> {
        unsafe {
            // SAFETY: the only requirement is for self.as_ptr() to be valid up to and including
            // the nearest NUL byte, which this type requires
            let ret = T::strchr(self.as_ptr(), T::r2c(c));
            // SAFETY: strchr must either return NULL (not found) or a substring of self, which can
            // never exceed the nearest NUL byte of self
            Self::from_nullable_ptr(ret)
        }
    }
    // TODO: strrchr, strchrnul wrappers

    #[inline]
    pub fn contains(self, c: T::Char) -> bool {
        self.find(c).is_some()
    }
    #[inline]
    pub fn first(self) -> T::Char {
        unsafe {
            // SAFETY: Self must be valid up to and including its nearest NUL byte, which certainly
            // implies its readable length is nonzero (string is empty if this first byte is 0).
            T::c2r(self.ptr.read())
        }
    }
    #[inline]
    pub fn first_char(self) -> Option<char> {
        char::from_u32(self.first().into())
    }
    /// Same as `split_first` except also requires that the first char be convertible into `char`
    #[inline]
    pub fn split_first_char(self) -> Option<(char, Self)> {
        self.split_first()
            .and_then(|(c, r)| Some((char::from_u32(c.into())?, r)))
    }
    /// Split this string into `Some((first_byte, string_after_that))` or `None` if empty.
    #[inline]
    pub fn split_first(self) -> Option<(T::Char, Self)> {
        if self.first() == T::NUL {
            return None;
        }
        Some((self.first(), unsafe {
            Self::from_ptr(self.as_ptr().add(1))
        }))
    }
    pub fn to_chars_with_nul(self) -> &'a [T::Char] {
        unsafe {
            // SAFETY: The string must be valid at least until (and including) the NUL byte.
            let len = T::strlen(self.ptr.as_ptr());
            core::slice::from_raw_parts(self.ptr.as_ptr().cast(), len + 1)
        }
    }
    pub fn to_chars(self) -> &'a [T::Char] {
        let s = self.to_chars_with_nul();
        &s[..s.len() - 1]
    }
    pub const fn as_ptr(self) -> *const T::C {
        self.ptr.as_ptr()
    }
    pub const unsafe fn from_chars_with_nul_unchecked(chars: &'a [T::Char]) -> Self {
        Self::from_ptr(chars.as_ptr().cast())
    }
    pub fn from_chars_with_nul(chars: &'a [T::Char]) -> Result<Self, FromCharsWithNulError> {
        if chars.last() != Some(&T::NUL) || chars[..chars.len() - 1].contains(&T::NUL) {
            return Err(FromCharsWithNulError);
        }

        Ok(unsafe { Self::from_chars_with_nul_unchecked(chars) })
    }
    pub fn from_chars_until_nul(chars: &'a [T::Char]) -> Result<Self, FromCharsUntilNulError> {
        if !chars.contains(&T::NUL) {
            return Err(FromCharsUntilNulError);
        }

        Ok(unsafe { Self::from_chars_with_nul_unchecked(chars) })
    }
    /// Scan the string to get its length.
    #[doc(alias = "strlen")]
    #[doc(alias = "wcslen")]
    pub fn len(self) -> usize {
        self.to_chars().len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.first() == T::NUL
    }
}
impl<'a> CStr<'a> {
    pub fn to_owned_cstring(self) -> CString {
        CString::from(unsafe { core::ffi::CStr::from_ptr(self.ptr.as_ptr()) })
    }
    pub fn borrow(string: &'a CString) -> Self {
        unsafe { Self::from_ptr(string.as_ptr()) }
    }
    #[inline]
    pub fn to_bytes(self) -> &'a [u8] {
        self.to_chars()
    }
    #[inline]
    pub fn to_bytes_with_nul(self) -> &'a [u8] {
        self.to_chars_with_nul()
    }
    pub fn to_str(self) -> Result<&'a str, Utf8Error> {
        core::str::from_utf8(self.to_bytes())
    }
    pub fn to_string_lossy(self) -> Cow<'a, str> {
        String::from_utf8_lossy(self.to_bytes())
    }
    #[inline]
    pub const unsafe fn from_bytes_with_nul_unchecked(bytes: &'a [u8]) -> Self {
        Self::from_chars_with_nul_unchecked(bytes)
    }
    #[inline]
    pub fn from_bytes_with_nul(bytes: &'a [u8]) -> Result<Self, FromCharsWithNulError> {
        Self::from_chars_with_nul(bytes)
    }
    #[inline]
    pub fn from_bytes_until_nul(bytes: &'a [u8]) -> Result<Self, FromCharsUntilNulError> {
        Self::from_chars_until_nul(bytes)
    }
}

unsafe impl<T: Kind> Send for NulStr<'_, T> {}
unsafe impl<T: Kind> Sync for NulStr<'_, T> {}

impl From<&core::ffi::CStr> for CStr<'_> {
    fn from(s: &core::ffi::CStr) -> Self {
        // SAFETY:
        // * We can assume that `s` is valid because the caller should have upheld its
        // safety concerns when constructing it.
        unsafe { Self::from_ptr(s.as_ptr()) }
    }
}

#[derive(Debug)]
pub struct FromCharsWithNulError;

#[derive(Debug)]
pub struct FromCharsUntilNulError;

pub use alloc::ffi::CString;
