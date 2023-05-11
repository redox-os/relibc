use crate::platform::types::*;

// TODO: Statically verify size and align
#[repr(C)]
#[derive(Clone, Copy)]
pub union sem_t {
    pub size: [c_char; 4],
    pub align: c_long,
}
pub type RlctSempahore = crate::sync::Semaphore;

#[no_mangle]
pub unsafe extern "C" fn sem_init(sem: *mut sem_t, _pshared: c_int, value: c_uint) -> c_int {
    sem.cast::<RlctSempahore>().write(RlctSempahore::new(value));

    0
}

#[no_mangle]
pub unsafe extern "C" fn sem_destroy(sem: *mut sem_t) -> c_int {
    core::ptr::drop_in_place(sem.cast::<RlctSempahore>());
    0
}


// TODO: va_list
// #[no_mangle]
pub unsafe extern "C" fn sem_open(name: *const c_char, oflag: c_int, /* (va_list) value: c_uint */) -> *mut sem_t {
    todo!("named semaphores")
}

// #[no_mangle]
pub unsafe extern "C" fn sem_close(sem: *mut sem_t) -> c_int {
    todo!("named semaphores")
}

// #[no_mangle]
pub unsafe extern "C" fn sem_unlink(name: *const c_char) -> c_int {
    todo!("named semaphores")
}

#[no_mangle]
pub unsafe extern "C" fn sem_wait(sem: *mut sem_t) -> c_int {
    get(sem).wait(None);

    0
}

#[no_mangle]
pub unsafe extern "C" fn sem_trywait(sem: *mut sem_t) -> c_int {
    get(sem).try_wait();

    0
}

#[no_mangle]
pub unsafe extern "C" fn sem_post(sem: *mut sem_t) -> c_int {
    get(sem).post(1);

    0
}

#[no_mangle]
pub unsafe extern "C" fn sem_getvalue(sem: *mut sem_t, sval: *mut c_int) -> c_int {
    sval.write(get(sem).value() as c_int);

    0
}

unsafe fn get<'any>(sem: *mut sem_t) -> &'any RlctSempahore {
    &*sem.cast()
}
