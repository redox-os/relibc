use alloc::{
    boxed::Box,
    collections::btree_set::BTreeSet,
    ffi::CString,
    string::{String, ToString},
    vec::Vec,
};
use core::{ffi::c_int, str};
use redox_rt::{proc::FdGuardUpper, signal::tmp_disable_signals};
use syscall::{data::Stat, error::*, flag::*};

use super::{FdGuard, Pal, Sys, libcscheme};
use crate::{
    error::Errno,
    fs::File,
    header::{fcntl, limits, sys_file},
    out::Out,
    sync::rwlock::{ReadGuard, RwLock},
};

pub use redox_path::{RedoxPath, canonicalize_using_cwd};

pub fn normalize_path<'a>(path: &'a str) -> Option<(bool, String)> {
    let absolute = match RedoxPath::from_absolute(path) {
        Some(absolute) => absolute,
        None => return Some((true, partially_canonical(path)?)),
    };
    let canonical = absolute.canonical()?;
    Some((false, canonical.to_string()))
}

pub fn normalize_scheme_rooted_path<'a>(path: &'a str) -> Option<(bool, String)> {
    let absolute = match RedoxPath::from_absolute(path) {
        Some(absolute) => absolute,
        None => return Some((true, partially_canonical(path)?)),
    };
    let canonical = absolute.canonical()?;
    Some((false, scheme_rooted_path(&canonical.to_string()).ok()?))
}

fn partially_canonical(path: &str) -> Option<String> {
    let mut stack = Vec::new();
    let mut paths_to_check = BTreeSet::new();
    let mut up_counts = 0;

    for part in path.split('/') {
        if part.is_empty() || part == "." {
            continue;
        } else if part == ".." {
            if let Some(part) = stack.pop() {
                paths_to_check.insert(
                    core::iter::repeat("..")
                        .take(up_counts)
                        .chain(stack.clone())
                        .chain(core::iter::once(part))
                        .collect::<Vec<_>>()
                        .join("/"),
                );
            } else {
                up_counts += 1;
            }
        } else {
            stack.push(part);
        }
    }

    for path in paths_to_check {
        let _ = current_dir()
            .ok()?
            .as_ref()
            .unwrap()
            .fd
            .openat(&path, O_STAT, 0)
            .ok()?;
    }

    Some(
        core::iter::repeat("..")
            .take(up_counts)
            .chain(stack)
            .collect::<Vec<_>>()
            .join("/"),
    )
}

// TODO: Define in syscall
const PATH_MAX: usize = 4096;

// POSIX states chdir is both thread-safe and signal-safe. Thus we need to synchronize access to CWD, but at the
// same time forbid signal handlers from running in the meantime, to avoid reentrant deadlock.
pub fn chdir(path: &str) -> Result<()> {
    let _siglock = tmp_disable_signals();
    let mut cwd_guard = CWD.write();
    let (is_relative, path) = normalize_path(path).ok_or(Error::new(ENOENT))?;
    if is_relative {
        let fd = current_dir()?
            .as_ref()
            .unwrap()
            .fd
            .openat(&path, O_STAT, 0)?
            .to_upper()
            .unwrap();
        let mut stat = Stat::default();
        if fd.fstat(&mut stat).is_err() || (stat.st_mode & MODE_TYPE) != MODE_DIR {
            return Err(Error::new(ENOTDIR));
        }

        let canon = canonicalize_using_cwd(cwd_guard.as_ref().map(|c| c.path.as_ref()), &path)
            .ok_or(Error::new(ENOENT))?;
        *cwd_guard = Some(Cwd {
            path: canon.into_boxed_str(),
            fd,
        });
    } else {
        let canon_with_scheme = scheme_rooted_path(&path)?;

        let fd = FdGuard::open(&canon_with_scheme, O_STAT)?
            .to_upper()
            .unwrap();
        let mut stat = Stat::default();
        if fd.fstat(&mut stat).is_err() || (stat.st_mode & MODE_TYPE) != MODE_DIR {
            return Err(Error::new(ENOTDIR));
        }

        *cwd_guard = Some(Cwd {
            path: path.into_boxed_str(),
            fd,
        });
    }

    Ok(())
}

