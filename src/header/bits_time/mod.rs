use crate::{
    header::time::NANOSECONDS,
    platform::types::{c_long, time_t},
};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/time.h.html>.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct timespec {
    pub tv_sec: time_t,
    pub tv_nsec: c_long,
}

impl timespec {
    // TODO: Write test

    /// similar logic with timeradd
    pub fn add(base: timespec, interval: timespec) -> Option<timespec> {
        let delta_sec = base.tv_sec.checked_add(interval.tv_sec)?;
        let delta_nsec = base.tv_nsec.checked_add(interval.tv_nsec)?;

        if delta_sec < 0 || delta_nsec < 0 {
            return None;
        }

        Some(Self {
            tv_sec: delta_sec + (delta_nsec / NANOSECONDS) as time_t,
            tv_nsec: delta_nsec % NANOSECONDS,
        })
    }
    /// similar logic with timersub
    pub fn subtract(later: timespec, earlier: timespec) -> Option<timespec> {
        let delta_sec = later.tv_sec.checked_sub(earlier.tv_sec)?;
        let delta_nsec = later.tv_nsec.checked_sub(earlier.tv_nsec)?;

        let time = if delta_nsec < 0 {
            let roundup_sec = -delta_nsec / NANOSECONDS + 1;
            timespec {
                tv_sec: delta_sec - (roundup_sec as time_t),
                tv_nsec: roundup_sec * NANOSECONDS - delta_nsec,
            }
        } else {
            timespec {
                tv_sec: delta_sec + (delta_nsec / NANOSECONDS) as time_t,
                tv_nsec: delta_nsec % NANOSECONDS,
            }
        };

        if time.tv_sec < 0 {
            // https://man7.org/linux/man-pages/man2/settimeofday.2.html
            // caller should return EINVAL
            return None;
        }

        Some(time)
    }
    pub fn is_default(&self) -> bool {
        self.tv_nsec == 0 && self.tv_sec == 0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cbindgen_stupid_alias_timespec(_: timespec) {}
