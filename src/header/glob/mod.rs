//! `glob.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/glob.h.html>.

use core::ptr;

use alloc::{boxed::Box, vec::Vec};

use crate::{
    c_str::{CStr, CString},
    header::{
        dirent::{closedir, opendir, readdir},
        errno::*,
        fnmatch::{FNM_NOESCAPE, FNM_PERIOD, fnmatch},
        sys_stat::{S_IFDIR, S_IFMT, stat},
    },
    platform::{
        self,
        types::{c_char, c_int, c_uchar, c_void, size_t},
    },
};

// Cause glob() to return on error
pub const GLOB_ERR: c_int = 0x0001;
// Each pathname that is a directory that matches pattern has a slash appended
pub const GLOB_MARK: c_int = 0x0002;
// Do not sort returned pathnames
pub const GLOB_NOSORT: c_int = 0x0004;
// Add gl_offs amount of null pointers to the beginning of `gl_pathv`
pub const GLOB_DOOFFS: c_int = 0x0008;
// If pattern does not match, return a list containing only pattern
pub const GLOB_NOCHECK: c_int = 0x0010;
// Append generated pathnames to those previously obtained
pub const GLOB_APPEND: c_int = 0x0020;
// Disable backslash escaping
pub const GLOB_NOESCAPE: c_int = 0x0040;
// Allow wildcards to match '.' (GNU extension)
pub const GLOB_PERIOD: c_int = 0x0080;

// Attempt to allocate memory failed
pub const GLOB_NOSPACE: c_int = 1;
// Scan was stopped because GLOB_ERR was set or `errfunc` returned non-zero
pub const GLOB_ABORTED: c_int = 2;
// Pattern does not match any existing pathname, and GLOB_NOCHECK was not set
pub const GLOB_NOMATCH: c_int = 3;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/glob.h.html>.
#[derive(Debug)]
#[repr(C)]
pub struct glob_t {
    pub gl_pathc: size_t, // Count of paths matched by pattern (POSIX required field)
    pub gl_offs: size_t,  // Slots to reserve at the beginning of gl_pathv (POSIX required field)
    pub gl_pathv: *mut *mut c_char, // Pointer to list of matched pathnames (POSIX required field)

