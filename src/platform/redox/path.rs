use alloc::{borrow::Cow, string::ToString};
use arrayvec::ArrayString;
use core::{
    ffi::c_int,
    str::{self, FromStr},
};
use redox_path::{RedoxReference, RedoxStr};
use redox_rt::{proc::FdGuardUpper, signal::tmp_disable_signals};
use syscall::{data::Stat, error::*, flag::*};

use super::{FdGuard, Pal, Sys, libcscheme};
use crate::{
    c_str::CStr,
    error::Errno,
    fs::File,
    header::{fcntl, limits, sys_file},
    out::Out,
    sync::rwlock::{ReadGuard, RwLock},
};

pub use redox_path::RedoxPath;

// POSIX states chdir is both thread-safe and signal-safe. Thus we need to synchronize access to CWD, but at the
// same time forbid signal handlers from running in the meantime, to avoid reentrant deadlock.
pub fn chdir(path: RedoxStr<'_>) -> Result<()> {
    let _siglock = tmp_disable_signals();
    let mut cwd_guard = CWD.write();
    let (path, fd) = match path {
        RedoxStr::Absolute(path) => {
            let RedoxPath::Standard(path) = path.to_standard_canon() else {
                // actually unreachable
                return Err(Error::new(EINVAL));
            };
            let fd = FdGuard::open(&path, O_STAT)?.to_upper()?;
            (RedoxPath::from_reference(path).unwrap(), fd)
        }
        RedoxStr::Relative(redox_reference) => {
            let Some(cwd) = cwd_guard.as_ref() else {
                return Err(Error::new(ENOENT));
            };
            let fd = cwd
                .fd
                .openat(redox_reference.as_ref(), O_STAT, 0)?
                .to_upper()?;
            (cwd.redox.canonicalize_as_cwd(redox_reference.into()), fd)
        }
    };
    let mut stat = Stat::default();
    if fd.fstat(&mut stat).is_err() || (stat.st_mode & MODE_TYPE) != MODE_DIR {
        return Err(Error::new(ENOTDIR));
    }
    let cow: Cow<'_, str> = path.into();
    let path = to_cwd_path(&cow)?;
    let redox = RedoxPath::from_absolute(cow.into_owned()).unwrap();
    *cwd_guard = Some(Cwd { path, redox, fd });
    Ok(())
}

pub fn fchdir(fd: c_int) -> Result<()> {
    let mut buf = CwdPath::zero_filled();
    unsafe {
        // SAFETY: Sys::fpath is using str::from_utf8 already
        let res = Sys::fpath(fd, buf.as_bytes_mut())?;
        buf.set_len(res);
    }
    let fd = FdGuard::new(syscall::fcntl(
        fd as usize,
        syscall::F_DUPFD,
        syscall::UPPER_FDTBL_TAG,
    )?)
    .to_upper()
    .unwrap();
    set_cwd_manual(buf, fd)?;
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
pub fn current_dir() -> Result<ReadGuard<'static, Option<Cwd<'static>>>> {
    let _siglock = tmp_disable_signals();
    let guard = CWD.read();

    if guard.as_ref().is_none() {
        return Err(Error::new(ENOENT));
    }

    Ok(guard)
}

pub type CwdPath = ArrayString<{ limits::PATH_MAX }>;

pub struct Cwd<'a> {
    pub path: CwdPath,
    pub redox: RedoxPath<'a>,
    pub fd: FdGuardUpper,
}

static CWD: RwLock<Option<Cwd<'_>>> = RwLock::new(None);

pub fn to_cwd_path(path: &str) -> Result<CwdPath> {
    ArrayString::from_str(&path).or(Err(Error::new(ENAMETOOLONG)))
}

pub fn set_cwd_manual(path: CwdPath, fd: FdGuardUpper) -> Result<()> {
    let Some(redox) = RedoxPath::from_absolute(path.to_string()) else {
        return Err(Error::new(EBADF));
    };
    let _siglock = tmp_disable_signals();
    *CWD.write() = Some(Cwd { path, redox, fd });
    Ok(())
}

