//! pty.h implementation, not POSIX specified

use core::{mem, ptr, slice};

use crate::{
    header::{fcntl, limits, pthread, signal, sys_ioctl, sys_wait, termios, unistd, utmp},
    platform::{self, types::*},
};

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
mod imp;

#[no_mangle]
pub unsafe extern "C" fn openpty(
    amaster: *mut c_int,
    aslave: *mut c_int,
    namep: *mut c_char,
    termp: *const termios::termios,
    winp: *const sys_ioctl::winsize,
) -> c_int {
    let mut tmp_name = [0; limits::PATH_MAX];
    let name = if !namep.is_null() {
        slice::from_raw_parts_mut(namep as *mut u8, limits::PATH_MAX)
    } else {
        &mut tmp_name
    };

    let (master, slave) = match imp::openpty(name) {
        Ok(ok) => ok,
        Err(()) => return -1,
    };

    if !termp.is_null() {
        termios::tcsetattr(slave, termios::TCSANOW, termp);
    }

    if !winp.is_null() {
        sys_ioctl::ioctl(slave, sys_ioctl::TIOCSWINSZ, winp as *mut c_void);
    }

    *amaster = master;
    *aslave = slave;

    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn forkpty(
    pm: *mut c_int,
    name: *mut c_char,
    tio: *const termios::termios,
    ws: *const sys_ioctl::winsize,
) -> c_int {
    let mut m = 0;
    let mut s = 0;
    let mut ec = 0;
    let mut p: [c_int; 2] = [0; 2];
    let mut cs = 0;
    let mut pid = -1;
    let mut set = signal::sigset_t::default();
    let mut oldset = signal::sigset_t::default();

    if openpty(&mut m, &mut s, name, tio, ws) < 0 {
        return -1;
    }

    signal::sigfillset(&mut set);
    signal::pthread_sigmask(signal::SIG_BLOCK, &mut set, &mut oldset);
    pthread::pthread_setcancelstate(pthread::PTHREAD_CANCEL_DISABLE, &mut cs);

    if unistd::pipe2(p.as_mut_ptr(), fcntl::O_CLOEXEC) != 0 {
        unistd::close(s);
    } else {
        pid = unistd::fork();
        if pid == 0 {
            unistd::close(m);
            unistd::close(p[0]);
            if utmp::login_tty(s) != 0 {
                unistd::write(
                    p[1],
                    platform::ERRNO.as_ptr().cast(),
                    mem::size_of_val(&platform::ERRNO),
                );
                unistd::_exit(127);
            }
            unistd::close(p[1]);
            pthread::pthread_setcancelstate(cs, ptr::null_mut());
            signal::pthread_sigmask(signal::SIG_SETMASK, &mut oldset, ptr::null_mut());
            return 0;
        }

        unistd::close(s);
        unistd::close(p[1]);

        if unistd::read(
            p[0],
            &mut ec as *mut c_int as *mut c_void,
            mem::size_of::<c_int>(),
        ) > 0
        {
            let mut status = 0;
            sys_wait::waitpid(pid, &mut status, 0);
            pid = -1;
            platform::ERRNO.set(ec);
        }
        unistd::close(p[0]);
    }
    if pid > 0 {
        *pm = m;
    } else {
        unistd::close(m);
    }
    pthread::pthread_setcancelstate(cs, ptr::null_mut());
    signal::pthread_sigmask(signal::SIG_SETMASK, &mut oldset, ptr::null_mut());
    pid
}
