#![allow(non_camel_case_types)]

use crate::platform::types::*;

use crate::header::sched::sched_param;

use crate::sync::AtomicLock;
use core::sync::atomic::{AtomicU32 as AtomicUint, AtomicI32 as AtomicInt};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct pthread_attr_t {
    pub detachstate: c_uchar,
    pub inheritsched: c_uchar,
    pub schedpolicy: c_uchar,
    pub scope: c_uchar,
    pub guardsize: size_t,
    pub stacksize: size_t,
    pub stack: size_t,
    pub param: sched_param,
}

#[repr(C)]
pub struct pthread_rwlockattr_t {
    pub pshared: c_int,
}

#[repr(C)]
pub struct pthread_rwlock_t {
    pub state: AtomicInt,
}

#[repr(C)]
pub struct pthread_barrier_t {
    pub count: AtomicUint,
    pub original_count: c_uint,
    pub epoch: AtomicInt,
}

#[repr(C)]
pub struct pthread_barrierattr_t {
    pub pshared: c_int,
}

#[repr(C)]
pub struct pthread_mutex_t {
    pub inner: AtomicInt,
}

#[repr(C)]
pub struct pthread_mutexattr_t {
    pub prioceiling: c_int,
    pub protocol: c_int,
    pub pshared: c_int,
    pub robust: c_int,
    pub ty: c_int,
}

#[repr(C)]
pub struct pthread_condattr_t {
    pub clock: clockid_t,
    pub pshared: c_int,
}

#[repr(C)]
pub struct pthread_cond_t {
    pub cur: AtomicInt,
    pub prev: AtomicInt,
}
#[repr(C)]
pub struct pthread_spinlock_t {
    pub inner: AtomicInt,
}

#[repr(C)]
pub struct pthread_once_t {
    pub inner: AtomicInt,
}

pub type pthread_t = *mut ();
pub type pthread_key_t = c_ulong;
