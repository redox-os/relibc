use crate::platform::types::c_long;

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/getopt.3.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syscall(sysno: c_long, mut args: ...) -> c_long {
    let a1 = unsafe { args.arg::<usize>() };
    let a2 = unsafe { args.arg::<usize>() };
    let a3 = unsafe { args.arg::<usize>() };
    let a4 = unsafe { args.arg::<usize>() };
    let a5 = unsafe { args.arg::<usize>() };
    let a6 = unsafe { args.arg::<usize>() };

    (unsafe { sc::syscall6(sysno as usize, a1, a2, a3, a4, a5, a6) }) as c_long
}
