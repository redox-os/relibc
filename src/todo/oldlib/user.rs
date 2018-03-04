use core::ptr::null;
use libc::{uid_t, gid_t, c_char};
use ::types::passwd;
use syscall::{Error, ENOENT};
use alloc::string::{String, ToString};
use core::str::FromStr;

static mut PASSWD: passwd = passwd {
    pw_name: null(),
    pw_passwd: null(),
    pw_uid: 0,
    pw_gid: 0,
    pw_gecos: null(),
    pw_dir: null(),
    pw_shell: null()
};

static mut PW_NAME: Option<String> = None;
static mut PW_PASSWD: Option<String> = None;
static mut PW_GECOS: Option<String> = None;
static mut PW_DIR: Option<String> = None;
static mut PW_SHELL: Option<String> = None;

macro_rules! try_opt {
    ($e:expr) =>(
        match $e {
            Some(v) => v,
            None => return None,
        }
    )
}

unsafe fn parse_passwd(pwd: &str) -> Option<*const passwd> {
    let mut parts = pwd.split(';');

    let name = try_opt!(parts.next()).to_string() + "\0";
    let passwd = try_opt!(parts.next()).to_string() + "\0";
    let uid = if let Ok(uid) = uid_t::from_str(try_opt!(parts.next())) {
        uid
    } else {
        return None
    };
    let gid = if let Ok(gid) = gid_t::from_str(try_opt!(parts.next())) {
        gid
    } else {
        return None
    };
    let gecos = try_opt!(parts.next()).to_string() + "\0";
    let dir = try_opt!(parts.next()).to_string() + "\0";
    let shell = try_opt!(parts.next()).to_string() + "\0";

    PASSWD = passwd {
        pw_name: name.as_ptr() as *const c_char,
        pw_passwd: passwd.as_ptr() as *const c_char,
        pw_uid: uid,
        pw_gid: gid,
        pw_gecos: gecos.as_ptr() as *const c_char,
        pw_dir: dir.as_ptr() as *const c_char,
        pw_shell: shell.as_ptr() as *const c_char
    };

    PW_NAME = Some(name);
    PW_PASSWD = Some(passwd);
    PW_GECOS = Some(gecos);
    PW_DIR = Some(dir);
    PW_SHELL = Some(shell);

    Some(&PASSWD as *const _)
}

libc_fn!(unsafe getpwnam(name: *const c_char) -> Result<*const passwd> {
    if let Ok(passwd_string) = String::from_utf8(::file_read_all("/etc/passwd")?) {
        for line in passwd_string.lines() {
            if line.split(';').nth(0).map(str::as_bytes) == Some(::cstr_to_slice(name)) {
                if let Some(pass) = parse_passwd(line) {
                    return Ok(pass);
                }
            }
        }
    }
    Err(Error::new(ENOENT))
});

libc_fn!(unsafe getpwuid(uid: uid_t) -> Result<*const passwd> {
    if let Ok(passwd_string) = String::from_utf8(::file_read_all("/etc/passwd")?) {
        for line in passwd_string.lines() {
            if line.split(';').nth(2).map(uid_t::from_str) == Some(Ok(uid)) {
                if let Some(pass) = parse_passwd(line) {
                    return Ok(pass);
                }
            }
        }
    }
    Err(Error::new(ENOENT))
});
