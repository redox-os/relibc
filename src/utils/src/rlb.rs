use alloc::boxed::Box;
use core::{mem, ptr, str};
use platform::read;

/// Implements an `Iterator` which returns on either newline or EOF.
#[derive(Clone, Copy)]
pub struct RawLineBuffer {
    pub fd: i32,
    pub cur: usize,
    pub read: usize,
    pub buf: [u8; 8 * 1024],
}

impl Default for RawLineBuffer {
    fn default() -> RawLineBuffer {
        RawLineBuffer {
            fd: 0,
            cur: 0,
            read: 0,
            buf: unsafe { mem::uninitialized() },
        }
    }
}

impl RawLineBuffer {
    pub fn new(fd: i32) -> RawLineBuffer {
        RawLineBuffer {
            fd: fd,
            cur: 0,
            read: 0,
            buf: unsafe { mem::uninitialized() },
        }
    }
}

impl Iterator for RawLineBuffer {
    type Item = Box<str>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur != 0 && self.read != 0 {
            if let Some(mut pos) = self.buf[self.cur..self.read]
                .iter()
                .position(|&x| x == b'\n')
            {
                pos += self.cur + 1;
                let line = unsafe { str::from_utf8_unchecked(&self.buf[self.cur..pos]) };
                let boxed_array: Box<[u8]> = Box::from(line.as_bytes());
                let boxed_line: Box<str> = unsafe { mem::transmute(boxed_array) };
                self.cur = pos;
                return Some(boxed_line);
            }

            let mut temp: [u8; 8 * 1024] = unsafe { mem::uninitialized() };
            unsafe {
                ptr::copy(self.buf[self.cur..].as_ptr(), temp.as_mut_ptr(), self.read);
                ptr::swap(self.buf.as_mut_ptr(), temp.as_mut_ptr());
            };

            self.read -= self.cur;
            self.cur = 0;
        }

        let bytes_read = {
            let buf = &mut self.buf[self.cur..];
            read(self.fd, buf) as usize
        };

        let end = match bytes_read {
            0 => return None,
            read => {
                self.read += read;
                self.buf[..self.read]
                    .iter()
                    .position(|&x| x == b'\n')
                    .unwrap_or(0)
            }
        };

        self.cur = end;
        let line = unsafe { str::from_utf8_unchecked(&self.buf[..end]) };
        let boxed_array: Box<[u8]> = Box::from(line.as_bytes());
        let boxed_line: Box<str> = unsafe { mem::transmute(boxed_array) };

        Some(boxed_line)
    }
}
