use alloc::{borrow::ToOwned, boxed::Box, string::String, vec::Vec};
use redox_rt::signal::tmp_disable_signals;
use syscall::{data::Stat, error::*, flag::*};

use super::{libcscheme, FdGuard};
use crate::sync::Mutex;

pub use redox_path::canonicalize_using_cwd;

// TODO: Define in syscall
const PATH_MAX: usize = 4096;

// XXX: chdir is not marked thread-safe (MT-safe) by POSIX. But on Linux it is simply run as a
// syscall and is therefore atomic, which is presumably why Rust's libstd doesn't synchronize
// access to this.
//
// https://internals.rust-lang.org/t/synchronized-ffi-access-to-posix-environment-variable-functions/15475
//
// chdir is however signal-safe, forbidding the use of locks. We therefore call sigprocmask before
// and after acquiring the locks. (TODO: ArcSwap? That will need to be ported to no_std first,
// though).
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

// TODO: MaybeUninit
pub fn getcwd(buf: &mut [u8]) -> Option<usize> {
    let _siglock = tmp_disable_signals();
    let cwd_guard = CWD.lock();
    let cwd = cwd_guard.as_deref().unwrap_or("").as_bytes();

    // But is already checked not to be empty.
    if buf.len() - 1 < cwd.len() {
        return None;
    }

    buf[..cwd.len()].copy_from_slice(&cwd);
    buf[cwd.len()..].fill(0_u8);

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

                let bytes_read = syscall::read(*resolve_fd, &mut resolve_buf)?;
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
