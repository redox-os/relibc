use syscall;
use syscall::data::TimeSpec;
use syscall::error::{Error, EFAULT};
use libc::{c_int, c_void};
use types::timeval;

libc_fn!(unsafe clock_gettime(clk_id: c_int, tp: *mut TimeSpec) -> Result<c_int> {
    Ok(syscall::clock_gettime(clk_id as usize, &mut *tp)? as c_int)
});

libc_fn!(unsafe _gettimeofday(tv: *mut timeval, _tz: *const c_void) -> Result<c_int> {
    if !tv.is_null() {
        let mut tp = TimeSpec::default();
        syscall::clock_gettime(syscall::flag::CLOCK_REALTIME, &mut tp)?;
        (*tv).tv_sec = tp.tv_sec;
        (*tv).tv_usec = (tp.tv_nsec / 1000) as i64;
        Ok(0)
    } else {
        Err(Error::new(EFAULT))
    }
});

libc_fn!(unsafe nanosleep(req: *const TimeSpec, rem: *mut TimeSpec) -> Result<c_int> {
    Ok(syscall::nanosleep(&*req, &mut *rem)? as c_int)
});