pub fn fchdir(fd: c_int) -> Result<()> {
    let mut buf = [0; PATH_MAX];
    let res = syscall::fpath(fd as usize, &mut buf)?;

    let path = core::str::from_utf8(&buf[..res])
        .map_err(|_| Errno(EINVAL))?
        .to_string();
    let fd = FdGuard::new(syscall::fcntl(
        fd as usize,
        syscall::F_DUPFD,
        syscall::UPPER_FDTBL_TAG,
    )?)
    .to_upper()
    .unwrap();
    set_cwd_manual(path.into_boxed_str(), fd);
    Ok(())
}

// getcwd is similarly both thread-safe and signal-safe.
pub fn getcwd(mut buf: Out<[u8]>) -> Result<usize> {
    let _siglock = tmp_disable_signals();
    let guard = CWD.read();
    let cwd = guard.as_ref().ok_or(Error::new(ENOENT))?;
    let path_bytes = cwd.path.as_bytes();

    let [mut before, mut after] = buf
        .split_at_checked(path_bytes.len())
        .ok_or(Error::new(ERANGE))?;

    before.copy_from_slice(path_bytes);
    after.zero();

    Ok(path_bytes.len())
}

// Get Cwd object
pub fn current_dir() -> Result<ReadGuard<'static, Option<Cwd>>> {
    let guard = CWD.read();

    if guard.as_ref().is_none() {
        return Err(Error::new(ENOENT));
    }

    Ok(guard)
}

fn scheme_rooted_path(path: &str) -> Result<String> {
    let standard_scheme = path == "/scheme" || path.starts_with("/scheme/");
    let legacy_scheme = path
        .split("/")
        .next()
        .map(|c| c.contains(":"))
        .unwrap_or(false);

    Ok(if standard_scheme || legacy_scheme {
        path.to_string()
    } else {
        let mut result = format!("/scheme/file{}", path);

        // Trim trailing / to keep path canonical.
        if result.as_bytes().last() == Some(&b'/') {
            result.pop();
        }

        result
    })
}

// TODO: How much of this logic should be in redox-path?
fn canonicalize_with_cwd_internal(cwd: Option<&str>, path: &str) -> Result<String> {
    let path = canonicalize_using_cwd(cwd, path).ok_or(Error::new(ENOENT))?;
    scheme_rooted_path(&path)
}

pub fn canonicalize(path: &str) -> Result<String> {
    let _siglock = tmp_disable_signals();
    let cwd_guard = CWD.read();
    canonicalize_with_cwd_internal(cwd_guard.as_ref().map(|c| c.path.as_ref()), path)
}

pub struct Cwd {
    pub path: Box<str>,
    pub fd: FdGuardUpper,
}

// TODO: arraystring?
static CWD: RwLock<Option<Cwd>> = RwLock::new(None);

pub fn set_cwd_manual(path: Box<str>, fd: FdGuardUpper) {
    let _siglock = tmp_disable_signals();
    *CWD.write() = Some(Cwd { path, fd });
}

pub fn clone_cwd() -> Option<Box<str>> {
    let _siglock = tmp_disable_signals();
    CWD.read().as_ref().map(|cwd| cwd.path.clone())
}

