use crate::{
    error::{Errno, Result},
    header::{bits_sigset_t::sigset_t, errno, poll},
    platform::{Sys, types::c_int},
};
use alloc::vec::Vec;
use core::{cell::Cell, mem::size_of};
use redox_protocols::protocol::O_CLOEXEC;
use syscall::{Event, EventFlags, O_NONBLOCK};

// TODO: close on thread exit?
#[thread_local]
static CACHED_EVENT_FD: Cell<Option<usize>> = Cell::new(None);

impl Sys {
    /// Wrapper to use epoll for poll
    pub(crate) fn redox_poll(
        fds: &mut [poll::pollfd],
        timeout: c_int,
        sigmask: Option<&sigset_t>,
    ) -> Result<c_int> {
        let event_map = [
            (poll::POLLIN, EventFlags::EVENT_READ),
            (poll::POLLOUT, EventFlags::EVENT_WRITE),
            (poll::POLLERR, EventFlags::EVENT_ERR),
            (poll::POLLHUP, EventFlags::EVENT_HUP),
            // TODO: EventFlags::EVENT_NVAL
            (poll::POLLNVAL, EventFlags::from_bits_retain(16)),
            // TODO: 5 remaining poll flags...
        ];

        let ep_fd = match CACHED_EVENT_FD.get() {
            Some(fd) => fd,
            None => {
                let ep =
                    redox_rt::sys::open_into_upper("/scheme/event", syscall::O_RDWR | O_CLOEXEC)?;
                CACHED_EVENT_FD.set(Some(ep));
                ep
            }
        };

        let mut valid_count = 0;
        for pfd in fds.iter_mut() {
            pfd.revents = 0;
            if pfd.fd >= 0 {
                valid_count += 1;
            }
        }

        let total_capacity = (valid_count + 1) * 2;

        let mut stack_buf = [Event::default(); 64];
        let mut heap_buf = Vec::new();

        let buf_slice = if total_capacity <= 64 {
            &mut stack_buf[..total_capacity]
        } else {
            heap_buf.resize(total_capacity, Event::default());
            &mut heap_buf[..]
        };

        let mut input_idx = 0;
        for (original_idx, pfd) in fds.iter().enumerate() {
            if pfd.fd < 0 {
                continue;
            }

            let mut flags = EventFlags::empty();
            for (p, ep) in &event_map[..2] {
                if pfd.events & p > 0 {
                    flags |= *ep;
                }
            }

            buf_slice[input_idx] = Event {
                id: pfd.fd as usize,
                flags,
                data: original_idx,
            };
            input_idx += 1;
        }

        if timeout >= 0 {
            buf_slice[input_idx] = Event {
                id: syscall::EVENT_TIMEOUT_ID,
                flags: EventFlags::EVENT_READ,
                data: timeout as usize,
            };
            input_idx += 1;
        }

        let read_flags = if timeout == 0 { O_NONBLOCK as u64 } else { 0 };
        let write_len = input_idx * size_of::<Event>();

        let metadata = [0, write_len as u64, read_flags];
        let payload_ptr = buf_slice.as_mut_ptr().cast::<u8>();
        let payload_len = buf_slice.len() * size_of::<Event>();

        let callback = || {
            redox_rt::sys::sys_call_rw(
                ep_fd,
                unsafe { core::slice::from_raw_parts_mut(payload_ptr, payload_len) },
                syscall::CallFlags::empty(),
                &metadata,
            )
        };

        let bytes_read = match if let Some(sigmask) = sigmask {
            let allowset = !*sigmask;
            redox_rt::signal::callback_or_signal_async(allowset, callback)
        } else {
            callback()
        } {
            Ok(b) => b,
            Err(err) => {
                if err.errno == errno::EBADF {
                    CACHED_EVENT_FD.set(None);
                }
                return Err(Errno(err.errno));
            }
        };

        let event_read = bytes_read / size_of::<Event>();
        let event_buf = &buf_slice[input_idx..input_idx + event_read];
        let mut count = 0;

        for event in event_buf {
            if event.id == syscall::EVENT_TIMEOUT_ID {
                continue;
            }

            let original_idx = event.data;
            if let Some(pfd) = fds.get_mut(original_idx) {
                for (p, ep) in &event_map {
                    if event.flags.contains(*ep) {
                        pfd.revents |= *p;
                    }
                }

                if pfd.revents > 0 {
                    count += 1;
                }
            }
        }

        Ok(count as c_int)
    }
}
