use core::ptr;

use alloc::vec::Vec;

use crate::{
    fs::File,
    header::{
        fcntl::{O_CLOEXEC, O_RDWR},
        limits::PASS_MAX,
        stdio, termios,
    },
    io::{self, Read},
};

use crate::platform::types::*;

#[derive(Debug)]
enum Error {
    Io(io::Error),
    CannotConvertFd,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

unsafe fn getpass_rs(prompt: *const c_char) -> Result<*mut c_char, Error> {
    let mut f = File::open(c_str!("/dev/tty"), O_RDWR | O_CLOEXEC)?;

    let mut term = termios::termios::default();
    termios::tcgetattr(f.fd, &mut term as *mut termios::termios);
    let old_term = term.clone();

    term.c_iflag &= !(termios::IGNCR | termios::INLCR) as u32;
    term.c_iflag |= termios::ICRNL as u32;
    term.c_lflag &= !(termios::ECHO | termios::ISIG) as u32;
    term.c_lflag |= termios::ICANON as u32;

    let cfile = stdio::fdopen(f.fd, c_str!("w+e").as_ptr());

    if cfile.is_null() {
        return Err(Error::CannotConvertFd);
    }

    termios::tcsetattr(f.fd, termios::TCSAFLUSH, &term as *const termios::termios);
    stdio::fputs(prompt, cfile);
    stdio::fflush(cfile);

    let mut buff = Vec::new();
    let mut len = f.read(&mut buff)?;

    static mut PASSBUFF: [c_char; PASS_MAX] = [0; PASS_MAX];

    for (dst, src) in PASSBUFF.iter_mut().zip(&buff) {
        *dst = *src as c_char;
    }

    if len > 0 {
        if PASSBUFF[len - 1] == b'\n' as c_char || PASSBUFF.len() == len {
            len -= 1;
        }
    }

    PASSBUFF[len] = 0;

    termios::tcsetattr(
        f.fd,
        termios::TCSAFLUSH,
        &old_term as *const termios::termios,
    );
    stdio::fputs(c_str!("\n").as_ptr(), cfile);

    Ok(PASSBUFF.as_mut_ptr())
}

#[no_mangle]
pub extern "C" fn getpass(prompt: *const c_char) -> *mut c_char {
    unsafe { getpass_rs(prompt).unwrap_or(ptr::null_mut()) }
}