pub fn clone_cwd() -> Option<CwdPath> {
    let _siglock = tmp_disable_signals();
    CWD.read().as_ref().map(|cwd| cwd.path.clone())
}

fn open_absolute(path: &str, flags: usize) -> Result<usize> {
    if path.starts_with(libcscheme::LIBC_SCHEME) {
        libcscheme::open(path, flags)
    } else {
        redox_rt::sys::open(path, flags)
    }
}

// Read symlink content
fn read_link_content<'a, 'b>(
    dirfd: Option<&FdGuard>,
    path: &'a str,
    is_relative: bool,
) -> Result<RedoxStr<'b>> {
    let resolve_flags = O_CLOEXEC | O_SYMLINK | O_RDONLY;

    let fd = match (is_relative, dirfd) {
        (false, _) => FdGuard::open(path, resolve_flags)?,
        (true, None) => current_dir()?
            .as_ref()
            .unwrap()
            .fd
            .openat(path, resolve_flags, 0)?,
        (true, Some(dirfd)) => dirfd.openat(path, resolve_flags, 0)?,
    };

    let mut resolve_buf = [0_u8; limits::PATH_MAX + 1];
    let count = fd.read(&mut resolve_buf)?;
    if count == resolve_buf.len() {
        return Err(Error::new(ENAMETOOLONG));
    }

    // If the symbolic link path is non-UTF8, it cannot be opened, and is thus
    // considered a "dangling symbolic link".
    let path = core::str::from_utf8(&resolve_buf[..count]).map_err(|_| Error::new(ENOENT))?;
    RedoxStr::new(path.to_string()).ok_or(Error::new(EBADF))
}

/// Resolve symlink and open as a fd. Requires `current_path_string` as canonicalized.
fn resolve_sym_links<'a>(mut current_path_string: RedoxPath<'a>, flags: usize) -> Result<usize> {
    // Sym resolve loop
    for _ in 0..limits::SYMLOOP_MAX {
        let dirname = current_path_string.dirname();
        let cow: Cow<'_, str> = current_path_string.into();
        match open_absolute(&cow, flags) {
            Ok(fd) => return Ok(fd),
            Err(e) if e == Error::new(EXDEV) => {
                // dirfd is None because it's canonicalized
                let link_target = read_link_content(None, &cow, false)?;
                current_path_string = dirname.canonicalize_as_cwd(link_target);
            }
            Err(e) => return Err(e),
        }
    }

    Err(Error::new(ELOOP))
}

// TODO: Move to redox-rt, or maybe part of it?
pub fn openat(dirfd: c_int, path: RedoxStr<'_>, flags: usize) -> Result<usize> {
    if path.is_empty() && flags as i32 & fcntl::AT_EMPTY_PATH != fcntl::AT_EMPTY_PATH {
        return Err(Error::new(ENOENT));
    }

    if !path.is_relative() {
        return open(path, flags);
    }

    let _siglock = tmp_disable_signals();
    let fcntl_flags = flags & syscall::O_FCNTL_MASK;
    let path: Cow<'_, str> = path.into();

    // First try
    let initial_res = if dirfd == fcntl::AT_FDCWD {
        current_dir()?
            .as_ref()
            .unwrap()
            .fd
            .openat(&path, flags, fcntl_flags)
            .map(|fd: FdGuard| fd.take())
    } else {
        redox_rt::sys::openat(dirfd as usize, &path, flags, fcntl_flags)
    };

    match initial_res {
        Ok(fd) => Ok(fd),
        Err(e) if e == Error::new(EXDEV) => {
            let (link_path, dirfd) = if dirfd == fcntl::AT_FDCWD {
                let link_path = read_link_content(None, &path, true);
                (link_path, dirfd)
            } else {
                let fd = FdGuard::new(dirfd as usize);
                let link_path = read_link_content(Some(&fd), &path, true);
                (link_path, fd.take() as i32)
            };
            let link_path = openat2_path(dirfd, link_path?, 0)?;
            resolve_sym_links(link_path, flags)
        }
        Err(e) => Err(e),
    }
}

