use crate::{
    error::Errno,
    header::errno::{EINVAL, ENOENT},
    out::Out,
    platform::{Pal, Sys},
};

// Stub for call used in exit
#[unsafe(no_mangle)]
pub extern "C" fn pthread_terminate() {}

mod epoll;

#[test]
fn access() {
    use crate::header::unistd;

    //TODO: create test files
    assert_eq!(
        Sys::access(c"not a file!".into(), unistd::F_OK),
        Err(Errno(ENOENT))
    );
    assert_eq!(Sys::access(c"README.md".into(), unistd::F_OK), Ok(()));
    assert_eq!(Sys::access(c"README.md".into(), unistd::R_OK), Ok(()));
    assert_eq!(Sys::access(c"README.md".into(), unistd::W_OK), Ok(()));
    assert_eq!(
        Sys::access(c"README.md".into(), unistd::X_OK),
        Err(Errno(EINVAL))
    );
}

// FIXME: Test needs rewriting so it compiles
#[test]
fn brk() {
    use core::ptr;

    let current = unsafe { Sys::brk(ptr::null_mut()) }.unwrap();
    assert_ne!(current, ptr::null_mut());

    let request = unsafe { current.add(4096) };
    let next = unsafe { Sys::brk(request) }.unwrap();
    assert_eq!(next, request);
}

#[test]
fn chdir() {
    //TODO: create test files
    assert_eq!(Sys::chdir(c"src".into()), Ok(()));
}

//TODO: chmod

//TODO: chown

#[test]
fn clock_gettime() {
    use crate::header::time;

    {
        let mut timespec = time::timespec {
            tv_sec: -1,
            tv_nsec: -1,
        };
        assert_eq!(
            Sys::clock_gettime(time::CLOCK_REALTIME, Out::from_mut(&mut timespec)),
            Ok(())
        );
        assert_ne!(timespec.tv_sec, -1);
        assert_ne!(timespec.tv_nsec, -1);
    }

    {
        let mut timespec = time::timespec {
            tv_sec: -1,
            tv_nsec: -1,
        };
        assert_eq!(
            Sys::clock_gettime(time::CLOCK_MONOTONIC, Out::from_mut(&mut timespec)),
            Ok(())
        );
        assert_ne!(timespec.tv_sec, -1);
        assert_ne!(timespec.tv_nsec, -1);
    }
}

//TDOO: everything else

#[test]
fn getrandom() {
    use crate::platform::types::ssize_t;

    let mut arrays = [[0; 32]; 32];
    for i in 1..arrays.len() {
        assert_eq!(Sys::getrandom(&mut arrays[i], 0).unwrap(), arrays[i].len());

        for j in 0..arrays.len() {
            if i != j {
                assert_ne!(&arrays[i], &arrays[j]);
            }
        }
    }
}
