#[allow(unused_imports)]
// i586 unused: object::NativeEndian, read::elf::Rela, ld_so::dso::resolve_sym, types::c_uint, dso::Rela, Sym
use alloc::{
    collections::BTreeMap,
    rc::Rc,
    string::{String, ToString},
    sync::{Arc, Weak},
    vec::Vec,
};
use object::{
    NativeEndian, elf,
    read::elf::{Rela as _, Sym},
};

use core::{
    cell::RefCell,
    ptr::{self, NonNull},
};

use crate::{
    ALLOCATOR,
    c_str::{CStr, CString},
    error::Errno,
    header::{
        dl_tls::{__tls_get_addr, dl_tls_index},
        fcntl, sys_mman,
        unistd::F_OK,
    },
    ld_so::dso::{SymbolBinding, resolve_sym},
    out::Out,
    platform::{
        Pal, Sys,
        types::{c_int, c_uint, c_void},
    },
    sync::rwlock::RwLock,
};

use super::{
    PATH_SEP,
    access::accessible,
    callbacks::LinkerCallbacks,
    debug::{_dl_debug_state, _r_debug, RTLDState},
    dso::{DSO, ProgramHeader, Rela},
    tcb::{Master, Tcb},
};

#[derive(Debug, Copy, Clone)]
pub enum DlError {
    /// Failed to locate the requested DSO.
    NotFound,
    /// The DSO is malformed somehow.
    Malformed,
    /// Invalid DSO handle.
    InvalidHandle,
    /// Out of memory.
    Oom,
}

impl DlError {
    /// Returns a human-readable, null-terminated C string describing the error.
    pub const fn repr(&self) -> &'static core::ffi::CStr {
        match self {
            DlError::NotFound => {
                c"Failed to locate the requested DSO. Set `LD_DEBUG=all` for more information."
            }

            DlError::Malformed => {
                c"The DSO is malformed somehow. Set `LD_DEBUG=all` for more information."
            }

            DlError::InvalidHandle => {
                c"Invalid DSO handle. Set `LD_DEBUG=all` for more information."
            }

            DlError::Oom => c"Out of memory.",
        }
    }
}

pub type Result<T> = core::result::Result<T, DlError>;

pub(super) static GLOBAL_SCOPE: RwLock<Scope> = RwLock::new(Scope::global());

struct MmapFile {
    fd: i32,
    ptr: *mut c_void,
    size: usize,
}

impl MmapFile {
    fn open(path: CStr, oflag: c_int) -> core::result::Result<Self, Errno> {
        let fd = Sys::open(path, oflag, 0 /* mode */)?;
        let mut stat = crate::header::sys_stat::stat::default();
        Sys::fstat(fd, Out::from_mut(&mut stat))?;

        let size = stat.st_size as usize;
        let ptr = unsafe {
            Sys::mmap(
                ptr::null_mut(),
                size,
                sys_mman::PROT_READ,
                sys_mman::MAP_PRIVATE,
                fd,
                0,
            )
        }?;

        Ok(Self { fd, ptr, size })
    }

    fn data(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr.cast::<u8>(), self.size) }
    }
}

impl Drop for MmapFile {
    fn drop(&mut self) {
        unsafe {
            Sys::munmap(self.ptr, self.size).unwrap();
            Sys::close(self.fd).unwrap();
        }
    }
}

#[derive(Clone, Debug)]
pub struct Symbol<'a> {
    pub name: &'a str,
    pub value: usize,
    pub base: usize,
    pub size: usize,
    pub sym_type: u8,
}

