// ld_so/src/tls.rs
//! Global TLS Management.
//! Maintains the registry of TLS modules available for lazy allocation.
//! Implements Module ID Reuse and Generation Counting.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::RwLock;

/// Information required to allocate a TLS block for a module.
#[derive(Clone, Copy, Debug)]
pub struct TlsModule {
    pub id: usize,
    pub size: usize,
    pub align: usize,
    pub image: usize, // Pointer to .tdata
    pub image_size: usize,
    pub offset: Option<usize>, // For Static TLS optimization
    pub is_static: bool,
}

/// The Global Generation Counter.
/// Incremented whenever a DSO with TLS is loaded/unloaded.
/// Threads check this against their local DTV generation to see if they need updates.
pub static TLS_GENERATION: AtomicUsize = AtomicUsize::new(1);

struct Registry {
    modules: Vec<Option<TlsModule>>,
    free_ids: Vec<usize>,
}

/// Global registry of TLS modules.
static TLS_REGISTRY: RwLock<Registry> = RwLock::new(Registry {
    modules: Vec::new(),
    free_ids: Vec::new(),
});

/// Register a new TLS module.
/// Returns the Module ID.
pub fn register_tls_module(
    size: usize,
    align: usize,
    image: usize,
    image_size: usize,
    offset: Option<usize>,
) -> usize {
    let mut registry = TLS_REGISTRY.write();
    
    // 1. Try to recycle a freed ID
    if let Some(id) = registry.free_ids.pop() {
        registry.modules[id] = Some(TlsModule {
            id,
            size,
            align,
            image,
            image_size,
            offset,
            is_static: offset.is_some(),
        });
        TLS_GENERATION.fetch_add(1, Ordering::SeqCst);
        return id;
    }

    // 2. Allocate new ID
    let id = registry.modules.len();
    
    // Module IDs start at 1 for DTV compatibility (0 is reserved/generation)
    if id == 0 {
        registry.modules.push(None);
        return register_tls_module(size, align, image, image_size, offset);
    }

    registry.modules.push(Some(TlsModule {
        id,
        size,
        align,
        image,
        image_size,
        offset,
        is_static: offset.is_some(),
    }));

    // Bump generation so threads notice the new module
    TLS_GENERATION.fetch_add(1, Ordering::SeqCst);
    
    id
}

/// Unregister a TLS module (called during dlclose).
pub fn unregister_tls_module(id: usize) {
    let mut registry = TLS_REGISTRY.write();
    if id < registry.modules.len() {
        registry.modules[id] = None;
        registry.free_ids.push(id);
        
        // Bump generation so threads know to clean up this slot if they touch it
        TLS_GENERATION.fetch_add(1, Ordering::SeqCst);
    }
}

/// Retrieve module info for lazy allocation.
pub fn get_tls_module(id: usize) -> Option<TlsModule> {
    let registry = TLS_REGISTRY.read();
    if id < registry.modules.len() {
        registry.modules[id]
    } else {
        None
    }
}

/// Get the current maximum module ID (used for DTV sizing).
pub fn max_module_id() -> usize {
    TLS_REGISTRY.read().modules.len().saturating_sub(1)
}