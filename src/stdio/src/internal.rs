use super::{constants, FILE};
use platform;
use platform::types::*;
use core::{mem, ptr, slice};

pub fn ftello(stream: &mut FILE) -> off_t {
    let pos = stream.seek(
        0,
        if (stream.flags & constants::F_APP > 0) && stream.wpos > stream.wbase {
            constants::SEEK_END
        } else {
            constants::SEEK_CUR
        },
    );
    if pos < 0 {
        return pos;
    }
    pos - (stream.rend as i64 - stream.rpos as i64) + (stream.wpos as i64 - stream.wbase as i64)
}
