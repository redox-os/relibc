use super::linker::Linker;
use crate::platform::types::c_void;
use alloc::boxed::Box;
use goblin::error::Result;

pub struct LinkerCallbacks {
    pub unload: Box<dyn Fn(&mut Linker, usize)>,
    pub load_library: Box<dyn Fn(&mut Linker, Option<&str>) -> Result<usize>>,
    pub get_sym: Box<dyn Fn(&Linker, usize, &str) -> Option<*mut c_void>>,
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

fn unload(linker: &mut Linker, lib_id: usize) {
    linker.unload(lib_id)
}

fn load_library(linker: &mut Linker, name: Option<&str>) -> Result<usize> {
    linker.load_library(name)
}

fn get_sym(linker: &Linker, lib_id: usize, name: &str) -> Option<*mut c_void> {
    linker.get_sym(lib_id, name)
}
