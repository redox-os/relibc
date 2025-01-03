use alloc::{
    collections::BTreeMap,
    rc::Rc,
    string::{String, ToString},
    sync::{Arc, Weak},
    vec::Vec,
};
use core::{
    cell::RefCell,
    mem::transmute,
    ptr::{self, NonNull},
};
use goblin::elf::{dynamic::DT_STRTAB, program_header, reloc, sym::STT_TLS, Elf};

use crate::{
    c_str::{CStr, CString},
    error::Errno,
    header::{
        dl_tls::{__tls_get_addr, dl_tls_index},
        fcntl, sys_mman,
        unistd::F_OK,
    },
    io::{self, Read},
    ld_so::dso::SymbolBinding,
    platform::{
        types::{c_char, c_int, c_uint, c_void},
        Pal, Sys,
    },
    sync::rwlock::RwLock,
};

use super::{
    access::accessible,
    callbacks::LinkerCallbacks,
    debug::{RTLDState, _dl_debug_state, _r_debug},
    dso::{is_pie_enabled, DSO},
    tcb::{Master, Tcb},
    PATH_SEP,
};

#[derive(Debug, Copy, Clone)]
pub enum DlError {
    /// Failed to locate the requested DSO.
    NotFound,
    /// The DSO is malformed somehow.
    Malformed,
    /// Invalid DSO handle.
    InvalidHandle,
}

impl DlError {
    /// Returns a human-readable, null-terminated C string describing the error.
    pub fn repr(&self) -> CStr<'static> {
        match self {
            DlError::NotFound => c_str!(
                "Failed to locate the requested DSO. Set `LD_DEBUG=all` for more information."
            ),

            DlError::Malformed => {
                c_str!("The DSO is malformed somehow. Set `LD_DEBUG=all` for more information.")
            }

            DlError::InvalidHandle => {
                c_str!("Invalid DSO handle. Set `LD_DEBUG=all` for more information.")
            }
        }
    }
}

pub type Result<T> = core::result::Result<T, DlError>;

static GLOBAL_SCOPE: RwLock<Scope> = RwLock::new(Scope::global());

/// Same as [`crate::fs::File`], but does not touch [`crate::platform::ERRNO`] as the dynamic
/// linker does not have thread-local storage.
struct File {
    fd: c_int,
}

