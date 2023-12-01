use super::{
    super::{types::*, Pal, PalEpoll},
    Sys,
};

use crate::{
    errno::Errno,
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

impl PalEpoll for Sys {
    fn epoll_create1(flags: c_int) -> Result<c_int, Errno> {
        Sys::open(c_str!("event:"), O_RDWR | flags, 0)
    }

    fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *mut epoll_event) -> Result<(), Errno> {
        match op {
            EPOLL_CTL_ADD | EPOLL_CTL_MOD => {
                Sys::write(
                    epfd,
                    &Event {
                        id: fd as usize,
                        flags: syscall::EventFlags::from_bits(unsafe { (*event).events as usize })
                            .expect("epoll: invalid bit pattern"),
                        // NOTE: Danger when using something smaller than 64-bit
                        // systems. If this is needed, use a box or something
                        data: unsafe { (*event).data.u64 as usize },
                    },
                )?;
                Ok(())
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
                Ok(())
            }
            _ => Err(Errno(EINVAL)),
        }
    }

    fn epoll_pwait(
        epfd: c_int,
        events: *mut epoll_event,
        maxevents: c_int,
        timeout: c_int,
        _sigset: *const sigset_t,
    ) -> Result<c_int, Errno> {
        // TODO: sigset
        assert_eq!(mem::size_of::<epoll_event>(), mem::size_of::<Event>());

        if maxevents <= 0 {
            return Err(Errno(EINVAL));
        }

        let timer_opt = if timeout != -1 {
            let mut timer = File::open(c_str!("time:4"), O_RDWR)?;
            let _ = Sys::write(
                epfd,
                &Event {
                    id: timer.fd as usize,
                    flags: EVENT_READ,
                    data: 0,
                },
            )?;

            let mut time = TimeSpec::default();
            let _ = timer.read(&mut time)?;
            time.tv_sec += (timeout as i64) / 1000;
            time.tv_nsec += (timeout % 1000) * 1000000;
            let _ = timer.write(&time)?;

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
                    events: event.flags.bits() as _,
                    data: epoll_data {
                        u64: event.data as u64,
                    },
                    ..Default::default()
                };
                count += 1;
            }
        }

        Ok(count as c_int)
    }
}
