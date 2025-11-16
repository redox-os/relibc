use alloc::{
    borrow::ToOwned,
    boxed::Box,
    ffi::CString,
    string::{String, ToString},
    vec::Vec,
};
use core::{ffi::c_int, str};
use redox_rt::signal::tmp_disable_signals;
use syscall::{data::Stat, error::*, flag::*};

use super::{FdGuard, Pal, Sys, libcscheme};
use crate::{
    error::Errno,
    fs::File,
    header::{fcntl, limits},
    out::Out,
    sync::Mutex,
};

pub use redox_path::{RedoxPath, canonicalize_using_cwd};

// TODO: Define in syscall
const PATH_MAX: usize = 4096;

// POSIX states chdir is both thread-safe and signal-safe. Thus we need to synchronize access to CWD, but at the
// same time forbid signal handlers from running in the meantime, to avoid reentrant deadlock.
pub fn chdir(path: &str) -> Result<()> {
    let _siglock = tmp_disable_signals();
    let mut cwd_guard = CWD.lock();

    let canon = canonicalize_using_cwd(cwd_guard.as_deref(), path).ok_or(Error::new(ENOENT))?;
    let canon_with_scheme = canonicalize_with_cwd_internal(cwd_guard.as_deref(), path)?;

    let fd = syscall::open(&canon_with_scheme, O_STAT | O_CLOEXEC)?;
    let mut stat = Stat::default();
    if syscall::fstat(fd, &mut stat).is_err() || (stat.st_mode & MODE_TYPE) != MODE_DIR {
        return Err(Error::new(ENOTDIR));
    }
    let _ = syscall::close(fd);

    *cwd_guard = Some(canon.into_boxed_str());

    Ok(())
}

// getcwd is similarly both thread-safe and signal-safe.
pub fn getcwd(mut buf: Out<[u8]>) -> Option<usize> {
    let _siglock = tmp_disable_signals();
    let cwd_guard = CWD.lock();
    let cwd = cwd_guard.as_deref().unwrap_or("").as_bytes();

    let [mut before, mut after] = buf.split_at_checked(cwd.len())?;

    before.copy_from_slice(&cwd);
    after.zero();

    Some(cwd.len())
}

/// Sets the default scheme
///
/// By default absolute paths resolve to /scheme/file, calling this function
/// allows a different scheme to be used as the root. This property is inherited
/// by child processes.
///
/// Resets CWD to /.
pub fn set_default_scheme(scheme: &str) -> Result<()> {
    let _siglock = tmp_disable_signals();
    let mut cwd_guard = CWD.lock();
    let mut default_scheme_guard = DEFAULT_SCHEME.lock();

    *cwd_guard = None;
    *default_scheme_guard = Some(scheme.into());

    Ok(())
}

// TODO: How much of this logic should be in redox-path?
fn canonicalize_with_cwd_internal(cwd: Option<&str>, path: &str) -> Result<String> {
    let path = canonicalize_using_cwd(cwd, path).ok_or(Error::new(ENOENT))?;

    let standard_scheme = path == "/scheme" || path.starts_with("/scheme/");
    let legacy_scheme = path
        .split("/")
        .next()
        .map(|c| c.contains(":"))
        .unwrap_or(false);

    Ok(if standard_scheme || legacy_scheme {
        path
    } else {
        let mut default_scheme_guard = DEFAULT_SCHEME.lock();
        let default_scheme = default_scheme_guard.get_or_insert_with(|| Box::from("file"));
        let mut result = format!("/scheme/{}{}", default_scheme, path);

        // Trim trailing / to keep path canonical.
        if result.as_bytes().last() == Some(&b'/') {
            result.pop();
        }

        result
    })
}

pub fn canonicalize(path: &str) -> Result<String> {
    let _siglock = tmp_disable_signals();
    let cwd_guard = CWD.lock();
    canonicalize_with_cwd_internal(cwd_guard.as_deref(), path)
}

// TODO: arraystring?
static CWD: Mutex<Option<Box<str>>> = Mutex::new(None);
static DEFAULT_SCHEME: Mutex<Option<Box<str>>> = Mutex::new(None);

pub fn set_cwd_manual(cwd: Box<str>) {
    let _siglock = tmp_disable_signals();
    *CWD.lock() = Some(cwd);
}

pub fn set_default_scheme_manual(scheme: Box<str>) {
    let _siglock = tmp_disable_signals();
    *DEFAULT_SCHEME.lock() = Some(scheme)
}

pub fn clone_cwd() -> Option<Box<str>> {
    let _siglock = tmp_disable_signals();
    CWD.lock().clone()
}

pub fn clone_default_scheme() -> Option<Box<str>> {
    let _siglock = tmp_disable_signals();
    DEFAULT_SCHEME.lock().clone()
}

// TODO: Move to redox-rt, or maybe part of it?
pub fn open(path: &str, flags: usize) -> Result<usize> {
    // TODO: SYMLOOP_MAX
    const MAX_LEVEL: usize = 64;

    let mut resolve_buf = [0_u8; 4096];
    let mut path = path;

    for _ in 0..MAX_LEVEL {
        let canon = canonicalize_with_cwd_internal(CWD.lock().as_deref(), path)?;

        let open_res = if canon.starts_with(libcscheme::LIBC_SCHEME) {
            libcscheme::open(&canon, flags)
        } else {
            syscall::open(&*canon, flags)
        };

        match open_res {
            Ok(fd) => return Ok(fd),
            Err(error) if error == Error::new(EXDEV) => {
                let resolve_flags = O_CLOEXEC | O_SYMLINK | O_RDONLY;
                let resolve_fd = FdGuard::new(syscall::open(&*canon, resolve_flags)?);

                let bytes_read = resolve_fd.read(&mut resolve_buf)?;
                // TODO: make resolve_buf PATH_MAX + 1 bytes?
                if bytes_read == resolve_buf.len() {
                    return Err(Error::new(ENAMETOOLONG));
                }

                // If the symbolic link path is non-UTF8, it cannot be opened, and is thus
                // considered a "dangling symbolic link".
                path = core::str::from_utf8(&resolve_buf[..bytes_read])
                    .map_err(|_| Error::new(ENOENT))?;
            }
            Err(other_error) => return Err(other_error),
        }
    }
    Err(Error::new(ELOOP))
}