    // Opaque pointer to allocation data
    __opaque: *mut c_void, // Vec<*mut c_char>
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/glob.html>.
#[linkage = "weak"] // GNU prefers its own glob e.g. in Make
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glob(
    pattern: *const c_char,
    flags: c_int,
    errfunc: Option<unsafe extern "C" fn(epath: *const c_char, eerrno: c_int) -> c_int>,
    pglob: *mut glob_t,
) -> c_int {
    if flags & GLOB_APPEND != GLOB_APPEND {
        unsafe {
            (*pglob).gl_pathc = 0;
            (*pglob).gl_pathv = ptr::null_mut();
            (*pglob).__opaque = ptr::null_mut();
        }
    }

    let glob_expr = unsafe { CStr::from_ptr(pattern) };

    if glob_expr.to_bytes() == b"" {
        return GLOB_NOMATCH;
    }

    let base_path = unsafe {
        CStr::from_bytes_with_nul_unchecked(if glob_expr.to_bytes().first() == Some(&b'/') {
            b"/\0"
        } else {
            b"\0"
        })
    };

    let errfunc = match errfunc {
        Some(f) => f,
        None => default_errfunc,
    };

    // Do the globbing
    let mut results = match inner_glob(&base_path, &glob_expr, flags, errfunc) {
        Ok(res) => res,
        Err(e) => return e,
    };

    // Handle GLOB_NOCHECK and no matches
    if results.is_empty() {
        if flags & GLOB_NOCHECK == GLOB_NOCHECK {
            results.push(glob_expr.to_owned_cstring());
        } else {
            return GLOB_NOMATCH;
        }
    }

    // Handle GLOB_NOSORT
    if flags & GLOB_NOSORT != GLOB_NOSORT {
        results.sort();
    }

    // Set gl_pathc
    if flags & GLOB_APPEND == GLOB_APPEND {
        unsafe {
            (*pglob).gl_pathc += results.len();
        }
    } else {
        unsafe {
            (*pglob).gl_pathc = results.len();
        }
    }

    let mut pathv: Box<Vec<*mut c_char>>;
    if flags & GLOB_APPEND == GLOB_APPEND {
        pathv = unsafe { Box::from_raw((*pglob).__opaque.cast()) };
        pathv.pop(); // Remove NULL from end
    } else {
        pathv = Box::new(Vec::new());
        if flags & GLOB_DOOFFS == GLOB_DOOFFS {
            let gl_offs = unsafe { (*pglob).gl_offs };
            pathv.reserve(gl_offs);
            for _ in 0..gl_offs {
                pathv.push(ptr::null_mut());
            }
        }
    }

    pathv.reserve_exact(results.len() + 1);
    pathv.extend(results.into_iter().map(|s| s.into_raw()));

    pathv.push(ptr::null_mut());

    unsafe {
        (*pglob).gl_pathv = pathv.as_ptr().cast_mut();
        (*pglob).__opaque = Box::into_raw(pathv).cast();
    }

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/glob.html>.
#[linkage = "weak"] // GNU prefers its own glob e.g. in Make
#[unsafe(no_mangle)]
pub unsafe extern "C" fn globfree(pglob: *mut glob_t) {
    // Retake ownership
    if unsafe { !(*pglob).__opaque.is_null() } {
        let pathv: Box<Vec<*mut c_char>> = unsafe { Box::from_raw((*pglob).__opaque.cast()) };
        for (idx, path) in pathv.into_iter().enumerate() {
            if unsafe { idx < (*pglob).gl_offs } {
                continue;
            }
            if !path.is_null() {
                unsafe {
                    drop(CString::from_raw(path));
                }
            }
        }
        unsafe {
            (*pglob).gl_pathv = ptr::null_mut();
        }
    }
}

type GlobErrorFunc = unsafe extern "C" fn(epath: *const c_char, eerrno: c_int) -> c_int;

struct DirEntry {
    name: CString,
    is_dir: bool,
}

unsafe extern "C" fn default_errfunc(epath: *const c_char, eerrno: c_int) -> c_int {
    0
}

fn list_dir(
    path: &CStr,
    errfunc: GlobErrorFunc,
    abort_on_error: bool,
) -> Result<Vec<DirEntry>, c_int> {
    const DT_DIR: c_uchar = 4; // From dirent.h
    const DT_LNK: c_uchar = 10; // From dirent.h

    let old_errno = platform::ERRNO.get();
    let mut results: Vec<DirEntry> = Vec::new();
    let open_path = if path.to_bytes().is_empty() {
        unsafe { &CStr::from_bytes_with_nul_unchecked(b".\0") }
    } else {
        path
    };
    let dir = unsafe { opendir(open_path.as_ptr()) };

    if dir.is_null() {
        let new_errno = platform::ERRNO.get();
        platform::ERRNO.set(old_errno);

        if unsafe { errfunc(path.as_ptr(), new_errno) } != 0 || abort_on_error {
            return Err(GLOB_ABORTED);
        }

        return Ok(results);
    }

    platform::ERRNO.set(0);

    loop {
        let entry = unsafe { readdir(&mut *dir) };
        if entry.is_null() {
            break;
        }

        let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()).to_owned_cstring() };

        if name.as_bytes() == b"." || name.as_bytes() == b".." {
            continue;
        }

        let is_dir: bool = unsafe {
            if (*entry).d_type == DT_DIR {
                true
            } else if (*entry).d_type == DT_LNK {
                // Resolve symbolic link
                let mut full_path = path.to_owned_cstring().into_string().unwrap();
                if !full_path.ends_with('/') {
                    full_path.push('/');
                }
                full_path.push_str(name.to_str().unwrap());
                full_path.push('\0');

                let mut link_info = stat::default();
                if stat(
                    full_path.as_ptr().cast::<c_char>(),
                    ptr::from_mut(&mut link_info),
                ) != 0
                {
                    let errno = platform::ERRNO.get();
                    platform::ERRNO.set(old_errno);
                    if errfunc(full_path.as_ptr().cast::<c_char>(), errno) != 0 || abort_on_error {
                        return Err(GLOB_ABORTED);
                    }
                }
                link_info.st_mode & S_IFMT == S_IFDIR
            } else {
                false
            }
        };

        results.push(DirEntry { name, is_dir });
    }

    // Check if entry == NULL because of an error
    let errno = platform::ERRNO.get();

    unsafe { closedir(Box::from_raw(dir)) };

    // Restore the old errno
    platform::ERRNO.set(old_errno);

