use core::mem::size_of;

use crate::header::{
    bits_time::timespec,
    fcntl::{O_CLOEXEC, O_CREAT, O_RDWR},
    signal::sigset_t,
};

use super::libredox::RawResult;

use syscall::{EINVAL, Error, Result};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_event_queue_create_v1(flags: u32) -> RawResult {
    Error::mux((|| {
        if flags != 0 {
            return Err(Error::new(EINVAL));
        }
        Ok(super::libredox::open("/scheme/event", O_CLOEXEC | O_CREAT | O_RDWR, 0o700)? as usize)
    })())
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
    Error::mux((|| -> Result<usize> {
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
        unsafe {
            buf.write(event::raw::RawEventV1 {
                fd: event.id,
                flags: event::raw::EventFlags::from(event.flags).bits(),
                user_data: event.data,
            })
        };

        Ok(1)
    })())
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_event_queue_ctl_v1(
    queue: usize,
    fd: usize,
    flags: u32,
    user_data: usize,
) -> RawResult {
    Error::mux((|| -> Result<usize> {
        let res = syscall::write(
            queue,
            &syscall::Event {
                id: fd,
                flags: event::raw::EventFlags::from_bits(flags)
                    .ok_or(Error::new(EINVAL))?
                    .into(),
                data: user_data,
            },
        )?;
        assert_eq!(
            res,
            size_of::<syscall::Event>(),
            "EOF not yet defined for event queue writes"
        );
        Ok(0)
    })())
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_event_queue_destroy_v1(queue: usize) -> RawResult {
    Error::mux(syscall::close(queue))
}
