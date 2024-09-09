use core::ptr;

use crate::{
    c_str::CStr,
    fs::File,
    header::{
        fcntl::{O_CLOEXEC, O_RDWR},
        limits::PASS_MAX,
        termios,
    },
    io::{self, Read, Write},
};

use crate::platform::types::*;

fn getpass_rs(prompt: CStr, passbuff: &mut [u8]) -> Result<*mut c_char, io::Error> {
    let mut f = File::open(c_str!("/dev/tty"), O_RDWR | O_CLOEXEC)?;

    let mut term = termios::termios::default();

    unsafe {
        termios::tcgetattr(f.fd, &mut term as *mut termios::termios);
    }

    let old_term = term.clone();

    term.c_iflag &= !(termios::IGNCR | termios::INLCR) as u32;
    term.c_iflag |= termios::ICRNL as u32;
    term.c_lflag &= !(termios::ECHO | termios::ISIG) as u32;
    term.c_lflag |= termios::ICANON as u32;

    unsafe {
        termios::tcsetattr(f.fd, termios::TCSAFLUSH, &term as *const termios::termios);
    }

    f.write(&prompt.to_bytes())?;
    f.flush()?;

    let mut len = f.read(passbuff)?;

    if len > 0 {
        if passbuff[len - 1] == b'\n' || passbuff.len() == len {
            len -= 1;
        }
    }

    passbuff[len] = 0;

    unsafe {
        termios::tcsetattr(
            f.fd,
            termios::TCSAFLUSH,
            &old_term as *const termios::termios,
        );
    }

    f.write(b"\n")?;
    f.flush()?;

    Ok(passbuff.as_mut_ptr() as *mut c_char)
}

#[no_mangle]
pub unsafe extern "C" fn getpass(prompt: *const c_char) -> *mut c_char {
    static mut PASSBUFF: [u8; PASS_MAX] = [0; PASS_MAX];

    unsafe { getpass_rs(CStr::from_ptr(prompt), &mut PASSBUFF).unwrap_or(ptr::null_mut()) }
}
