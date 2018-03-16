use super::{constants, FILE};
use platform;
use platform::types::*;
use core::{mem, ptr, slice};

pub fn stdio_read(stream: *mut FILE, buf: *mut u8, size: usize) -> usize {
    unsafe {
        let mut buff = slice::from_raw_parts_mut(buf, size - !((*stream).buf_size == 0) as usize);
        let mut file_buf = slice::from_raw_parts_mut((*stream).buf, (*stream).buf_size);
        let mut file = platform::FileReader((*stream).fd);
        let count = if buff.len() == 0 {
            file.read(&mut file_buf)
        } else {
            file.read(&mut buff) + file.read(&mut file_buf)
        };
        mem::forget(buff);
        mem::forget(file_buf);
        if count <= 0 {
            (*stream).flags |= if count == 0 {
                constants::F_EOF
            } else {
                constants::F_ERR
            };
            return 0;
        }
        if count as usize <= size {
            return count as usize;
        }
        (*stream).rpos = (*stream).buf;
        (*stream).rend = (*stream).buf.offset(count);
        *buf.offset(size as isize - 1) = *(*stream).rpos;
        (*stream).rpos = (*stream).rpos.add(1);
    }
    size
}

pub fn stdio_write(stream: *mut FILE, buf: *const u8, size: usize) -> usize {
    unsafe {
        let len = (*stream).wpos as usize - (*stream).wbase as usize;
        let mut advance = 0;
        let mut f_buf = slice::from_raw_parts((*stream).wbase, len);
        let mut buff = slice::from_raw_parts(buf, size);
        let mut f_filled = false;
        let mut rem = f_buf.len() + buff.len();
        let mut file = platform::FileWriter((*stream).fd);
        loop {
            let mut count = if f_filled {
                file.write(&f_buf[advance..])
            } else {
                file.write(&f_buf[advance..]) + file.write(buff)
            };
            if count == rem as isize {
                (*stream).wend = (*stream).buf.add((*stream).buf_size - 1);
                (*stream).wpos = (*stream).buf;
                (*stream).wbase = (*stream).buf;
                return size;
            }
            if count < 0 {
                (*stream).wpos = ptr::null_mut();
                (*stream).wbase = ptr::null_mut();
                (*stream).wend = ptr::null_mut();
                (*stream).flags |= constants::F_ERR;
                return 0;
            }
            rem -= count as usize;
            if count as usize > len {
                count -= len as isize;
                f_buf = buff;
                f_filled = true;
                advance = 0;
            }
            advance += count as usize;
        }
    }
}

pub unsafe fn to_read(stream: *mut FILE) -> bool {
    if (*stream).flags & constants::F_BADJ > 0 {
        // Static and needs unget region
        (*stream).buf = (*stream).buf.add((*stream).unget);
        (*stream).flags &= !constants::F_BADJ;
    }

    if (*stream).wpos > (*stream).wbase {
        if let Some(f) = (*stream).write {
            f(stream, ptr::null(), 0);
        }
    }
    (*stream).wpos = ptr::null_mut();
    (*stream).wbase = ptr::null_mut();
    (*stream).wend = ptr::null_mut();
    if (*stream).flags & constants::F_NORD > 0 {
        (*stream).flags |= constants::F_ERR;
        return true;
    }
    (*stream).rpos = (*stream).buf.offset((*stream).buf_size as isize - 1);
    (*stream).rend = (*stream).buf.offset((*stream).buf_size as isize - 1);
    if (*stream).flags & constants::F_EOF > 0 {
        true
    } else {
        false
    }
}

pub unsafe fn to_write(stream: *mut FILE) -> bool {
    if (*stream).flags & constants::F_BADJ > 0 {
        // Static and needs unget region
        (*stream).buf = (*stream).buf.add((*stream).unget);
        (*stream).flags &= !constants::F_BADJ;
    }

    if (*stream).flags & constants::F_NOWR > 0 {
        (*stream).flags &= constants::F_ERR;
        return false;
    }
    (*stream).rpos = ptr::null_mut();
    (*stream).rend = ptr::null_mut();
    (*stream).wpos = (*stream).buf;
    (*stream).wbase = (*stream).buf;
    (*stream).wend = (*stream).buf.offset((*stream).buf_size as isize - 1);
    return true;
}

pub unsafe fn ftello(stream: *mut FILE) -> off_t {
    if let Some(s) = (*stream).seek {
        let pos = s(
            stream,
            0,
            if ((*stream).flags & constants::F_APP > 0) && (*stream).wpos > (*stream).wbase {
                constants::SEEK_END
            } else {
                constants::SEEK_CUR
            },
        );
        if pos < 0 {
            return pos;
        }
        pos - ((*stream).rend as i64 - (*stream).rpos as i64)
            + ((*stream).wpos as i64 - (*stream).wbase as i64)
    } else {
        -1
    }
}

#[cfg(target_os = "linux")]
pub fn stdio_seek(stream: *mut FILE, off: off_t, whence: c_int) -> off_t {
    unsafe { platform::lseek((*stream).fd, off, whence) }
}

#[cfg(target_os = "redox")]
pub fn stdio_seek(stream: *mut FILE, off: off_t, whence: c_int) -> off_t {
    unsafe { platform::lseek((*stream).fd, off as isize, whence as usize) as off_t }
}