pub fn dir_path_and_fd_path(socket_path: &str) -> Result<(String, String)> {
    let _siglock = tmp_disable_signals();
    let cwd_guard = CWD.lock();

    let full_path = canonicalize_with_cwd_internal(cwd_guard.as_deref(), socket_path)?;

    let redox_path = RedoxPath::from_absolute(&full_path).ok_or(Error::new(EINVAL))?;
    let (_, mut ref_path) = redox_path.as_parts().ok_or(Error::new(EINVAL))?;
    if ref_path.as_ref().is_empty() {
        return Err(Error::new(EINVAL));
    }
    if redox_path.is_default_scheme() {
        let dir_to_open = String::from(get_parent_path(&full_path).ok_or(Error::new(EINVAL))?);
        Ok((dir_to_open, ref_path.as_ref().to_string()))
    } else {
        let full_path = canonicalize_with_cwd_internal(cwd_guard.as_deref(), ref_path.as_ref())?;
        let redox_path = RedoxPath::from_absolute(&full_path).ok_or(Error::new(EINVAL))?;
        let (_, path) = redox_path.as_parts().ok_or(Error::new(EINVAL))?;
        let dir_to_open = String::from(get_parent_path(&full_path).ok_or(Error::new(EINVAL))?);
        Ok((dir_to_open, path.as_ref().to_string()))
    }
}

fn get_parent_path(path: &str) -> Option<&str> {
    path.rfind('/').and_then(|index| {
        if index == 0 {
            // Path is something like "/file.txt" or the root "/".
            // The parent is the root directory "/".
            Some("/")
        } else {
            // Path is something like "/a/b/c.txt".
            // Take the slice from the beginning up to the last '/'.
            Some(&path[..index])
        }
    })
}

/// Resolve `path` under `dirfd`.
///
/// See [`openat2`] for more information.
pub(super) fn openat2_path(dirfd: c_int, path: &str, at_flags: c_int) -> Result<String, Errno> {
    // Ideally, the function calling this fn would check AT_EMPTY_PATH and just call fstat or
    // whatever with the fd.
    if path.is_empty() && at_flags & fcntl::AT_EMPTY_PATH != fcntl::AT_EMPTY_PATH {
        return Err(Errno(ENOENT));
    }

    // Absolute paths are passed without processing unless RESOLVE_BENEATH is used.
    // canonicalize_using_cwd checks that path is absolute so a third branch that does so here
    // isn't needed.
    if dirfd == fcntl::AT_FDCWD {
        // The special constant AT_FDCWD indicates that we should use the cwd.
        let mut buf = [0; limits::PATH_MAX];
        let len = getcwd(Out::from_mut(&mut buf)).ok_or(Errno(ENAMETOOLONG))?;
        // SAFETY: Redox's cwd is stored as a str.
        let cwd = unsafe { str::from_utf8_unchecked(&buf[..len]) };

        canonicalize_using_cwd(Some(cwd), path).ok_or(Errno(EBADF))
    } else {
        let mut buf = [0; limits::PATH_MAX];
        let len = Sys::fpath(dirfd, &mut buf)?;
        // SAFETY: fpath checks then copies valid UTF8.
        let dir = unsafe { str::from_utf8_unchecked(&buf[..len]) };

        canonicalize_using_cwd(Some(dir), path).ok_or(Errno(EBADF))
    }
}

/// Canonicalize and open `path` with respect to `dirfd`.
///
/// This unexported openat2 is similar to the Linux syscall but with a different interface. The
/// naming is mostly for convenience - it's not a drop in replacement for openat2.
///
/// # Arguments
/// * `dirfd` is a directory descriptor to which `path` is resolved.
/// * `path` is a relative or absolute path. Relative paths are resolved in relation to `dirfd`
/// while absolute paths skip `dirfd`.
/// * `at_flags` constrains how `path` is resolved.
/// * `oflags` are flags that are passed to open.
///
/// # Constants
/// `at_flags`:
/// * AT_EMPTY_PATH returns the path at `dirfd` itself if `path` is empty. If `path` is not
/// empty, it's resolved w.r.t `dirfd` like normal.
///
/// `dirfd`:
/// `AT_FDCWD` is a special constant for `dirfd` that resolves `path` under the current working
/// directory.
pub(super) fn openat2(
    dirfd: c_int,
    path: &str,
    at_flags: c_int,
    oflags: c_int,
) -> Result<File, Errno> {
    let path = openat2_path(dirfd, path, at_flags)?;
    let path = CString::new(path).map_err(|_| Errno(ENOENT))?;

    // Translate at flags into open flags; openat will do this on its own most likely.
    let oflags = if at_flags & fcntl::AT_SYMLINK_NOFOLLOW == fcntl::AT_SYMLINK_NOFOLLOW {
        fcntl::O_CLOEXEC | fcntl::O_NOFOLLOW | fcntl::O_PATH | fcntl::O_SYMLINK | oflags
    } else {
        fcntl::O_CLOEXEC | oflags
    };

    // TODO:
    // * Switch open to openat.
    File::open(path.as_c_str().into(), oflags)
}