    if errno != 0 {
        if unsafe { errfunc(path.as_ptr(), errno) } != 0 || abort_on_error {
            return Err(GLOB_ABORTED);
        }
    }

    Ok(results)
}

fn inner_glob(
    current_dir: &CStr,
    glob_expr: &CStr,
    flags: c_int,
    errfunc: GlobErrorFunc,
) -> Result<Vec<CString>, c_int> {
    let mut pattern: Vec<u8> = Vec::new();

    // Remove any '/' chars at the start of the expression
    let glob_expr = {
        let mut expr = glob_expr.to_bytes_with_nul();
        while expr.first() == Some(&b'/') {
            expr = &expr[1..];
        }
        unsafe { CStr::from_bytes_with_nul_unchecked(expr) }
    };

    // Get the next section of the glob expression (up to non-escaped '/')
    let glob_iter = glob_expr.to_bytes();
    let mut in_bracket = false;
    let mut escaped = false;
    let mut glob_consumed = 0;

    for ch in glob_iter {
        // Don't consume nul
        if ch == &b'\0' {
            break;
        }

        glob_consumed += 1;

        if ch == &b'/' && !escaped {
            break;
        }

        if ch == &b'[' && !escaped {
            in_bracket = true;
        } else if ch == &b']' {
            // '\' is a normal character in brackets so doesn't escape
            in_bracket = false;
        }

        escaped =
            ch == &b'\\' && !in_bracket && !escaped && (flags & GLOB_NOESCAPE != GLOB_NOESCAPE);

        pattern.push(*ch);
    }

    // Needs to be C-string
    pattern.push(b'\0');

    let new_glob_expr = unsafe {
        CStr::from_bytes_with_nul_unchecked(&glob_expr.to_bytes_with_nul()[glob_consumed..])
    };

    // Handle special path sections
    if pattern == b".\0" || pattern == b"..\0" {
        let mut new_dir: Vec<u8> = Vec::new();
        new_dir.extend_from_slice(current_dir.to_bytes());
        new_dir.extend_from_slice(&pattern);
        let new_dir_c = unsafe { CStr::from_bytes_with_nul_unchecked(&new_dir) };
        return inner_glob(&new_dir_c, &new_glob_expr, flags, errfunc);
    }

    let mut fnmatch_flags = 0;
    if flags & GLOB_NOESCAPE == GLOB_NOESCAPE {
        fnmatch_flags |= FNM_NOESCAPE;
    }
    if flags & GLOB_PERIOD == GLOB_PERIOD {
        fnmatch_flags |= FNM_PERIOD;
    }

    let mut matches: Vec<CString> = Vec::new();

    for entry in list_dir(current_dir, errfunc, flags & GLOB_ERR == GLOB_ERR)? {
        // If we still have pattern to match ignore non-directories
        if !new_glob_expr.to_bytes().is_empty() && !entry.is_dir {
            continue;
        }

        let mut path = current_dir.to_bytes().to_vec();

        if path != b"" && !path.ends_with(b"/") {
            path.push(b'/');
        }
        path.extend_from_slice(entry.name.as_bytes());

        if flags & GLOB_MARK == GLOB_MARK && new_glob_expr.to_bytes() == b"" && entry.is_dir {
            path.push(b'/');
        }

        // This shouldn't ever panic, we know the vec has no nul bytes
        let path = CString::new(path).unwrap();

        if unsafe {
            fnmatch(
                pattern.as_ptr().cast::<c_char>(),
                entry.name.as_ptr(),
                fnmatch_flags,
            )
        } == 0
        {
            if entry.is_dir && new_glob_expr.to_bytes() != b"" {
                let new_matches = inner_glob(&CStr::borrow(&path), &new_glob_expr, flags, errfunc)?;
                matches.extend(new_matches);
            } else {
                matches.push(path);
            }
        }
    }

    // It is an error if we don't find a directory when we expect one
    if matches.is_empty() && !new_glob_expr.to_bytes().is_empty() {
        let mut path = current_dir.to_bytes().to_vec();
        path.extend_from_slice(&pattern);
        if unsafe { errfunc(path.as_ptr().cast::<c_char>(), ENOENT) } != 0
            || flags & GLOB_ERR == GLOB_ERR
        {
            return Err(GLOB_ABORTED);
        }
    }

    Ok(matches)
}
