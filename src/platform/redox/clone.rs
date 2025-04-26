use core::mem::size_of;

use syscall::{
    data::Map,
    error::Result,
    flag::{MapFlags, O_CLOEXEC},
    SetSighandlerData,
};

use redox_rt::{proc::FdGuard, signal::sighandler_function};

pub use redox_rt::{proc::*, thread::*};
