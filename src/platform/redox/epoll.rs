use super::{
    super::{types::*, Pal, PalEpoll},
    Sys,
};

use crate::{
    fs::File,
    header::{errno::*, fcntl::*, signal::sigset_t, sys_epoll::*},
    io::prelude::*,
    platform,
    pthread::ResultExt,
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
    fn epoll_create1(flags: c_int) -> c_int {
        Sys::open(c_str!("/scheme/event"), O_RDWR | flags, 0)
    }

    fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *mut epoll_event) -> c_int {
        match op {
            EPOLL_CTL_ADD | EPOLL_CTL_MOD => {
                if Sys::write(
                    epfd,
                    &Event {
                        id: fd as usize,
                        flags: unsafe { epoll_to_event_flags((*event).events) },
                        // NOTE: Danger when using something smaller than 64-bit
                        // systems. If this is needed, use a box or something
                        data: unsafe { (*event).data.u64 as usize },
                    },
                )
                .or_minus_one_errno()
                    < 0
                {
                    -1
                } else {
                    0
                }
            }
            EPOLL_CTL_DEL => {
                if Sys::write(
                    epfd,
                    &Event {
                        id: fd as usize,
                        flags: syscall::EventFlags::empty(),
                        //TODO: Is data required?
                        data: 0,
                    },
                )
                .or_minus_one_errno()
                    < 0
                {
                    -1
                } else {
                    0
                }
            }
            _ => {
                platform::ERRNO.set(EINVAL);
                -1
            }
        }
    }

    fn epoll_pwait(
        epfd: c_int,
        events: *mut epoll_event,
        maxevents: c_int,
        timeout: c_int,
        _sigset: *const sigset_t,
    ) -> c_int {
        // TODO: sigset
        assert_eq!(mem::size_of::<epoll_event>(), mem::size_of::<Event>());

        if maxevents <= 0 {
            platform::ERRNO.set(EINVAL);
            return -1;
        }

        let timer_opt = if timeout != -1 {
            match File::open(c_str!("/scheme/time/4"), O_RDWR) {
                Err(_) => return -1,
                Ok(mut timer) => {
                    if Sys::write(
                        epfd,
                        &Event {
                            id: timer.fd as usize,
                            flags: EVENT_READ,
                            data: 0,
                        },
                    )
                    .or_minus_one_errno()
                        == -1
                    {
                        return -1;
                    }

                    let mut time = TimeSpec::default();
                    if let Err(err) = timer.read(&mut time) {
                        return -1;
                    }
                    time.tv_sec += (timeout as i64) / 1000;
                    time.tv_nsec += (timeout % 1000) * 1000000;
                    if let Err(err) = timer.write(&time) {
                        return -1;
                    }

                    Some(timer)
                }
            }
        } else {
            None
        };

        let bytes_read = Sys::read(epfd, unsafe {
            slice::from_raw_parts_mut(
                events as *mut u8,
                maxevents as usize * mem::size_of::<syscall::Event>(),
            )
        })
        .or_minus_one_errno(); // TODO
        if bytes_read == -1 {
            return -1;
        }
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

        count as c_int
    }
}
