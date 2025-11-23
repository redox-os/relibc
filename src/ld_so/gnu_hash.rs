// src/ld_so/gnu_hash.rs
//! Implementation of the GNU Hash mechanism for rapid symbol lookup.
//! This allows loading binaries compiled with --hash-style=gnu.

#![no_std]

use core::slice;

/// The GNU Hash header structure.
#[repr(C)]
pub struct GnuHashHeader {
    /// Number of bloom filter words.
    pub nbuckets: u32,
    /// Index of the first symbol in the dynamic symbol table accessible via this hash.
    pub symndx: u32,
    /// The number of words in the bloom filter.
    pub maskwords: u32,
    /// The shift count used in the bloom filter.
    pub shift2: u32,
}

pub struct GnuHash<'a> {
    header: &'a GnuHashHeader,
    bloom: &'a [usize],
    buckets: &'a [u32],
    chain: &'a [u32],
}

impl<'a> GnuHash<'a> {
    /// Parse the DT_GNU_HASH section.
    ///
    /// # Safety
    /// `ptr` must point to a valid DT_GNU_HASH section in memory.
    pub unsafe fn new(ptr: *const u32) -> Self {
        let header = &*(ptr as *const GnuHashHeader);
        
        // Bloom filter follows immediately after header
        let bloom_ptr = ptr.add(4) as *const usize;
        let bloom = slice::from_raw_parts(bloom_ptr, header.maskwords as usize);
        
        // Buckets follow bloom filter
        let buckets_ptr = bloom_ptr.add(header.maskwords as usize) as *const u32;
        let buckets = slice::from_raw_parts(buckets_ptr, header.nbuckets as usize);
        
        // Chain follows buckets. The length is indeterminate until we walk it, 
        // but essentially extends to the end of the dynsym table.
        // We create a slice purely for pointer arithmetic convenience, effectively unbounded safe access relies on symndx.
        let chain_ptr = buckets_ptr.add(header.nbuckets as usize);
        let chain = slice::from_raw_parts(chain_ptr, 0xFFFFFF); // Arbitrary large limit

        Self {
            header,
            bloom,
            buckets,
            chain,
        }
    }

    /// Standard GNU hash function.
    pub fn hash(name: &str) -> u32 {
        let mut h: u32 = 5381;
        for b in name.bytes() {
            h = (h << 5).wrapping_add(h).wrapping_add(b as u32);
        }
        h
    }

    /// Lookup a symbol index. Returns the index in the dynamic symbol table if found.
    pub fn lookup(&self, name: &str, hash: u32, sym_table: &[crate::header::elf::Sym], str_tab: &[u8]) -> Option<u32> {
        // 1. Bloom Filter Check
        let maskwords = self.header.maskwords as usize;
        if maskwords == 0 {
            return None;
        }
        
        let word = self.bloom[(hash as usize / (8 * core::mem::size_of::<usize>())) % maskwords];
        let mask = (1usize << (hash % (8 * core::mem::size_of::<usize>() as u32))) 
                 | (1usize << ((hash >> self.header.shift2) % (8 * core::mem::size_of::<usize>() as u32)));

        if (word & mask) != mask {
            return None; // Definitely not present
        }

        // 2. Bucket Lookup
        let bucket = self.buckets[(hash % self.header.nbuckets) as usize];
        if bucket < self.header.symndx {
            return None;
        }

        // 3. Chain Traversal
        let chain_idx_base = bucket - self.header.symndx;
        let mut i = 0;
        
        loop {
            let hash_entry = self.chain[(chain_idx_base + i) as usize];
            let sym_idx = bucket + i;
            
            // Verify hash match (top 31 bits)
            if (hash | 1) == (hash_entry | 1) {
                // Verify string match
                if let Some(sym) = sym_table.get(sym_idx as usize) {
                    // String comparison logic (helper function assumed to exist in dso/lib)
                    // In a real impl, we'd check symbol versioning here too.
                    if symbol_matches(sym, name, str_tab) {
                        return Some(sym_idx);
                    }
                }
            }

            // LSB indicates end of chain for this bucket
            if (hash_entry & 1) != 0 {
                break;
            }
            i += 1;
        }
        
        None
    }
}

fn symbol_matches(sym: &crate::header::elf::Sym, target: &str, str_tab: &[u8]) -> bool {
    if sym.st_name == 0 { return false; }
    let start = sym.st_name as usize;
    if start >= str_tab.len() { return false; }
    
    // Safe string comparison
    let mut sym_name = &str_tab[start..];
    // Truncate at null byte
    if let Some(end) = sym_name.iter().position(|&c| c == 0) {
        sym_name = &sym_name[..end];
    }
    
    if let Ok(s) = core::str::from_utf8(sym_name) {
        return s == target;
    }
    false
}