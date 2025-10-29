//! limits.h implementation for relibc

pub const HOST_NAME_MAX: usize = 64;
pub const NAME_MAX: usize = 255;
pub const PASS_MAX: usize = 128;
pub const PATH_MAX: usize = 4096;
pub const NGROUPS_MAX: usize = 65536;

// TODO: 4096 for most architectures as determined by a quick grep of musl's source; need a better
// way to determine it for other archs or to hard code a value.
#[cfg(target_os = "linux")]
pub const PAGE_SIZE: usize = 4096;