impl File {
    pub fn open(path: CStr, oflag: c_int) -> core::result::Result<Self, Errno> {
        Ok(Self {
            fd: Sys::open(path, oflag, 0)?,
        })
    }
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, io::Error> {
        Sys::read(self.fd, buf).map_err(|err| io::Error::from_raw_os_error(err.0))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Symbol {
    pub value: usize,
    pub base: usize,
    pub size: usize,
    pub sym_type: u8,
}

impl Symbol {
    pub fn as_ptr(self) -> *mut c_void {
        (self.base + self.value) as *mut c_void
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum Resolve {
    /// Resolve all undefined symbols immediately.
    #[cfg_attr(not(target_arch = "x86_64"), default)]
    Now,
    /// Perform lazy binding (i.e. symbols will be resolved when they are first
    /// used).
    #[cfg_attr(target_arch = "x86_64", default)]
    Lazy,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ObjectScope {
    Global,
    Local,
}

pub enum Scope {
    Global {
        objs: Vec<Weak<DSO>>,
    },
    Local {
        owner: Option<Weak<DSO>>,
        objs: Vec<Arc<DSO>>,
    },
}

impl Scope {
    #[inline]
    const fn global() -> Self {
        Self::Global { objs: Vec::new() }
    }

    #[inline]
    pub(super) const fn local() -> Self {
        Self::Local {
            owner: None,
            objs: Vec::new(),
        }
    }

    fn set_owner(&mut self, obj: Weak<DSO>) {
        match self {
            Self::Global { .. } => panic!("attempted to set global scope owner"),
            Self::Local { ref mut owner, .. } => {
                assert!(owner.is_none(), "attempted to change local scope owner");
                *owner = Some(obj);
            }
        }
    }

    fn add(&mut self, target: &Arc<DSO>) {
        match self {
            Self::Global { objs } => {
                let target = Arc::downgrade(target);
                for obj in objs.iter() {
                    if Weak::ptr_eq(obj, &target) {
                        return;
                    }
                }

                objs.push(target);
            }

            Self::Local { objs, .. } => {
                for obj in objs.iter() {
                    if Arc::ptr_eq(obj, target) {
                        return;
                    }
                }

                objs.push(target.clone());
            }
        }
    }

    fn get_sym(&self, name: &str) -> Option<(Symbol, SymbolBinding, Arc<DSO>)> {
        let mut res = None;

        let get_sym = |obj: Arc<DSO>| {
            if let Some((sym, binding)) = obj.get_sym(name) {
                if binding.is_global() {
                    return Some((sym, binding, obj.clone()));
                }

                res = Some((sym, binding, obj.clone()));
            }

            None
        };

        match self {
            Self::Global { objs } => objs.iter().map(|o| o.upgrade().unwrap()).find_map(get_sym),
            Self::Local { owner, objs } => {
                let owner = owner
                    .as_ref()
                    .expect("local scope without owner")
                    .upgrade()
                    .expect("local scope owner was dropped");

                core::iter::once(owner)
                    .chain(objs.iter().cloned())
                    .find_map(get_sym)
            }
        }
        .or(res)
    }

    fn move_into(&self, other: &mut Self) {
        // FIXME: move not copy? as afiak u cannot downgrade from a global to a local scope.
        match (self, other) {
            (Self::Local { owner, objs }, Self::Global { objs: other_objs }) => {
                let owner = owner.as_ref().expect("local scope without owner");
                other_objs.push(owner.clone());
                other_objs.extend(objs.iter().map(Arc::downgrade));
            }

            _ => unreachable!(),
        }
    }
}

// Used by dlfcn.h
//
// We need this as the handle must be created and destroyed with the dynamic
// linker's allocator.
pub struct ObjectHandle(*const DSO);

impl ObjectHandle {
    #[inline]
    fn new(obj: Arc<DSO>) -> Self {
        Self(Arc::into_raw(obj))
    }

    #[inline]
    fn into_inner(self) -> Arc<DSO> {
        unsafe { Arc::from_raw(self.0) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const c_void {
        self.0.cast()
    }

    #[inline]
    pub fn from_ptr(ptr: *const c_void) -> Option<Self> {
        NonNull::new(ptr as *mut DSO).map(|ptr| Self(ptr.as_ptr()))
    }
}

impl AsRef<DSO> for ObjectHandle {
    #[inline]
    fn as_ref(&self) -> &DSO {
        unsafe { &*self.0 }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Default)]
    pub struct DebugFlags: u32 {
        /// Display what objects and where they are being loaded.
        const LOAD = 1 << 1;
        /// Display library search paths.
        const SEARCH = 1 << 2;
        /// Display scope information.
        const SCOPES = 1 << 3;
    }
}

#[derive(Default)]
pub struct Config {
    debug_flags: DebugFlags,
    library_path: Option<String>,
    /// Resolve symbols at program startup.
    bind_now: bool,
}

impl Config {
    pub fn from_env(env: &BTreeMap<String, String>) -> Self {
        let debug_flags = env
            .get("LD_DEBUG")
            .map(|value| {
                let mut flags = DebugFlags::empty();
                for opt in value.split(',') {
                    flags |= match opt {
                        "load" => DebugFlags::LOAD,
                        "search" => DebugFlags::SEARCH,
                        "scopes" => DebugFlags::SCOPES,
                        "all" => DebugFlags::all(),
                        _ => {
                            eprintln!("[ld.so]: unknown debug flag '{}'", opt);
                            DebugFlags::empty()
                        }
                    };
                }

                flags
            })
            .unwrap_or(DebugFlags::empty());

        Self {
            debug_flags,
            library_path: env.get("LD_LIBRARY_PATH").cloned(),
            bind_now: env
                .get("LD_BIND_NOW")
                .map(|value| !value.is_empty())
                .unwrap_or_default(),
        }
    }
}

pub struct Linker {
    config: Config,

    next_object_id: usize,
    next_tls_module_id: usize,
    tls_size: usize,
    objects: BTreeMap<usize, Arc<DSO>>,
    name_to_object_id_map: BTreeMap<String, usize>,
    pub cbs: Rc<RefCell<LinkerCallbacks>>,
}

const ROOT_ID: usize = 1;

impl Linker {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            next_object_id: ROOT_ID,
            next_tls_module_id: 1,
            tls_size: 0,
            objects: BTreeMap::new(),
            name_to_object_id_map: BTreeMap::new(),
            cbs: Rc::new(RefCell::new(LinkerCallbacks::new())),
        }
    }

    pub fn load_program(&mut self, path: &str, base_addr: Option<usize>) -> Result<usize> {
        let dso = self.load_object(
            path,
            &None,
            base_addr,
            false,
            if self.config.bind_now {
                Resolve::Now
            } else {
                Resolve::default()
            },
            ObjectScope::Global,
        )?;
        Ok(dso.entry_point)
    }

    pub fn load_library(
        &mut self,
        name: Option<&str>,
        resolve: Resolve,
        scope: ObjectScope,
        noload: bool,
    ) -> Result<ObjectHandle> {
        trace!(
            "[ld.so] dlopen({:?}, {:#?}, {:#?}, noload={})",
            name,
            resolve,
            scope,
            noload
        );

        if noload && resolve == Resolve::Now {
            // Do not perform lazy binding anymore.
            // * Check if loaded with Resolve::Now and if so, early return.
            // * If not, resolve all symbols now.
            todo!("resolve symbols now!");
        }

        match name {
            Some(name) => {
                if let Some(id) = self.name_to_object_id_map.get(name) {
                    let obj = self.objects.get_mut(id).unwrap();

                    // We may be upgrading the object from a local scope to the
                    // global scope.
                    if scope == ObjectScope::Global {
                        if self.config.debug_flags.contains(DebugFlags::SCOPES) {
                            eprintln!("[ld.so]: moving {} into the global scope", obj.name);
                        }

                        let mut global_scope = GLOBAL_SCOPE.write();
                        obj.scope.move_into(&mut global_scope);
                    }

                    Ok(ObjectHandle::new(obj.clone()))
                } else if !noload {
                    let parent_runpath = &self
                        .objects
                        .get(&ROOT_ID)
                        .and_then(|parent| parent.runpath.clone());

                    Ok(ObjectHandle::new(self.load_object(
                        name,
                        parent_runpath,
                        None,
                        true,
                        if self.config.bind_now {
                            Resolve::Now
                        } else {
                            resolve
                        },
                        scope,
                    )?))
                } else {
                    // FIXME: LoadError?
                    // Err(Error::Malformed(format!(
                    //     "object '{}' has not yet been loaded",
                    //     name
                    // )))
                    Ok(ObjectHandle(ptr::null()))
                }
            }

            None => Ok(ObjectHandle::new(
                self.objects
                    .get(&ROOT_ID)
                    .expect("root object missing")
                    .clone(),
            )),
        }
    }

    pub fn get_sym(&self, handle: Option<ObjectHandle>, name: &str) -> Option<*mut c_void> {
        let guard;

        if let Some(handle) = handle.as_ref() {
            &handle.as_ref().scope
        } else {
            guard = GLOBAL_SCOPE.read();
            &guard
        }
        .get_sym(name)
        .map(|(symbol, _, obj)| {
            if symbol.sym_type != STT_TLS {
                symbol.as_ptr()
            } else {
                let mut tls_index = dl_tls_index {
                    ti_module: obj.tls_module_id as u64,
                    ti_offset: symbol.value as u64,
                };

                unsafe { __tls_get_addr(&mut tls_index) }
            }
        })
    }

    pub fn unload(&mut self, handle: ObjectHandle) {
        let obj = handle.into_inner();
        if !obj.dlopened {
            return;
        }

        trace!(
            "[ld.so] unloading {} (sc={}, wc={})",
            obj.name,
            Arc::strong_count(&obj),
            Arc::weak_count(&obj)
        );

        // One for the reference we have and the other for the one in the
        // objects map.
        if Arc::strong_count(&obj) == 2 {
            // Remove from the global scope.
            match *GLOBAL_SCOPE.write() {
                Scope::Global { ref mut objs } => {
                    objs.retain(|o| !Weak::ptr_eq(o, &Arc::downgrade(&obj)));
                }

                _ => unreachable!(),
            }

            let _ = self.objects.remove(&obj.id).unwrap();
            for dep in obj.dependencies.iter() {
                self.unload(ObjectHandle::new(
                    self.objects
                        .get(self.name_to_object_id_map.get(dep).unwrap())
                        .unwrap()
                        .clone(),
                ));
            }
            self.name_to_object_id_map.remove(&obj.name);
            assert!(Arc::strong_count(&obj) == 1);
            drop(obj);
        }

        // obj is dropped here.
    }

    pub fn fini(&self) {
        for obj in self.objects.values() {
            obj.run_fini();
        }
    }

    fn load_object(
        &mut self,
        path: &str,
        runpath: &Option<String>,
        base_addr: Option<usize>,
        dlopened: bool,
        resolve: Resolve,
        scope: ObjectScope,
    ) -> Result<Arc<DSO>> {
        let resolve = if cfg!(target_arch = "x86_64") {
            resolve
        } else {
            // Lazy binding is not currently supported on non-x86_64 architectures.
            Resolve::Now
        };

        unsafe { _r_debug.state = RTLDState::RT_ADD };
        _dl_debug_state();

        let mut new_objects = Vec::new();
        let mut objects_data = Vec::new();
        let mut tcb_masters = Vec::new();
        let loaded_dso = self.load_objects_recursive(
            path,
            runpath,
            base_addr,
            dlopened,
            &mut new_objects,
            &mut objects_data,
            &mut tcb_masters,
            None,
            scope,
        )?;

        unsafe {
            if !dlopened {
                #[cfg(target_os = "redox")]
                let (tcb, old_tcb) = {
                    use redox_rt::signal::tmp_disable_signals;

                    let old_tcb = Tcb::current().expect("failed to get bootstrap TCB");
                    let new_tcb = Tcb::new(self.tls_size).map_err(|_| DlError::Malformed)?; // This actually allocates TCB, TLS and ABI page.

                    // Stash
                    let new_tls_end = new_tcb.generic.tls_end;
                    let new_tls_len = new_tcb.generic.tls_len;
                    let new_tcb_ptr = new_tcb.generic.tcb_ptr;
                    let new_tcb_len = new_tcb.generic.tcb_len;

                    // Unmap just the TCB page.
                    Sys::munmap(new_tcb as *mut Tcb as *mut c_void, syscall::PAGE_SIZE).unwrap();

                    let new_addr = ptr::addr_of!(*new_tcb) as usize;

                    assert_eq!(
                        syscall::syscall5(
                            syscall::SYS_MREMAP,
                            old_tcb as *mut Tcb as usize,
                            syscall::PAGE_SIZE,
                            new_addr,
                            syscall::PAGE_SIZE,
                            (syscall::MremapFlags::FIXED | syscall::MremapFlags::KEEP_OLD).bits()
                                | (syscall::MapFlags::PROT_READ | syscall::MapFlags::PROT_WRITE)
                                    .bits(),
                        )
                        .expect("mremap: failed to alias TCB"),
                        new_addr,
                    );
                    // XXX: New TCB is now at the same physical address as the old TCB.

                    let _guard = tmp_disable_signals();
                    // Restore
                    new_tcb.generic.tls_end = new_tls_end;
                    new_tcb.generic.tls_len = new_tls_len;
                    new_tcb.generic.tcb_ptr = new_tcb_ptr;
                    new_tcb.generic.tcb_len = new_tcb_len;

                    drop(_guard);
                    (new_tcb, old_tcb as *mut Tcb as *mut c_void)
                };

                #[cfg(not(target_os = "redox"))]
                let tcb = Tcb::new(self.tls_size).map_err(|_| DlError::Malformed)?;

                // We are now loading the main program or its dependencies. The TLS for all initially
                // loaded objects reside in the static TLS block. Depending on the architecture, the
                // static TLS block is either placed before the TP or after the TP.
                let tcb_ptr = tcb as *mut Tcb;

                // Setup the DTVs.
                tcb.setup_dtv(tcb_masters.len());

                for obj in new_objects.iter() {
                    if obj.tls_module_id == 0 {
                        // No TLS for this object.
                        continue;
                    }

                    let dtv_idx = obj.tls_module_id - 1;

                    if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
                        // Below the TP
                        tcb.dtv_mut()[dtv_idx] = tcb_ptr.cast::<u8>().sub(obj.tls_offset);
                    } else {
                        // FIMXE(andypython): Make it above the TP
                        //
                        // tcb.dtv_mut().unwrap()[obj.tls_module_id - 1] =
                        //     tcb_ptr.add(1).cast::<u8>().add(obj.tls_offset);
                        //
                        // FIXME(andypython): https://gitlab.redox-os.org/redox-os/relibc/-/merge_requests/570#note_35788
                        tcb.dtv_mut()[dtv_idx] = tcb.tls_end.sub(tcb.tls_len).add(obj.tls_offset);
                    }
                }

                tcb.append_masters(tcb_masters);
                // Copy the master data into the static TLS block.
                tcb.copy_masters().map_err(|_| DlError::Malformed)?;
                tcb.activate();

                #[cfg(target_os = "redox")]
                {
                    // Unmap the old TCB.
                    Sys::munmap(old_tcb, syscall::PAGE_SIZE).unwrap();
                }
            } else {
                let tcb = Tcb::current().expect("failed to get current tcb");

                // TLS variables for dlopen'ed objects are lazily allocated in `__tls_get_addr`.
                tcb.append_masters(tcb_masters);
            }
        }

        // new_objects is already reversed.
        for (i, obj) in new_objects.into_iter().enumerate() {
            self.relocate(&obj, &objects_data[i], resolve)?;
            self.run_init(&obj);
            self.register_object(obj);
        }

        unsafe { _r_debug.state = RTLDState::RT_CONSISTENT };
        _dl_debug_state();

        Ok(loaded_dso)
    }

    fn register_object(&mut self, obj: Arc<DSO>) {
        self.name_to_object_id_map.insert(obj.name.clone(), obj.id);
        self.objects.insert(obj.id, obj);
    }

    fn load_objects_recursive(
        &mut self,
        name: &str,
        parent_runpath: &Option<String>,
        base_addr: Option<usize>,
        dlopened: bool,
        new_objects: &mut Vec<Arc<DSO>>,
        objects_data: &mut Vec<Vec<u8>>,
        tcb_masters: &mut Vec<Master>,
        // The object that caused this object to be loaded.
        dependent: Option<&mut DSO>,
        scope: ObjectScope,
    ) -> Result<Arc<DSO>> {
        // fixme: double lookup slow
        if let Some(id) = self.name_to_object_id_map.get(name) {
            if let Some(obj) = self.objects.get_mut(id) {
                return Ok(obj.clone());
            }
        } else if let Some(obj) = new_objects.iter_mut().find(|o| o.name == name) {
            return Ok(obj.clone());
        }

        let debug = self.config.debug_flags.contains(DebugFlags::LOAD);

        let path = self.search_object(name, parent_runpath)?;
        let data = self.read_file(&path)?;
        let (mut obj, tcb_master) = DSO::new(
            &path,
            &data,
            base_addr,
            dlopened,
            self.next_object_id,
            self.next_tls_module_id,
            self.tls_size,
        )
        .map_err(|err| {
            if debug {
                eprintln!("[ld.so]: failed to load '{}': {}", name, err)
            }

            DlError::Malformed
        })?;

        if debug {
            eprintln!(
                "[ld.so]: loading object: {} at {:#x}",
                name,
                obj.mmap.as_ptr() as usize
            );
        }

        self.next_object_id += 1;

        if let Some(master) = tcb_master {
            if !dlopened {
                self.tls_size += master.offset; // => aligned ph.p_memsz
            }

            tcb_masters.push(master);
            self.next_tls_module_id += 1;
        }

        let runpath = obj.runpath.clone();
        let dependencies = obj.dependencies.clone();

        for dep_name in dependencies.iter() {
            self.load_objects_recursive(
                dep_name,
                &runpath,
                None,
                dlopened,
                new_objects,
                objects_data,
                tcb_masters,
                Some(&mut obj),
                scope,
            )?;
        }

        let obj = Arc::new_cyclic(|sref| {
            obj.scope.set_owner(sref.clone());
            obj
        });

        if let Some(dependent) = dependent {
            match scope {
                ObjectScope::Local => dependent.scope.add(&obj),
                ObjectScope::Global => GLOBAL_SCOPE.write().add(&obj),
            }
        } else if let ObjectScope::Global = scope {
            GLOBAL_SCOPE.write().add(&obj);
        }

        objects_data.push(data);
        new_objects.push(obj.clone());

        Ok(obj)
    }

    fn search_object(&self, name: &str, parent_runpath: &Option<String>) -> Result<String> {
        let debug = self.config.debug_flags.contains(DebugFlags::SEARCH);
        if debug {
            eprintln!("[ld.so]: looking for '{}'", name);
        }

        let mut full_path = name.to_string();
        if accessible(&full_path, F_OK).is_ok() {
            if debug {
                eprintln!("[ld.so]: found at '{}'!", full_path);
            }
            return Ok(full_path);
        } else {
            let mut search_paths = Vec::new();
            if let Some(runpath) = parent_runpath {
                search_paths.extend(runpath.split(PATH_SEP));
            }
            if let Some(ld_path) = self.config.library_path.as_ref() {
                search_paths.extend(ld_path.split(PATH_SEP));
            }
            search_paths.push("/lib");
            for part in search_paths.iter() {
                full_path = format!("{}/{}", part, name);
                if debug {
                    eprintln!("[ld.so]: trying path '{}'", full_path);
                }
                if accessible(&full_path, F_OK).is_ok() {
                    if debug {
                        eprintln!("[ld.so]: found at '{}'!", full_path);
                    }
                    return Ok(full_path);
                }
            }
        }

        if debug {
            eprintln!("[ld.so]: failed to locate '{}'", name);
        }

        Err(DlError::NotFound)
    }

    fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let debug = self.config.debug_flags.contains(DebugFlags::SEARCH);

        let mut data = Vec::new();
        let path_c = CString::new(path).map_err(|err| {
            if debug {
                eprintln!("[ld.so]: invalid path '{}': {}", path, err)
            }

            DlError::NotFound
        })?;

        let flags = fcntl::O_RDONLY | fcntl::O_CLOEXEC;
        let mut file = File::open(CStr::borrow(&path_c), flags).map_err(|err| {
            if debug {
                eprintln!("[ld.so]: failed to open '{}': {}", path, err)
            }

            DlError::NotFound
        })?;

        file.read_to_end(&mut data).map_err(|err| {
            if debug {
                eprintln!("[ld.so]: failed to read '{}': {}", path, err)
            }

            DlError::NotFound
        })?;

        Ok(data)
    }

    /// Perform lazy relocations.
    fn lazy_relocate(&self, obj: &Arc<DSO>, elf: &Elf, resolve: Resolve) -> Result<()> {
        let Some(got) = obj.got() else { return Ok(()) };
        let object_base_addr = obj.mmap.as_ptr() as u64;
        let debug = self.config.debug_flags.contains(DebugFlags::LOAD);

        unsafe {
            got.add(1).write(Arc::as_ptr(obj) as usize);
            got.add(2).write(__plt_resolve_trampoline as usize);
        }

        for rel in elf.pltrelocs.iter() {
            let ptr = if is_pie_enabled(elf) {
                (object_base_addr + rel.r_offset) as *mut u64
            } else {
                rel.r_offset as *mut u64
            };

            match (rel.r_type, resolve) {
                (reloc::R_X86_64_JUMP_SLOT, Resolve::Lazy) if is_pie_enabled(elf) => unsafe {
                    *ptr += object_base_addr;
                },

                (reloc::R_X86_64_JUMP_SLOT, Resolve::Lazy) => {
                    // NOP.
                }

                (reloc::R_X86_64_JUMP_SLOT, Resolve::Now) => {
                    let sym = elf.dynsyms.get(rel.r_sym).ok_or_else(|| {
                        if debug {
                            eprintln!("missing symbol for relocation {:?}", rel)
                        }

                        DlError::Malformed
                    })?;

                    let name = elf.dynstrtab.get_at(sym.st_name).ok_or_else(|| {
                        if debug {
                            eprintln!("missing name for symbol {:?}", sym)
                        }

                        DlError::Malformed
                    })?;

                    let resolved = GLOBAL_SCOPE
                        .read()
                        .get_sym(name)
                        .or_else(|| obj.scope.get_sym(name))
                        .map(|(sym, _, _)| sym.as_ptr())
                        .expect("unresolved symbol");

                    let addend = rel.r_addend.unwrap_or(0) as u64;

                    unsafe {
                        *ptr = resolved as u64 + addend;
                    }
                }

                _ => todo!("unsupported relocation type {:?}", rel.r_type),
            }
        }

        Ok(())
    }

    fn relocate(&self, obj: &Arc<DSO>, object_data: &[u8], resolve: Resolve) -> Result<()> {
        // Perform static relocations.
        let elf = Elf::parse(object_data).or(Err(DlError::Malformed))?;
        let debug = self.config.debug_flags.contains(DebugFlags::LOAD); // FIXME

        trace!("link {}", obj.name);

        let mmap = &obj.mmap;
        let b = mmap.as_ptr() as usize;

        // Relocate
        for rel in elf.dynrelas.iter().chain(elf.dynrels.iter()) {
            trace!(
                "  rel {}: {:x?}",
                reloc::r_to_str(rel.r_type, elf.header.e_machine),
                rel
            );

            let symbol = if rel.r_sym > 0 {
                let sym = elf.dynsyms.get(rel.r_sym).ok_or_else(|| {
                    if debug {
                        eprintln!("[ld.so]: missing symbol for relocation {:?}", rel)
                    }

                    DlError::Malformed
                })?;

                let name = elf.dynstrtab.get_at(sym.st_name).ok_or_else(|| {
                    if debug {
                        eprintln!("[ld.so]: missing name for symbol {:?}", sym)
                    }

                    DlError::Malformed
                })?;

                let symbol = GLOBAL_SCOPE
                    .read()
                    .get_sym(name)
                    .or_else(|| obj.scope.get_sym(name))
                    .map(|(sym, _, obj)| (sym, obj.tls_offset));

                // TODO: below doesn't work because of missing __preinit_array_{start,end} and __init_array_{start,end} symbols in dynamic linked programs
                // TODO: found := symbol.is_some()
                /*
                if !found {
                    return Err(Error::Malformed(format!("missing symbol for name {}", name)));
                }
                */
                symbol
            } else {
                None
            };

            let (s, t) = symbol
                .as_ref()
                .map(|(sym, t)| (sym.as_ptr() as usize, *t))
                .unwrap_or((0, 0));

            let a = rel.r_addend.unwrap_or(0) as usize;

            let ptr = if is_pie_enabled(&elf) {
                (b + rel.r_offset as usize) as *mut u8
            } else {
                rel.r_offset as *mut u8
            };
            let set_u64 = |value| {
                trace!(
                    "    set_u64 r_type={} value={:#x} ptr={:p} base={:#x} mmap_len={:#x} a={:#x}",
                    rel.r_type,
                    value,
                    ptr,
                    b,
                    mmap.len(),
                    a
                );
                unsafe {
                    *(ptr as *mut u64) = value;
                }
            };

            match rel.r_type {
                reloc::R_X86_64_64 => {
                    set_u64((s + a) as u64);
                }
                reloc::R_X86_64_DTPMOD64 => {
                    set_u64(obj.tls_module_id as u64);
                }
                reloc::R_X86_64_DTPOFF64 => {
                    if s != 0 {
                        set_u64((s - b) as u64);
                    } else {
                        set_u64(s as u64);
                    }
                }
                reloc::R_X86_64_GLOB_DAT | reloc::R_X86_64_JUMP_SLOT => {
                    set_u64(s as u64);
                }
                reloc::R_X86_64_RELATIVE => {
                    set_u64((b + a) as u64);
                }
                reloc::R_X86_64_TPOFF64 => {
                    if rel.r_sym > 0 {
                        let (sym, _) = symbol
                            .as_ref()
                            .expect("R_X86_64_TPOFF64 called without valid symbol");
                        set_u64((sym.value + a).wrapping_sub(t) as u64);
                    } else {
                        set_u64(a.wrapping_sub(t) as u64);
                    }
                }
                reloc::R_X86_64_IRELATIVE => unsafe {
                    let f: unsafe extern "C" fn() -> u64 = transmute(b + a);
                    set_u64(f());
                },
                reloc::R_X86_64_COPY => unsafe {
                    let (sym, _) = symbol
                        .as_ref()
                        .expect("R_X86_64_COPY called without valid symbol");
                    ptr::copy_nonoverlapping(sym.as_ptr() as *const u8, ptr, sym.size);
                },
                _ => {
                    panic!(
                        "    {} unsupported",
                        reloc::r_to_str(rel.r_type, elf.header.e_machine)
                    );
                }
            }
        }

        self.lazy_relocate(obj, &elf, resolve)?;

        // Protect pages
        for ph in elf
            .program_headers
            .iter()
            .filter(|ph| ph.p_type == program_header::PT_LOAD)
        {
            let voff = ph.p_vaddr % ph.p_align;
            let vaddr = (ph.p_vaddr - voff) as usize;
            let vsize = ((ph.p_memsz + voff) as usize).next_multiple_of(ph.p_align as usize);
            let mut prot = 0;
            if ph.p_flags & program_header::PF_R == program_header::PF_R {
                prot |= sys_mman::PROT_READ;
            }

            // W ^ X. If it is executable, do not allow it to be writable, even if requested
            if ph.p_flags & program_header::PF_X == program_header::PF_X {
                prot |= sys_mman::PROT_EXEC;
            } else if ph.p_flags & program_header::PF_W == program_header::PF_W {
                prot |= sys_mman::PROT_WRITE;
            }

            unsafe {
                let ptr = if is_pie_enabled(&elf) {
                    mmap.as_ptr().add(vaddr)
                } else {
                    vaddr as *const u8
                };
                trace!("  prot {:#x}, {:#x}: {:p}, {:#x}", vaddr, vsize, ptr, prot);
                Sys::mprotect(ptr as *mut c_void, vsize, prot).map_err(|err| {
                    if debug {
                        eprintln!("[ld.so]: failed to mprotect: {}", err)
                    }

                    DlError::Malformed
                })?;
            }
        }

        Ok(())
    }

    fn run_init(&self, obj: &DSO) {
        use crate::platform::{self, types::*};

        if let Some((symbol, SymbolBinding::Global)) = obj.get_sym("__relibc_init_environ") {
            unsafe {
                symbol
                    .as_ptr()
                    .cast::<*mut *mut c_char>()
                    .write(platform::environ);
            }
        }

        obj.run_init();
    }
}

// GOT[1] = object_id
// GOT[2] = __plt_resolve_trampoline
//
// The stubs in .plt will push the relocation index and the object pointer onto
// the stack and jump to [`__plt_resolve_trampoline`]. The trampoline will then
// call this function to resolve the symbol and update the respective GOT entry.
// The trampoline will then jump to the resolved symbol.
//
// FIXME(andypython): 32-bit
extern "C" fn __plt_resolve_inner(obj: *const DSO, relocation_index: c_uint) -> *mut c_void {
    let resolve_sym = |name: &str, scopes: &[&Scope]| -> Option<Symbol> {
        scopes
            .iter()
            .find_map(|scope| scope.get_sym(name))
            .map(|(sym, _, _)| sym)
    };

    let obj = unsafe { &*obj };
    let obj_base = obj.mmap.as_ptr() as usize;
    let dynamic = obj.dynamic.as_ref().unwrap();
    let jmprel = dynamic.info.jmprel;

    let rela = unsafe {
        &*((obj_base + jmprel) as *const reloc::reloc64::Rela).add(relocation_index as usize)
    };

    assert_eq!(
        reloc::reloc64::r_type(rela.r_info),
        reloc::R_X86_64_JUMP_SLOT
    );

    let sym = obj
        .dynsyms
        .get(reloc::reloc64::r_sym(rela.r_info) as usize)
        .expect("symbol not found");
    assert_ne!(sym.st_name, 0);

    let strtab_offset = dynamic
        .dyns
        .iter()
        .find(|r#dyn| r#dyn.d_tag == DT_STRTAB)
        .unwrap()
        .d_val;

    let name = unsafe {
        CStr::from_ptr(
            (strtab_offset + sym.st_name as u64 + if obj.pie { obj_base as u64 } else { 0u64 })
                as *const c_char,
        )
    };

    let resolved = resolve_sym(name.to_str().unwrap(), &[&GLOBAL_SCOPE.read(), &obj.scope])
        .unwrap_or_else(|| panic!("symbol '{}' not found", name.to_str().unwrap()))
        .as_ptr();

    trace!(
        "@plt: {} -> *mut {:#x}",
        name.to_string_lossy(),
        resolved as usize
    );

    let ptr = if obj.pie {
        (obj_base as u64 + rela.r_offset) as *mut u64
    } else {
        rela.r_offset as *mut u64
    };

    unsafe { *ptr = resolved as u64 }
    resolved
}

extern "C" {
    fn __plt_resolve_trampoline() -> usize;
}

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    "
.global __plt_resolve_trampoline
.hidden __plt_resolve_trampoline
__plt_resolve_trampoline:
    push    rsi
    push    rdi
 
    mov     rdi, qword ptr [rsp + 0x10]
    mov     rsi, qword ptr [rsp + 0x18]

    // stash the floating point argument registers
    sub     rsp, 128
    movdqu  [rsp + 0x00], xmm0
    movdqu  [rsp + 0x10], xmm1
    movdqu  [rsp + 0x20], xmm2
    movdqu  [rsp + 0x30], xmm3
    movdqu  [rsp + 0x40], xmm4
    movdqu  [rsp + 0x50], xmm5
    movdqu  [rsp + 0x60], xmm6
    movdqu  [rsp + 0x70], xmm7

    push   rax
    push   rcx
    push   rdx
    push   r8
    push   r9
    push   r10

    push   rbp
    mov    rbp, rsp
    and    rsp, 0xfffffffffffffff0
    call   {__plt_resolve_inner}
    mov    r11, rax
    mov    rsp, rbp
    pop    rbp

    pop    r10
    pop    r9
    pop    r8
    pop    rdx
    pop    rcx
    pop    rax

    movdqu  xmm7, [rsp + 0x70]
    movdqu  xmm6, [rsp + 0x60]
    movdqu  xmm5, [rsp + 0x50]
    movdqu  xmm4, [rsp + 0x40]
    movdqu  xmm3, [rsp + 0x30]
    movdqu  xmm2, [rsp + 0x20]
    movdqu  xmm1, [rsp + 0x10]
    movdqu  xmm0, [rsp + 0x00]
    add     rsp, 128

    pop    rdi
    pop    rsi

    add    rsp, 0x10
    jmp    r11

    ud2
.size __plt_resolve_trampoline, . - __plt_resolve_trampoline
",
    __plt_resolve_inner = sym __plt_resolve_inner
);

#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    "
.global __plt_resolve_trampoline
.hidden __plt_resolve_trampoline
__plt_resolve_trampoline:
    ud2
.size __plt_resolve_trampoline, . - __plt_resolve_trampoline
    "
);

#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(
    "
.global __plt_resolve_trampoline
.hidden __plt_resolve_trampoline
__plt_resolve_trampoline:
    udf #0
.size __plt_resolve_trampoline, . - __plt_resolve_trampoline
    "
);

#[cfg(target_arch = "riscv64")]
core::arch::global_asm!(
    "
.global __plt_resolve_trampoline
.hidden __plt_resolve_trampoline
__plt_resolve_trampoline:
    unimp
.size __plt_resolve_trampoline, . - __plt_resolve_trampoline
    "
);
