use crate::{
    error::{Errno, Result},
    fs::File,
    header::{
        sys_socket::{
            connect,
            constants::{AF_UNIX, SOCK_CLOEXEC, SOCK_DGRAM},
            sockaddr, socket,
        },
        sys_un::sockaddr_un,
        unistd::close,
    },
    io::BufWriter,
    platform::ERRNO,
};

use super::logger::LogSink;

/// Unix Domain Socket connected to /dev/log.
pub struct LogFile(BufWriter<File>);

impl LogSink for LogFile {
    type Sink = BufWriter<File>;

    fn open() -> Result<Self>
    where
        Self: Sized,
    {
        let log_addr = {
            let path = c"/dev/log";
            let mut sockaddr: sockaddr_un = unsafe { core::mem::zeroed() };
            sockaddr.sun_family = AF_UNIX as _;
            unsafe {
                core::ptr::copy_nonoverlapping(
                    path.as_ptr(),
                    sockaddr.sun_path.as_mut_ptr(),
                    path.count_bytes(),
                )
            };
            path
        };
        let log_fd = {
            let result = unsafe { socket(AF_UNIX, SOCK_DGRAM | SOCK_CLOEXEC, 0) };
            if result > 0 {
                Ok(result)
            } else {
                Err(Errno(ERRNO.get()))
            }
        }?;

        // SAFETY:
        // * connect handles invalid descriptors.
        // * log_addr is a sockaddr_un so the size is correct.
        if unsafe {
            connect(
                log_fd,
                &raw const log_addr as *const sockaddr,
                size_of::<sockaddr_un>(),
            ) < 0
        } {
            // In case close sets ERRNO.
            let e = ERRNO.get();
            close(log_fd);
            return Err(Errno(e));
        }

        Ok(Self(BufWriter::new(File::new(log_fd))))
    }

    #[inline(always)]
    fn writer(&mut self) -> &mut Self::Sink {
        &mut self.0
    }
}