// TODO: Move to redox-rt, or maybe part of it?
pub fn open(path: RedoxStr<'_>, flags: usize) -> Result<usize> {
    if path.is_empty() {
        return Err(Error::new(ENOENT));
    }

    let is_relative = path.is_relative();
    let _siglock = tmp_disable_signals();
    let path: Cow<'_, str> = path.into();

    // First try
    let initial_res = if is_relative {
        let fcntl_flags = flags & syscall::O_FCNTL_MASK;
        current_dir()?
            .as_ref()
            .unwrap()
            .fd
            .openat(&path, flags, fcntl_flags)
            .map(|fd: FdGuard| fd.take())
    } else {
        open_absolute(&path, flags)
    };

    match initial_res {
        Ok(fd) => Ok(fd),
        Err(e) if e == Error::new(EXDEV) => {
            let link_path = read_link_content(None, &path, true);
            let link_path = openat2_path(fcntl::AT_FDCWD, link_path?, 0)?;
            resolve_sym_links(link_path, flags)
        }
        Err(e) => Err(e),
    }
}

/// Return the directory (or scheme path) and the relative socket path to the scheme
pub fn dir_path_and_fd_path(
    socket_path: RedoxStr<'_>,
) -> Result<(RedoxPath<'_>, RedoxReference<'_>)> {
    let _siglock = tmp_disable_signals();
    let cwd_guard = CWD.read();
    let cwd_path = cwd_guard.as_ref().map(|c| &c.redox);
    let redox_path = openat2_path(fcntl::AT_FDCWD, socket_path, 0)?;

    let (scheme, ref_path) = redox_path.as_parts().ok_or(Error::new(EINVAL))?;
    if ref_path.as_ref().is_empty() {
        return Err(Error::new(EINVAL));
    }
    let ref_path = RedoxReference::new(ref_path.to_string()).unwrap();
    let dir_to_open = ref_path.clone().dirname();
    Ok((scheme.canonicalize_as_scheme(dir_to_open.into()), ref_path))
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
pub(super) fn openat2_path(
    dirfd: c_int,
    path: RedoxStr<'_>,
    at_flags: c_int,
) -> Result<RedoxPath<'_>, Errno> {
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
        let redox_path = match path {
            RedoxStr::Absolute(redox_path) => redox_path,
            RedoxStr::Relative(redox_reference) => {
                let cwd_guard = CWD.read();
                let cwd_path = cwd_guard.as_ref().map(|c| &c.redox);
                let cwd_path = cwd_path.ok_or(Error::new(EINVAL))?;
                cwd_path.canonicalize_as_cwd(redox_reference.into())
            }
        };
        Ok(redox_path)
    } else {
        let mut buf = [0; limits::PATH_MAX];
        let len = Sys::fpath(dirfd, &mut buf)?;
        // SAFETY: fpath checks then copies valid UTF8.
        let dir = unsafe { str::from_utf8_unchecked(&buf[..len]) };
        let dir = RedoxPath::from_absolute(dir).ok_or(Errno(EBADF))?;
        Ok(dir.canonicalize_as_cwd(path))
    }
}

fn at_flags_to_open_flags(at_flags: c_int) -> c_int {
    let mut out: c_int = 0;
    if at_flags & fcntl::AT_SYMLINK_NOFOLLOW == fcntl::AT_SYMLINK_NOFOLLOW {
        out |= fcntl::O_NOFOLLOW | fcntl::O_PATH;
    }
    out
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
    path: CStr,
    at_flags: c_int,
    oflags: c_int,
) -> Result<File, Errno> {
    // Translate at flags into open flags; openat will do this on its own most likely.
    let oflags = at_flags_to_open_flags(at_flags) | fcntl::O_CLOEXEC | oflags;
    File::openat(dirfd, path, oflags)
}
