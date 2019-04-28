use super::super::types::*;
use super::super::{Pal, PalEpoll};
use super::Sys;

use core::{mem, slice};
use fs::File;
use header::errno::*;
use header::fcntl::*;
use header::signal::sigset_t;
use header::sys_epoll::*;
use io::prelude::*;
use platform;
use syscall::data::{Event, TimeSpec};
use syscall::flag::EVENT_READ;

impl PalEpoll for Sys {
    fn epoll_create1(flags: c_int) -> c_int {
        Sys::open(
            c_str!("event:"),
            O_RDWR | flags,
            0,
        )
    }

    fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *mut epoll_event) -> c_int {
        match op {
            EPOLL_CTL_ADD | EPOLL_CTL_MOD => {
                Sys::write(
                    epfd,
                    &Event {
                        id: fd as usize,
                        flags: unsafe { (*event).events as usize },
                        // NOTE: Danger when using non 64-bit systems. If this is
                        // needed, use a box or something
                        data: unsafe { mem::transmute((*event).data) },
                    },
                ) as c_int
            },
            EPOLL_CTL_DEL => {
                Sys::write(
                    epfd,
                    &Event {
                        id: fd as usize,
                        flags: 0,
                        //TODO: Is data required?
                        data: 0,
                    },
                ) as c_int
            },
            _ => {
                unsafe { platform::errno = EINVAL };
                return -1;
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

        let timer_opt = if timeout != -1 {
            match File::open(c_str!("time:4"), O_RDWR) {
                Err(_) => return -1,
                Ok(mut timer) => {
                    if Sys::write(
                        epfd,
                        &Event {
                            id: timer.fd as usize,
                            flags: EVENT_READ,
                            data: 0,
                        },
                    ) == -1
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
            slice::from_raw_parts_mut(events as *mut u8, maxevents as usize)
        });
        if bytes_read == -1 {
            return -1;
        }
        let read = bytes_read as usize / mem::size_of::<epoll_event>();

        for i in 0..read {
            unsafe {
                let event_ptr = events.add(i);
                let event = *(event_ptr as *mut Event);
                if let Some(ref timer) = timer_opt {
                    if event.id as c_int == timer.fd {
                        return EINTR;
                    }
                }
                *event_ptr = epoll_event {
                    events: event.flags as _,
                    data: mem::transmute(event.data),
                    ..Default::default()
                };
            }
        }

        read as c_int
    }
}