impl Symbol<'_> {
    pub fn as_ptr(&self) -> *mut c_void {
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
pub enum ScopeKind {
    Global,
    Local,
}

pub enum Scope {
    /// The global scope initially contains the main program and all of its
    /// dependencies. Additional objects will be added to this scope via
    /// `dlopen(2)` if the `RTLD_GLOBAL` flag is set.
    Global { objs: Vec<Weak<DSO>> },
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
    const fn local() -> Self {
        Self::Local {
            owner: None,
            objs: Vec::new(),
        }
    }

    fn set_owner(&mut self, obj: Weak<DSO>) {
        match self {
            Self::Global { .. } => panic!("attempted to set global scope owner"),
            Self::Local { owner, .. } => {
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

    pub(super) fn get_sym<'a>(
        &self,
        name: &'a str,
    ) -> Option<(Symbol<'a>, SymbolBinding, Arc<DSO>)> {
        self._get_sym(name, 0)
    }

    pub(super) fn _get_sym<'a>(
        &self,
        name: &'a str,
        skip: usize,
    ) -> Option<(Symbol<'a>, SymbolBinding, Arc<DSO>)> {
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
            Self::Global { objs } => objs
                .iter()
                .skip(skip)
                .map(|o| o.upgrade().unwrap())
                .find_map(get_sym),
            Self::Local { owner, objs } => {
                let owner = owner
                    .as_ref()
                    .expect("local scope without owner")
                    .upgrade()
                    .expect("local scope owner was dropped");

                core::iter::once(owner)
                    .chain(objs.iter().cloned())
                    .skip(skip)
                    .find_map(get_sym)
            }
        }
        .or(res)
    }

    fn copy_into(&self, other: &mut Self) {
        match (self, other) {
            (Self::Local { owner, objs }, Self::Global { objs: other_objs }) => {
                // FIXME: may have duplicates
                let owner = owner.as_ref().expect("local scope without owner");
                other_objs.push(owner.clone());
                other_objs.extend(objs.iter().map(Arc::downgrade));
            }

            _ => unreachable!(),
        }
    }

    fn debug(&self) {
        match self {
            Self::Global { objs } => {
                println!(
                    "[@global] {:?}",
                    objs.iter()
                        .map(|x| x.upgrade().unwrap().name.clone())
                        .collect::<Vec<_>>()
                );
            }

            Self::Local { owner, objs } => {
                let owner = owner.as_ref().unwrap().upgrade().unwrap();
                println!(
                    "[{}] {:?}",
                    owner.name,
                    objs.iter().map(|x| x.name.clone()).collect::<Vec<_>>()
                )
            }
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
            ScopeKind::Global,
        )?;
        Ok(dso.entry_point)
    }

    pub fn load_library(
        &mut self,
        name: Option<&str>,
        resolve: Resolve,
        scope: ScopeKind,
        noload: bool,
    ) -> Result<ObjectHandle> {
        log::trace!(
            "[ld.so] load_library(name={:?}, resolve={:#?}, scope={:#?}, noload={})",
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
                    let obj = self.objects.get(id).unwrap();

                    // We may be upgrading the object from a local scope to the
                    // global scope.
                    if scope == ScopeKind::Global {
                        if self.config.debug_flags.contains(DebugFlags::SCOPES) {
                            eprintln!("[ld.so]: moving {} into the global scope", obj.name);
                        }

                        {
                            let mut global_scope = GLOBAL_SCOPE.write();
                            obj.scope().copy_into(&mut global_scope);
                        }
                        self.scope_debug();
                    }

                    Ok(ObjectHandle::new(obj.clone()))
                } else if !noload {
                    let parent_runpath = &self
                        .objects
                        .get(&ROOT_ID)
                        .and_then(|parent| parent.runpath().map(|path| path.to_string()));

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

            None => match self.objects.get(&ROOT_ID) {
                Some(obj) => Ok(ObjectHandle::new(obj.clone())),
                None => Err(DlError::NotFound),
            },
        }
    }

    pub fn get_sym(&self, handle: Option<ObjectHandle>, name: &str) -> Option<*mut c_void> {
        let guard;

        if let Some(handle) = handle.as_ref() {
            handle.as_ref().scope()
        } else {
            guard = GLOBAL_SCOPE.read();
            &guard
        }
        .get_sym(name)
        .map(|(symbol, _, obj)| {
            if symbol.sym_type != elf::STT_TLS {
                symbol.as_ptr()
            } else {
                let mut tls_index = dl_tls_index {
                    ti_module: obj.tls_module_id,
                    ti_offset: symbol.value,
                };

                unsafe { __tls_get_addr(&raw mut tls_index) }
            }
        })
    }

    pub fn unload(&mut self, handle: ObjectHandle) {
        let obj = handle.into_inner();
        if !obj.dlopened {
            return;
        }

        log::trace!(
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
            for dep in obj.dependencies() {
                self.unload(ObjectHandle::new(
                    self.objects
                        .get(self.name_to_object_id_map.get(*dep).unwrap())
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
        scope: ScopeKind,
    ) -> Result<Arc<DSO>> {
        let resolve = if cfg!(target_arch = "x86_64") {
            resolve
        } else {
            // Lazy binding is not currently supported on non-x86_64 architectures.
            Resolve::Now
        };

        _r_debug.lock().state = RTLDState::RT_ADD;
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

        for (i, obj) in new_objects.iter().enumerate() {
            obj.relocate(&objects_data[i], resolve).unwrap();
        }

        unsafe {
            if !dlopened {
                #[cfg(target_os = "redox")]
                let (tcb, old_tcb, thr_fd) = {
                    use redox_rt::signal::tmp_disable_signals;

                    let old_tcb = Tcb::current().expect("failed to get bootstrap TCB");
                    let thr_fd = (&mut *old_tcb.os_specific.thr_fd.get())
                        .take()
                        .expect("no thread FD present");
                    let new_tcb = Tcb::new(self.tls_size)?; // This actually allocates TCB, TLS and ABI page.

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
                    (new_tcb, old_tcb as *mut Tcb as *mut c_void, thr_fd)
                };

                #[cfg(not(target_os = "redox"))]
                let tcb = Tcb::new(self.tls_size)?;

                // We are now loading the main program or its dependencies. The TLS for all initially
                // loaded objects reside in the static TLS block. Depending on the architecture, the
                // static TLS block is either placed before the TP or after the TP.
                //
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
                        tcb.dtv_mut()[dtv_idx] = tcb.tls_end.sub(obj.tls_offset);
                    } else {
                        // FIMXE(andypython): Make it above the TP
                        //
                        // tcb.dtv_mut().unwrap()[obj.tls_module_id - 1] =
                        //     tcb_ptr.add(1).cast::<u8>().add(obj.tls_offset);
                        //
                        // FIXME(andypython): https://gitlab.redox-os.org/redox-os/relibc/-/merge_requests/570#note_35788
                        let tls_start = tcb.tls_end.sub(tcb.tls_len);
                        tcb.dtv_mut()[dtv_idx] = tls_start.add(obj.tls_offset);
                    }
                }

                tcb.append_masters(tcb_masters);
                // Copy the master data into the static TLS block.
                tcb.copy_masters().map_err(|_| DlError::Malformed)?;
                tcb.activate(
                    #[cfg(target_os = "redox")]
                    Some(thr_fd),
                );
                tcb.mspace = ALLOCATOR.get();

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

        for obj in new_objects.into_iter() {
            obj.mark_ready();
            self.run_init(&obj);
            self.register_object(obj);
        }

        _r_debug.lock().state = RTLDState::RT_CONSISTENT;
        _dl_debug_state();

        Ok(loaded_dso)
    }

    fn register_object(&mut self, obj: Arc<DSO>) {
        self.name_to_object_id_map.insert(obj.name.clone(), obj.id);
        self.objects.insert(obj.id, obj);
    }

    /// Loads the specified object and all of its dependencies.
    ///
    /// `new_objects` contains any new objects that were loaded. Order is
    /// reverse of how the scope is populated.
    ///
    /// The scope is populated such that the loaded objects are in breadth-first
    /// order. This means that first the requested object is added to the scope,
    /// and then its dependencies are added in the order of their respective
    /// `DT_NEEDED` entries in the requested object. This is done recursively
    /// until all dependencies have been loaded.
    ///
    /// If a dependency has already been loaded, it is *not* added to the scope
    /// nor to `new_objects`.
    fn load_objects_recursive<'a>(
        &mut self,
        name: &str,
        parent_runpath: &Option<String>,
        base_addr: Option<usize>,
        dlopened: bool,
        new_objects: &mut Vec<Arc<DSO>>,
        objects_data: &mut Vec<Vec<ProgramHeader>>,
        tcb_masters: &mut Vec<Master>,
        // Scope of the object that caused this object to be loaded.
        dependent_scope: Option<&mut Scope>,
        scope_kind: ScopeKind,
    ) -> Result<Arc<DSO>> {
        // fixme: double lookup slow
        if let Some(id) = self.name_to_object_id_map.get(name) {
            if let Some(obj) = self.objects.get(id) {
                if let Some(scope) = dependent_scope {
                    match scope_kind {
                        ScopeKind::Local => scope.add(obj),
                        ScopeKind::Global => GLOBAL_SCOPE.write().add(obj),
                    }
                } else if scope_kind == ScopeKind::Global {
                    GLOBAL_SCOPE.write().add(obj);
                }
                return Ok(obj.clone());
            }
        } else if let Some(obj) = new_objects.iter().find(|o| o.name == name) {
            if let Some(scope) = dependent_scope {
                match scope_kind {
                    ScopeKind::Local => scope.add(obj),
                    ScopeKind::Global => GLOBAL_SCOPE.write().add(obj),
                }
            } else if scope_kind == ScopeKind::Global {
                GLOBAL_SCOPE.write().add(obj);
            }
            return Ok(obj.clone());
        }

        let debug = self.config.debug_flags.contains(DebugFlags::LOAD);

        let path = self.search_object(name, parent_runpath)?;
        let file = self.read_file(&path)?;
        let data = file.data();
        let (obj, tcb_master, elf) = DSO::new(
            &path,
            data,
            base_addr,
            dlopened,
            self.next_object_id,
            self.next_tls_module_id,
            // Ensure TLS is aligned to 16 bytes for SSE
            self.tls_size.next_multiple_of(16),
        )
        .map_err(|err| {
            if debug {
                eprintln!("[ld.so]: failed to load '{}': {}", name, err)
            }

            DlError::Malformed
        })?;

        if debug {
            eprintln!(
                "[ld.so]: loading object: {} at {:#x}:{:#x} (pie: {})",
                name,
                obj.mmap.as_ptr() as usize,
                obj.mmap.as_ptr() as usize + obj.mmap.len(),
                obj.pie,
            );
        }

        self.next_object_id += 1;

        if let Some(master) = tcb_master {
            if !dlopened {
                self.tls_size = master.offset; // => aligned ph.p_memsz
            }

            tcb_masters.push(master);
            self.next_tls_module_id += 1;
        }

        let runpath = obj.runpath().map(|rpath| rpath.to_string());
        let dependencies = obj
            .dependencies()
            .iter()
            .map(|dep| dep.to_string())
            .collect::<Vec<_>>();

        let obj = Arc::new(obj);
        let mut scope = Scope::local();

        if let Some(dependent_scope) = dependent_scope {
            match scope_kind {
                ScopeKind::Local => dependent_scope.add(&obj),
                ScopeKind::Global => GLOBAL_SCOPE.write().add(&obj),
            }
        } else if let ScopeKind::Global = scope_kind {
            GLOBAL_SCOPE.write().add(&obj);
        }

        for dep_name in dependencies.iter() {
            self.load_objects_recursive(
                dep_name,
                &runpath,
                None,
                dlopened,
                new_objects,
                objects_data,
                tcb_masters,
                Some(&mut scope),
                scope_kind,
            )?;
        }

        objects_data.push(elf);
        new_objects.push(obj.clone());

        scope.set_owner(Arc::downgrade(&obj));
        obj.scope.call_once(|| scope);

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

    fn read_file(&self, path: &str) -> Result<MmapFile> {
        let debug = self.config.debug_flags.contains(DebugFlags::SEARCH);

        let path_c = CString::new(path).map_err(|err| {
            if debug {
                eprintln!("[ld.so]: invalid path '{}': {}", path, err)
            }

            DlError::NotFound
        })?;

        let flags = fcntl::O_RDONLY | fcntl::O_CLOEXEC;
        let file = MmapFile::open(CStr::borrow(&path_c), flags).map_err(|err| {
            if debug {
                eprintln!("[ld.so]: failed to open '{}': {}", path, err)
            }

            DlError::NotFound
        })?;

        Ok(file)
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

    fn scope_debug(&self) {
        if self.config.debug_flags.contains(DebugFlags::SCOPES) {
            println!("[ld.so]: =========== SCOPES ==========");
            GLOBAL_SCOPE.read().debug();
            for obj in self.objects.values() {
                obj.scope().debug();
            }
            println!("[ld.so]: ==============================");
        }
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
#[cfg(target_pointer_width = "64")]
extern "C" fn __plt_resolve_inner(obj: *const DSO, relocation_index: c_uint) -> *mut c_void {
    let obj = unsafe { &*obj };
    let obj_base = obj.mmap.as_ptr() as usize;
    let jmprel = obj.dynamic.jmprel;

    let rela = unsafe { &*(jmprel as *const Rela).add(relocation_index as usize) };
    assert_eq!(rela.r_type(NativeEndian, false), elf::R_X86_64_JUMP_SLOT);

    let sym = obj
        .dynamic
        .symbol(rela.symbol(NativeEndian, false).unwrap())
        .expect("symbol not found");
    assert_ne!(sym.st_name(NativeEndian), 0);

    let name = core::str::from_utf8(
        obj.dynamic
            .dynstrtab
            .get(sym.st_name(NativeEndian))
            .unwrap(),
    )
    .expect("non utf8 symbol name");

    let resolved = resolve_sym(name, &[&GLOBAL_SCOPE.read(), obj.scope()])
        .map(|(sym, _, _)| sym)
        .unwrap_or_else(|| panic!("symbol '{name}' not found"))
        .as_ptr();

    let ptr = if obj.pie {
        (obj_base as u64 + rela.r_offset(NativeEndian)) as *mut u64
    } else {
        rela.r_offset(NativeEndian) as *mut u64
    };
    #[cfg(feature = "trace_tls")]
    log::trace!("@plt: {} -> *mut {:p}", name, ptr);

    unsafe { *ptr = resolved as u64 }
    resolved
}

unsafe extern "C" {
    pub(super) fn __plt_resolve_trampoline() -> usize;
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
