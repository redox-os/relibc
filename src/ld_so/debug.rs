use crate::{c_str::CString, platform::types::*};
use alloc::boxed::Box;

#[repr(C)]
pub enum RTLDState {
    /// Mapping change is complete.
    RT_CONSISTENT,
    /// Beginning to add a new object.
    RT_ADD,
    /// Beginning to remove an object mapping.
    RT_DELETE,
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
        r_map: 0 as *mut LinkMap,
        r_brk: _dl_debug_state,
        state: RTLDState::RT_CONSISTENT,
        r_ldbase: 0,
    };

    pub fn insert(&mut self, l_addr: usize, name: &str, l_ld: usize) {
        if self.r_map.is_null() {
            self.r_map = LinkMap::new_with_args(l_addr, name, l_ld);
        } else {
            unsafe { (*self.r_map).add_object(l_addr, name, l_ld) };
        }
        return;
    }
    pub fn insert_first(&mut self, l_addr: usize, name: &str, l_ld: usize) {
        if self.r_map.is_null() {
            self.r_map = LinkMap::new_with_args(l_addr, name, l_ld);
        } else {
            let tmp = self.r_map;
            self.r_map = LinkMap::new_with_args(l_addr, name, l_ld);
            unsafe { (*self.r_map).link(&mut *tmp) };
        }
        return;
    }
}

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
            l_name: 0 as *const c_char,
            l_ld: 0,
            l_next: 0 as *mut LinkMap,
            l_prev: 0 as *mut LinkMap,
        });
        Box::into_raw(map)
    }
    fn link(&mut self, map: &mut LinkMap) {
        map.l_prev = self as *mut LinkMap;
        self.l_next = map as *mut LinkMap;
    }
    fn new_with_args(l_addr: usize, name: &str, l_ld: usize) -> *mut Self {
        let map = LinkMap::new();
        unsafe {
            (*map).l_addr = l_addr;
            (*map).l_ld = l_ld;
            let c_name = CString::new(name).unwrap();
            (*map).l_name = c_name.into_raw() as *const c_char;
        }
        map
    }

    fn add_object(&mut self, l_addr: usize, name: &str, l_ld: usize) {
        let node = LinkMap::new_with_args(l_addr, name, l_ld);
        let mut last = self;
        while !last.l_next.is_null() {
            last = unsafe { last.l_next.as_mut() }.unwrap();
        }
        unsafe {
            (*node).l_prev = last;
            (*last).l_next = node;
        }
    }
}

/*
 * Gdb may be looking for this fuction with that exact name and set
 * break point there
 */
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn _dl_debug_state() {}

#[no_mangle]
pub static mut _r_debug: RTLDDebug = RTLDDebug::NEW;