// TODO: Move to redox-rt, or maybe part of it?
pub fn open(path: &str, flags: usize) -> Result<usize> {
    // TODO: SYMLOOP_MAX
    const MAX_LEVEL: usize = 64;

    if path == "" {
        return Err(Error::new(ENOENT));
    }

    let open_absolute = |path: &str| -> Result<usize> {
        if path.starts_with(libcscheme::LIBC_SCHEME) {
            libcscheme::open(path, flags)
        } else {
            redox_rt::sys::open(path, flags)
        }
    };

    let read_link_content = |path: &str, is_relative: bool| -> Result<String> {
        let mut resolve_buf = [0_u8; 4096];
        let resolve_flags = O_CLOEXEC | O_SYMLINK | O_RDONLY;

        let fd = if is_relative {
            let fcntl_flags = resolve_flags & syscall::O_FCNTL_MASK;
            current_dir()?
                .as_ref()
                .unwrap()
                .fd
                .openat(path, resolve_flags, fcntl_flags)?
        } else {
            FdGuard::open(path, resolve_flags)?
        };

        let count = fd.read(&mut resolve_buf)?;
        if count == resolve_buf.len() {
            // TODO: make resolve_buf PATH_MAX + 1 bytes?
            return Err(Error::new(ENAMETOOLONG));
        }
        // If the symbolic link path is non-UTF8, it cannot be opened, and is thus
        // considered a "dangling symbolic link".
        core::str::from_utf8(&resolve_buf[..count])
            .map_err(|_| Error::new(ENOENT))
            .map(|s| s.to_string())
    };

    let calc_next_abs_path = |current_abs: &str, link_target: &str| -> Result<String> {
        let _siglock = tmp_disable_signals();
        let parent = get_parent_path(current_abs).ok_or(Error::new(ENOENT))?;

        canonicalize_using_cwd(Some(parent), link_target).ok_or(Error::new(ENOENT))
    };

    let (is_relative, canon) = normalize_scheme_rooted_path(path).ok_or(Error::new(ENOENT))?;
    let mut current_path_string: String;

    // First try
    let initial_res = if is_relative {
        let fcntl_flags = flags & syscall::O_FCNTL_MASK;
        current_dir()?
            .as_ref()
            .unwrap()
            .fd
            .openat(&canon, flags, fcntl_flags)
            .map(|fd: FdGuard| fd.take())
    } else {
        open_absolute(&canon)
    };

    match initial_res {
        Ok(fd) => return Ok(fd),
        Err(e) if e == Error::new(EXDEV) => {
            let link_target = read_link_content(&canon, is_relative)?;

            let _siglock = tmp_disable_signals();
            let cwd_guard = CWD.read();
            let current_abs =
                canonicalize_using_cwd(cwd_guard.as_ref().map(|c| c.path.as_ref()), &canon)
                    .ok_or(Error::new(ENOENT))?;

            current_path_string = calc_next_abs_path(&current_abs, &link_target)?;
        }
        Err(e) => return Err(e),
    }

    // Sym reolve loop
    for _ in 0..(MAX_LEVEL - 1) {
        match open_absolute(&current_path_string) {
            Ok(fd) => return Ok(fd),
            Err(e) if e == Error::new(EXDEV) => {
                let link_target = read_link_content(&current_path_string, false)?;

                current_path_string = calc_next_abs_path(&current_path_string, &link_target)?;
            }
            Err(e) => return Err(e),
        }
    }

    Err(Error::new(ELOOP))
}

pub fn dir_path_and_fd_path(socket_path: &str) -> Result<(String, String)> {
    let _siglock = tmp_disable_signals();
    let cwd_guard = CWD.read();
    let cwd_path = cwd_guard.as_ref().map(|c| c.path.as_ref());

    let full_path = canonicalize_with_cwd_internal(cwd_path, socket_path)?;

    let redox_path = RedoxPath::from_absolute(&full_path).ok_or(Error::new(EINVAL))?;
    let (_, ref_path) = redox_path.as_parts().ok_or(Error::new(EINVAL))?;
    if ref_path.as_ref().is_empty() {
        return Err(Error::new(EINVAL));
    }
    if redox_path.is_default_scheme() {
        let dir_to_open = String::from(get_parent_path(&full_path).ok_or(Error::new(EINVAL))?);
        Ok((dir_to_open, ref_path.as_ref().to_string()))
    } else {
        let full_path = canonicalize_with_cwd_internal(cwd_path, ref_path.as_ref())?;
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

pub struct FileLock(c_int);

impl FileLock {
    pub fn lock(fd: c_int, op: c_int) -> Result<Self> {
        if op & sys_file::LOCK_SH | sys_file::LOCK_EX == 0 {
            return Err(Error::new(EINVAL));
        }

        Sys::flock(fd, op)?;
        Ok(Self(fd))
    }

    pub fn unlock(self) -> Result<()> {
        Sys::flock(self.0, sys_file::LOCK_UN).map_err(Into::into)
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let fd = self.0;
        self.0 = -1;
        let _ = Sys::flock(self.0, sys_file::LOCK_UN);
    }
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
        let len = match getcwd(Out::from_mut(&mut buf)) {
            Ok(len) => len,
            Err(e) if e.errno == ERANGE => {
                return Err(Errno(ENAMETOOLONG));
            }
            Err(e) => return Err(Errno(e.errno)),
        };
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
