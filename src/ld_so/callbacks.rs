use super::linker::{Linker, ObjectHandle, ObjectScope, Resolve};
use crate::platform::types::c_void;
use alloc::boxed::Box;
use goblin::error::Result;

pub struct LinkerCallbacks {
    pub unload: Box<dyn Fn(&mut Linker, ObjectHandle)>,
    pub load_library:
        Box<dyn Fn(&mut Linker, Option<&str>, Resolve, ObjectScope, bool) -> Result<ObjectHandle>>,
    pub get_sym: Box<dyn Fn(&Linker, Option<ObjectHandle>, &str) -> Option<*mut c_void>>,
}

impl LinkerCallbacks {
    pub fn new() -> LinkerCallbacks {
        LinkerCallbacks {
            unload: Box::new(unload),
            load_library: Box::new(load_library),
            get_sym: Box::new(get_sym),
        }
    }
}

fn unload(linker: &mut Linker, handle: ObjectHandle) {
    linker.unload(handle)
}

fn load_library(
    linker: &mut Linker,
    name: Option<&str>,
    resolve: Resolve,
    scope: ObjectScope,
    noload: bool,
) -> Result<ObjectHandle> {
    linker.load_library(name, resolve, scope, noload)
}

fn get_sym(linker: &Linker, handle: Option<ObjectHandle>, name: &str) -> Option<*mut c_void> {
    linker.get_sym(handle, name)
}
