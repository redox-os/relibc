//use super::{constants, FILE};
//use platform::types::*;
//
//pub fn ftello(stream: &mut FILE) -> off_t {
//    let pos = stream.seek(
//        0,
//        if let Some((wbase, wpos, _)) = stream.write {
//            if (stream.flags & constants::F_APP > 0) && wpos > wbase {
//                constants::SEEK_END
//            } else {
//                constants::SEEK_CUR
//            }
//        } else {
//            constants::SEEK_CUR
//        },
//    );
//    if pos < 0 {
//        return pos;
//    }
//    let rdiff = if let Some((rpos, rend)) = stream.read {
//        rend - rpos
//    } else {
//        0
//    };
//    let wdiff = if let Some((wbase, wpos, _)) = stream.write {
//        wpos - wbase
//    } else {
//        0
//    };
//    pos - rdiff as i64 + wdiff as i64
//}
