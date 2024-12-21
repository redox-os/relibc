use super::{
    super::{types::*, Pal, PalEpoll},
    Sys,
};

use crate::{
    error::{Errno, ResultExt},
    fs::File,
    header::{errno::*, fcntl::*, signal::sigset_t, sys_epoll::*},
    io::prelude::*,
    platform,
};
use core::{mem, slice};
use syscall::{
    data::{Event, TimeSpec},
    flag::EVENT_READ,
};

fn epoll_to_event_flags(epoll: c_uint) -> syscall::EventFlags {
    let mut event_flags = syscall::EventFlags::empty();

    if epoll & EPOLLIN != 0 {
        event_flags |= syscall::EventFlags::EVENT_READ;
    }

    if epoll & EPOLLOUT != 0 {
        event_flags |= syscall::EventFlags::EVENT_WRITE;
    }

    /*TODO: support more EPOLL flags
    let unsupported = !(EPOLLIN | EPOLLOUT);
    if epoll & unsupported != 0 {
        eprintln!("epoll unsupported flags 0x{:X}", epoll & unsupported);
    }
    */

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
        Sys::open(c_str!("/scheme/event"), O_RDWR | flags, 0)
    }

    unsafe fn epoll_ctl(
        epfd: c_int,
        op: c_int,
        fd: c_int,
        event: *mut epoll_event,
    ) -> Result<(), Errno> {
        match op {
            EPOLL_CTL_ADD | EPOLL_CTL_MOD => {
                Sys::write(
                    epfd,
                    &Event {
                        id: fd as usize,
                        flags: unsafe { epoll_to_event_flags((*event).events) },
                        // NOTE: Danger when using something smaller than 64-bit
                        // systems. If this is needed, use a box or something
                        data: unsafe { (*event).data.u64 as usize },
                    },
                )?;
            }
            EPOLL_CTL_DEL => {
                Sys::write(
                    epfd,
                    &Event {
                        id: fd as usize,
                        flags: syscall::EventFlags::empty(),
                        //TODO: Is data required?
                        data: 0,
                    },
                )?;
            }
            _ => return Err(Errno(EINVAL)),
        }
        Ok(())
    }

    unsafe fn epoll_pwait(
        epfd: c_int,
        events: *mut epoll_event,
        maxevents: c_int,
        timeout: c_int,
        _sigset: *const sigset_t,
    ) -> Result<usize, Errno> {
        // TODO: sigset
        assert_eq!(mem::size_of::<epoll_event>(), mem::size_of::<Event>());

        if maxevents <= 0 {
            return Err(Errno(EINVAL));
        }

        let timer_opt = if timeout != -1 {
            let mut timer = File::open(c_str!("/scheme/time/4"), O_RDWR)?;
            Sys::write(
                epfd,
                &Event {
                    id: timer.fd as usize,
                    flags: EVENT_READ,
                    data: 0,
                },
            )?;

            let mut time = TimeSpec::default();
            let _ = timer
                .read(&mut time)
                .map_err(|err| Errno(err.raw_os_error().unwrap_or(EIO)))?;
            time.tv_sec += (timeout as i64) / 1000;
            time.tv_nsec += (timeout % 1000) * 1000000;
            let _ = timer
                .write(&time)
                .map_err(|err| Errno(err.raw_os_error().unwrap_or(EIO)))?;

            Some(timer)
        } else {
            None
        };

        let bytes_read = Sys::read(epfd, unsafe {
            slice::from_raw_parts_mut(
                events as *mut u8,
                maxevents as usize * mem::size_of::<syscall::Event>(),
            )
        })?;
        let read = bytes_read as usize / mem::size_of::<syscall::Event>();

        let mut count = 0;
        for i in 0..read {
            unsafe {
                let event_ptr = events.add(i);
                let target_ptr = events.add(count);
                let event = *(event_ptr as *mut Event);
                if let Some(ref timer) = timer_opt {
                    if event.id as c_int == timer.fd {
                        // Do not count timer event
                        continue;
                    }
                }
                *target_ptr = epoll_event {
                    events: event_flags_to_epoll(event.flags),
                    data: epoll_data {
                        u64: event.data as u64,
                    },
                    ..Default::default()
                };
                count += 1;
            }
        }

        Ok(count)
    }
}
