use super::{
    super::{
        Pal, PalEpoll,
        types::{c_int, c_uint},
    },
    Sys,
};

use crate::{
    error::Errno,
    header::{bits_sigset_t::sigset_t, errno::*, fcntl::*, sys_epoll::*},
};
use core::{mem, ptr, slice};
use syscall::{data::Event, flag::EVENT_READ};

fn epoll_to_event_flags(epoll: c_uint) -> syscall::EventFlags {
    let mut event_flags = syscall::EventFlags::empty();

    if epoll & EPOLLIN != 0 {
        event_flags |= syscall::EventFlags::EVENT_READ;
    }

    if epoll & EPOLLOUT != 0 {
        event_flags |= syscall::EventFlags::EVENT_WRITE;
    }

    /*TODO: support more EPOLL flags  */
    let unsupported = !(EPOLLIN | EPOLLOUT);
    if epoll & unsupported != 0 {
        log::trace!("epoll unsupported flags 0x{:X}", epoll & unsupported);
    }

    event_flags
}

fn event_flags_to_epoll(flags: syscall::EventFlags) -> c_uint {
    let mut epoll = 0;

    if flags.contains(syscall::EventFlags::EVENT_READ) {
        epoll |= EPOLLIN;
    }

    if flags.contains(syscall::EventFlags::EVENT_WRITE) {
        epoll |= EPOLLOUT;
    }

    epoll
}

impl PalEpoll for Sys {
    fn epoll_create1(flags: c_int) -> Result<c_int, Errno> {
        match redox_rt::sys::open("/scheme/event", (O_RDWR | flags) as usize) {
            Ok(fd) => Ok(fd as c_int),
            Err(e) => Err(Errno::from(e)),
        }
    }

    fn epoll_ctl(
        epfd: c_int,
        op: c_int,
        fd: c_int,
        event: Option<&mut epoll_event>,
    ) -> Result<(), Errno> {
        match op {
            EPOLL_CTL_ADD | EPOLL_CTL_MOD => {
                let Some(event) = event else {
                    return Err(Errno(EINVAL));
                };
                Sys::write(
                    epfd,
                    &Event {
                        id: fd as usize,
                        flags: epoll_to_event_flags(event.events),
                        data: event.data.into(),
                    },
                )?;
            }
            EPOLL_CTL_DEL => {
                Sys::write(
                    epfd,
                    &Event {
                        id: fd as usize,
                        flags: syscall::EventFlags::empty(),
                        // data is not required
                        data: 0,
                    },
                )?;
            }
            _ => return Err(Errno(EINVAL)),
        }
        Ok(())
    }

    fn epoll_pwait(
        epfd: c_int,
        events: &mut [epoll_event],
        timeout: c_int,
        sigmask: Option<&sigset_t>,
    ) -> Result<usize, Errno> {
        const { assert!(mem::size_of::<epoll_event>() == mem::size_of::<Event>()) }

        if timeout != -1 {
            register_timeout(epfd, timeout)?;
        }
        let mut callback = || {
            syscall::read(
                epfd as usize,
                // SAFETY: the const assert guarantees the same size
                unsafe {
                    slice::from_raw_parts_mut(
                        events.as_mut_ptr().cast::<u8>(),
                        events.len() as usize * mem::size_of::<syscall::Event>(),
                    )
                },
            )
        };

        let bytes_read = if let Some(sigmask) = sigmask {
            // Allowset is inverse of sigset mask
            let allowset = !sigmask;
            redox_rt::signal::callback_or_signal_async(allowset, callback)
        } else {
            callback()
        }?;

        let read = bytes_read as usize / mem::size_of::<syscall::Event>();
        assert!(read <= events.len());

        let events_ptr = events.as_mut_ptr();
        let mut count = 0;

        for i in 0..read {
            // SAFETY:
            // We already checked the size and it's aligned
            unsafe {
                let event_ptr = events_ptr.add(i) as *const syscall::Event;
                let event = ptr::read(event_ptr);

                if event.id == syscall::EVENT_TIMEOUT_ID {
                    continue;
                }
                let new_event =
                    epoll_event::new(event_flags_to_epoll(event.flags), event.data.into());
                events_ptr.add(count).write(new_event);
                count += 1;
            }
        }

        Ok(count)
    }
}

fn register_timeout(epfd: i32, timeout: i32) -> Result<usize, Errno> {
    if timeout < 0 {
        return Err(Errno(EINVAL));
    }
    Sys::write(
        epfd,
        &Event {
            id: syscall::EVENT_TIMEOUT_ID,
            flags: EVENT_READ,
            data: timeout as usize,
        },
    )
}
