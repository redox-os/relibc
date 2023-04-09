#![allow(non_camel_case_types)]

use crate::platform::types::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct sched_param {
    pub sched_priority: c_int,
}
