//! tar.h implementation for Redox, following POSIX.1-1990 specification
//!  and https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/tar.h.html

use core::slice;

/// Block size for tar archives (512 bytes).
pub const BLOCKSIZE: usize = 512;

/// Default record size for tar archives (10KB, consisting of 20 blocks).
pub const RECORDSIZE: usize = BLOCKSIZE * 20; // 10KB (default for tar archives)

/// Field lengths in tar headers
pub const NAME_SIZE: usize = 100; // File name
pub const MODE_SIZE: usize = 8; // File mode
pub const UID_SIZE: usize = 8; // Owner's numeric user ID
pub const GID_SIZE: usize = 8; // Group's numeric user ID
pub const SIZE_SIZE: usize = 12; // File size in bytes
pub const MTIME_SIZE: usize = 12; // Modification time
pub const CHKSUM_SIZE: usize = 8; // Checksum
pub const LINKNAME_SIZE: usize = 100; // Name of linked file
pub const MAGIC_SIZE: usize = 6; // Magic string size
pub const VERSION_SIZE: usize = 2; // Version string size
pub const UNAME_SIZE: usize = 32; // Owner user name
pub const GNAME_SIZE: usize = 32; // Owner group name
pub const DEVMAJOR_SIZE: usize = 8; // Major device number
pub const DEVMINOR_SIZE: usize = 8; // Minor device number
pub const PREFIX_SIZE: usize = 155; // Prefix for file name
pub const HEADER_SIZE: usize = 512; // Total header size

/// Bits used in the mode field - value in octal
pub const TSUID: u16 = 0o4000; // Set user ID on execution
pub const TSGID: u16 = 0o2000; // Set group ID on execution
pub const TSVTX: u16 = 0o1000; // Sticky bit
pub const TUREAD: u16 = 0o0400; // Read permission, owner
pub const TUWRITE: u16 = 0o0200; // Write permission, owner
pub const TUEXEC: u16 = 0o0100; // Execute/search permission, owner
pub const TGREAD: u16 = 0o0040; // Read permission, group
pub const TGWRITE: u16 = 0o0020; // Write permission, group
pub const TGEXEC: u16 = 0o0010; // Execute/search permission, group
pub const TOREAD: u16 = 0o0004; // Read permission, others
pub const TOWRITE: u16 = 0o0002; // Write permission, others
pub const TOEXEC: u16 = 0o0001; // Execute/search permission, others

/// Values used in typeflag field
pub const REGTYPE: u8 = b'0'; // Regular file
pub const AREGTYPE: u8 = b'\0'; // Regular file (old format)
pub const LNKTYPE: u8 = b'1'; // Link
pub const SYMTYPE: u8 = b'2'; // Symbolic link
pub const CHRTYPE: u8 = b'3'; // Character special
pub const BLKTYPE: u8 = b'4'; // Block special
pub const DIRTYPE: u8 = b'5'; // Directory
pub const FIFOTYPE: u8 = b'6'; // FIFO special
pub const CONTTYPE: u8 = b'7'; // Contiguous file

/// tar format magic and version
pub const TMAGIC: &str = "ustar"; // Magic string : ustar and a null
pub const TMAGLEN: usize = 6; // Length of the magic string
pub const TVERSION: &str = "00"; // Version string
pub const TVERSLEN: usize = 2; // Length of the version string

/// Reserved for future standards
pub const XHDRTYPE: u8 = b'x'; // Extended header referring to the next file in the archive
pub const XGLTYPE: u8 = b'g'; // Global extended header

/// Reserved values for GNU tar extensions
// pub const GNUTYPE_DUMPDIR: u8 = b'D'; // Directory dump
// pub const GNUTYPE_MULTIVOL: u8 = b'M'; // Multi-volume file
// pub const GNUTYPE_LONGNAME: u8 = b'L'; // Long file name
// pub const GNUTYPE_LONGLINK: u8 = b'K'; // Long link name
// pub const GNUTYPE_SPARSE: u8 = b'S'; // Sparse file

/// Represents a tar archive header following the POSIX ustar format.
///
/// The header contains metadata about a file in a tar archive, including
/// its name, size, permissions, and other attributes. All text fields are
/// null-terminated.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TarHeader {
    // Byte offset - usage
    pub name: [u8; NAME_SIZE],         // 0   - File name
    pub mode: [u8; MODE_SIZE],         // 100 - Permissions
    pub uid: [u8; UID_SIZE],           // 108 - User ID
    pub gid: [u8; GID_SIZE],           // 116 - Group ID
    pub size: [u8; SIZE_SIZE],         // 124 - File size in bytes
    pub mtime: [u8; MTIME_SIZE],       // 136 - Modification time
    pub chksum: [u8; CHKSUM_SIZE],     // 148 - Header checksum
    pub typeflag: u8,                  // 156 - File type
    pub linkname: [u8; LINKNAME_SIZE], // 157 - Linked file name
    pub magic: [u8; MAGIC_SIZE],       // 257 - UStar magic
    pub version: [u8; VERSION_SIZE],   // 263 - UStar version
    pub uname: [u8; UNAME_SIZE],       // 265 - Owner user name
    pub gname: [u8; GNAME_SIZE],       // 297 - Owner group name
    pub devmajor: [u8; DEVMAJOR_SIZE], // 329 - Major device number
    pub devminor: [u8; DEVMINOR_SIZE], // 337 - Minor device number
    pub prefix: [u8; PREFIX_SIZE],     // 345 - Prefix for file name
    pub padding: [u8; 12],             // 500 - Padding to make 512 bytes
}

impl Default for TarHeader {
    fn default() -> Self {
        let mut header = Self {
            name: [0; NAME_SIZE],
            mode: [0; MODE_SIZE],
            uid: [0; UID_SIZE],
            gid: [0; GID_SIZE],
            size: [0; SIZE_SIZE],
            mtime: [0; MTIME_SIZE],
            chksum: [0; CHKSUM_SIZE],
            typeflag: AREGTYPE,
            linkname: [0; LINKNAME_SIZE],
            magic: [0; MAGIC_SIZE],
            version: [0; VERSION_SIZE],
            uname: [0; UNAME_SIZE],
            gname: [0; GNAME_SIZE],
            devmajor: [0; DEVMAJOR_SIZE],
            devminor: [0; DEVMINOR_SIZE],
            prefix: [0; PREFIX_SIZE],
            padding: [0; 12],
        };

        // Set default magic ("ustar") and version ("00")
        let magic_bytes = TMAGIC.as_bytes(); // "ustar"
        header.magic[..magic_bytes.len()].copy_from_slice(magic_bytes);
        // tar specification often expects "ustar\0"
        if MAGIC_SIZE >= 6 && TMAGIC.len() < MAGIC_SIZE {
            header.magic[TMAGIC.len()] = 0;
        }

        let version_bytes = TVERSION.as_bytes(); // "00"
        header.version[..version_bytes.len()].copy_from_slice(version_bytes);

        header
    }
}

impl TarHeader {
    /// Calculates the checksum of the tar header as required by the specification.
    /// Before computing, the checksum field is treated as if it contained all spaces (0x20).
    pub fn calculate_checksum(&self) -> usize {
        let mut header_copy = *self;
        header_copy.chksum.fill(b' ');
        let bytes =
            unsafe { slice::from_raw_parts(&header_copy as *const _ as *const u8, HEADER_SIZE) };
        bytes.iter().map(|&b| b as usize).sum()
    }
}
