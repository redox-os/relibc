use crate::{c_str::CStr, platform::types::c_char};
use alloc::boxed::Box;
use core::ptr;

#[repr(C)]
pub enum RTLDState {
    /// Mapping change is complete.
    RtConsistent,
    /// Beginning to add a new object.
    RtAdd,
    /// Beginning to remove an object mapping.
    RtDelete,
}

/// Data structure for sharing debugging information from the
/// run-time dynamic linker for loaded ELF shared objects.
#[repr(C)]
pub struct RTLDDebug {
    /// Version number for this protocol.
    r_version: i32,
    /// Head of the chain of loaded objects.
    r_map: *mut LinkMap,
    //struct link_map *r_map;
    /// This is the address of a function internal to the run-time linker,
    /// that will always be called when the linker begins to map in a
    /// library or unmap it, and again when the mapping change is complete.
    /// The debugger can set a breakpoint at this address if it wants to
    /// notice shared object mapping changes.
    pub r_brk: extern "C" fn(),

    /// This state value describes the mapping change taking place when
    /// the `r_brk' address is called.
    pub state: RTLDState,

    ///  Base address the linker is loaded at.
    pub r_ldbase: usize,
}

impl RTLDDebug {
    const NEW: Self = RTLDDebug {
        r_version: 1,
        r_map: ptr::null_mut::<LinkMap>(),
        r_brk: _dl_debug_state,
        state: RTLDState::RtConsistent,
        r_ldbase: 0,
    };

    pub fn insert(&mut self, l_addr: usize, name: &CStr, l_ld: usize) -> LinkMapGuard {
        let node = if self.r_map.is_null() {
            self.r_map = LinkMap::new_with_args(l_addr, name, l_ld);
            self.r_map
        } else {
            unsafe { (*self.r_map).add_object(l_addr, name, l_ld) }
        };
        LinkMapGuard(node)
    }
    pub fn insert_first(&mut self, l_addr: usize, name: &CStr, l_ld: usize) -> LinkMapGuard {
        if self.r_map.is_null() {
            self.r_map = LinkMap::new_with_args(l_addr, name, l_ld);
        } else {
            let tmp = self.r_map;
            self.r_map = LinkMap::new_with_args(l_addr, name, l_ld);
            unsafe { (*self.r_map).link(&mut *tmp) };
        }
        LinkMapGuard(self.r_map)
    }
    unsafe fn remove(&mut self, node: *mut LinkMap) {
        unsafe {
            let prev = (*node).l_prev;
            let next = (*node).l_next;

            if !prev.is_null() {
                (*prev).l_next = next;
            } else {
                self.r_map = next;
            }

            if !next.is_null() {
                (*next).l_prev = prev;
            }

            LinkMap::remove(node);
        }
    }
}

/// SAFETY: safe as long as caller wraps the instance in a mutex,
/// or similar structure that guarantees exclusive mutable access.
/// Separate instances must not contain pointers to the same LinkMap instance.
unsafe impl Send for RTLDDebug {}
/// SAFETY: safe as long as caller wraps the instance in a mutex,
/// or similar structure that guarantees exclusive mutable access.
/// Separate instances must not contain pointers to the same LinkMap instance.
unsafe impl Sync for RTLDDebug {}

#[repr(C)]
struct LinkMap {
    /* These members are part of the protocol with the debugger.
    This is the same format used in SVR4.  */
    /// Difference between the address in the ELF
    /// file and the addresses in memory.
    l_addr: usize,
    /// Absolute file name object was found in.
    l_name: *const c_char,
    /// Dynamic section of the shared object.
    l_ld: usize,
    l_next: *mut LinkMap,
    l_prev: *mut LinkMap,
}

impl LinkMap {
    fn new() -> *mut Self {
        let map = Box::new(LinkMap {
            l_addr: 0,
            l_name: ptr::null(),
            l_ld: 0,
            l_next: ptr::null_mut(),
            l_prev: ptr::null_mut(),
        });
        Box::into_raw(map)
    }
    fn link(&mut self, map: &mut LinkMap) {
        map.l_prev = ptr::from_mut::<LinkMap>(self);
        self.l_next = ptr::from_mut::<LinkMap>(map);
    }
    fn new_with_args(l_addr: usize, name: &CStr, l_ld: usize) -> *mut Self {
        let map = LinkMap::new();
        unsafe {
            (*map).l_addr = l_addr;
            (*map).l_ld = l_ld;
            (*map).l_name = name.as_ptr();
        }
        map
    }

    fn add_object(&mut self, l_addr: usize, name: &CStr, l_ld: usize) -> *mut Self {
        let node = LinkMap::new_with_args(l_addr, name, l_ld);
        let mut last = self;
        while !last.l_next.is_null() {
            last = unsafe { last.l_next.as_mut() }.unwrap();
        }
        unsafe {
            (*node).l_prev = last;
            last.l_next = node;
        }
        node
    }
    unsafe fn remove(node: *mut Self) {
        // SAFETY: caller handle linked list removal already
        unsafe { drop(Box::from_raw(node)) };
    }
}

pub struct LinkMapGuard(*mut LinkMap);

unsafe impl Send for LinkMapGuard {}
unsafe impl Sync for LinkMapGuard {}

impl Drop for LinkMapGuard {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        _r_debug.lock().state = RTLDState::RtDelete;
        _dl_debug_state();

        unsafe { _r_debug.lock().remove(self.0) };

        _r_debug.lock().state = RTLDState::RtConsistent;
        _dl_debug_state();
    }
}

/*
 * Gdb may be looking for this fuction with that exact name and set
 * break point there
 */
#[linkage = "weak"]
#[unsafe(no_mangle)]
pub extern "C" fn _dl_debug_state() {}

#[unsafe(no_mangle)]
pub static _r_debug: spin::Mutex<RTLDDebug> = spin::Mutex::new(RTLDDebug::NEW);
