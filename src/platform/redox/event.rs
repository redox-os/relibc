use core::mem::size_of;

use crate::{
    header::{
        bits_sigset_t::sigset_t,
        fcntl::{AT_FDCWD, O_CLOEXEC, O_CREAT, O_RDWR},
        time::timespec,
    },
    out::Out,
};

use super::libredox::RawResult;

use redox_path::RedoxStr;
use syscall::{EINVAL, Error, Result};

pub fn queue_create(flags: u32) -> Result<usize, Error> {
    if flags != 0 {
        return Err(Error::new(EINVAL));
    }
    // TODO: use redox_rt::sys::open, and why O_CREAT?
    Ok(super::libredox::openat(
        AT_FDCWD,
        RedoxStr::new("/scheme/event").unwrap(),
        O_CLOEXEC | O_CREAT | O_RDWR,
        0o700,
    )? as usize)
}

pub fn queue_ctl(queue: usize, fd: usize, flags: u32, user_data: usize) -> Result<usize, Error> {
    let flags = event::raw::EventFlags::from_bits(flags).ok_or(Error::new(EINVAL))?;
    let res = syscall::write(
        queue,
        &syscall::Event {
            id: fd,
            flags: flags.into(),
            data: user_data,
        },
    )?;
    assert_eq!(
        res,
        size_of::<syscall::Event>(),
        "EOF not yet defined for event queue writes"
    );
    Ok(0)
}

pub fn queue_get_events(
    queue: usize,
    mut buf: Out<event::raw::RawEventV1>,
    buf_count: usize,
    flags: u32,
) -> Result<usize, Error> {
    if flags != 0 || buf_count == 0 {
        return Err(Error::new(EINVAL));
    }
    let mut event = syscall::Event::default();
    let res = syscall::read(queue, &mut event)?;
    assert_eq!(
        res,
        size_of::<syscall::Event>(),
        "EOF not yet defined for event queue reads"
    );
    buf.write(event::raw::RawEventV1 {
        fd: event.id,
        flags: event::raw::EventFlags::from(event.flags).bits(),
        user_data: event.data,
    });

    Ok(1)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_event_queue_create_v1(flags: u32) -> RawResult {
    Error::mux(queue_create(flags))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_event_queue_get_events_v1(
    queue: usize,
    buf: *mut event::raw::RawEventV1,
    buf_count: usize,
    flags: u32,
    _timeout: *const timespec,
    _sigset: *const sigset_t,
) -> RawResult {
    Error::mux(queue_get_events(
        queue,
        unsafe { Out::nonnull(buf) },
        buf_count,
        flags,
    ))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_event_queue_ctl_v1(
    queue: usize,
    fd: usize,
    flags: u32,
    user_data: usize,
) -> RawResult {
    Error::mux(queue_ctl(queue, fd, flags, user_data))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_event_queue_destroy_v1(queue: usize) -> RawResult {
    Error::mux(redox_rt::sys::close(queue))
}
