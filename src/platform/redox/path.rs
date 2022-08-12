use syscall::error::*;
use syscall::flag::*;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::sync::Mutex;

// TODO: Define in syscall
const PATH_MAX: usize = 4096;

/// Make a relative path absolute.
///
/// Given a cwd of "scheme:/path", this his function will turn "foo" into "scheme:/path/foo".
/// "/foo" will turn into "scheme:/foo". "bar:/foo" will be used directly, as it is already
/// absolute
pub fn canonicalize_using_cwd(cwd_opt: Option<&str>, path: &str) -> Option<String> {
    let mut canon = if path.find(':').is_none() {
        let cwd = cwd_opt?;
        let path_start = cwd.find(':')? + 1;

        let mut canon = if !path.starts_with('/') {
            let mut c = cwd.to_owned();
            if ! c.ends_with('/') {
                c.push('/');
            }
            c
        } else {
            cwd[..path_start].to_owned()
        };

        canon.push_str(&path);
        canon
    } else {
        path.to_owned()
    };

    // NOTE: assumes the scheme does not include anything like "../" or "./"
    let mut result = {
        let parts = canon.split('/')
            .rev()
            .scan(0, |nskip, part| {
                if part == "." {
                    Some(None)
                } else if part == ".." {
                    *nskip += 1;
                    Some(None)
                } else if *nskip > 0 {
                        *nskip -= 1;
                        Some(None)
                } else {
                    Some(Some(part))
                }
            })
            .filter_map(|x| x)
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>();
        parts
            .iter()
            .rev()
            .fold(String::new(), |mut string, &part| {
                string.push_str(part);
                string.push('/');
                string
            })
    };
    result.pop(); // remove extra '/'

    // replace with the root of the scheme if it's empty
    Some(if result.is_empty() {
        let pos = canon.find(':')
                        .map_or(canon.len(), |p| p + 1);
        canon.truncate(pos);
        canon
    } else {
        result
    })
}

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
    let _siglock = SignalMask::lock();
    let mut cwd_guard = CWD.lock();

    let canonicalized = canonicalize_using_cwd(cwd_guard.as_deref(), path).ok_or(Error::new(ENOENT))?;

    *cwd_guard = Some(canonicalized.into_boxed_str());

    // TODO: Check that the dir exists and is a directory.

    Ok(())
    
}

pub fn clone_cwd() -> Option<Box<str>> {
    let _siglock = SignalMask::lock();
    CWD.lock().clone()
}

// TODO: MaybeUninit
pub fn getcwd(buf: &mut [u8]) -> Option<usize> {
    let _siglock = SignalMask::lock();
    let cwd_guard = CWD.lock();
    let cwd = cwd_guard.as_deref().unwrap_or("").as_bytes();

    // But is already checked not to be empty.
    if buf.len() - 1 < cwd.len() { return None; }

    buf[..cwd.len()].copy_from_slice(&cwd);
    buf[cwd.len()..].fill(0_u8);

    Some(cwd.len())
}

// TODO: Move cwd from kernel to libc. It is passed via auxiliary vectors.
pub fn canonicalize(path: &str) -> Result<String> {
    let _siglock = SignalMask::lock();
    let cwd = CWD.lock();
    canonicalize_using_cwd(cwd.as_deref(), path).ok_or(Error::new(ENOENT))
}

// TODO: arraystring?
static CWD: Mutex<Option<Box<str>>> = Mutex::new(None);

pub fn setcwd_manual(cwd: Box<str>) {
    let _siglock = SignalMask::lock();
    *CWD.lock() = Some(cwd);
}

/// RAII guard able to magically fix signal unsafety, by disabling signals during a critical
/// section.
pub struct SignalMask {
    oldset: [u64; 2],
}
impl SignalMask {
    pub fn lock() -> Self {
        let mut oldset = [0; 2];
        syscall::sigprocmask(syscall::SIG_SETMASK, Some(&[!0, !0]), Some(&mut oldset)).expect("failed to run sigprocmask");
        Self { oldset }
    }
}
impl Drop for SignalMask {
    fn drop(&mut self) {
        let _ = syscall::sigprocmask(syscall::SIG_SETMASK, Some(&self.oldset), None);
    }
}
