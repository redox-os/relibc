//! tar.h implementation for Redox, following POSIX.1-1990 specification
//!  and https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/tar.h.html

//!
//! This module provides functionality for working with tar archives, including
//! header creation, validation, and manipulation. It implements the POSIX.1-1990
//! ustar format.

use core::slice;

#[cfg(test)]
mod tests;

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
pub const GNUTYPE_DUMPDIR: u8 = b'D'; // Directory dump
pub const GNUTYPE_MULTIVOL: u8 = b'M'; // Multi-volume file
pub const GNUTYPE_LONGNAME: u8 = b'L'; // Long file name
pub const GNUTYPE_LONGLINK: u8 = b'K'; // Long link name
pub const GNUTYPE_SPARSE: u8 = b'S'; // Sparse file

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

pub struct TarHeaderBuilder {
    header: TarHeader,
}

impl TarHeaderBuilder {
    /// Creates a new `TarHeaderBuilder` with default values.
    pub fn new() -> Self {
        let mut header = TarHeader::default();
        header.magic.copy_from_slice(TMAGIC.as_bytes());
        header.version.copy_from_slice(TVERSION.as_bytes());
        Self { header }
    }

    /// Sets the name field in the tar header.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name of the file.
    ///
    /// # Errors
    ///
    /// Returns `TarError::FieldTooLong` if the name exceeds the maximum allowed length.
    pub fn name(mut self, name: &str) -> Result<Self, TarError> {
        self.header.name = TarHeader::to_null_terminated_field::<NAME_SIZE>(name)?;
        Ok(self)
    }

    /// Builds and returns the `TarHeader`.
    pub fn build(self) -> TarHeader {
        self.header
    }
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
            typeflag: REGTYPE, // Default to regular file type
            linkname: [0; LINKNAME_SIZE],
            magic: *b"ustar\0", // UStar magic string
            version: *b"00",    // UStar version
            uname: [0; UNAME_SIZE],
            gname: [0; GNAME_SIZE],
            devmajor: [0; DEVMAJOR_SIZE],
            devminor: [0; DEVMINOR_SIZE],
            prefix: [0; PREFIX_SIZE],
            padding: [0; 12],
        };

        // Set default magic and version
        let magic_bytes = TMAGIC.as_bytes(); // "ustar"
        header.magic[..magic_bytes.len()].copy_from_slice(magic_bytes);

        // POSIX ustar magic expects a trailing null
        // TMAGIC = "ustar" (5 chars) + '\0' = 6 chars total
        if TMAGLEN == 6 {
            header.magic[5] = 0;
        }

        // Set the version field to "00"
        let version_bytes = TVERSION.as_bytes(); // "00"
        header.version[..version_bytes.len()].copy_from_slice(version_bytes);

        header
    }
}

impl TarHeader {
    /// Converts a byte array field to a UTF-8 string, trimming null bytes.
    ///
    /// # Returns
    ///
    /// * `Some(&str)` - If the field contains valid UTF-8 data
    /// * `None` - If the field contains invalid UTF-8 data
    pub fn to_str(field: &[u8]) -> Option<&str> {
        let end = field.iter().position(|&b| b == 0).unwrap_or(field.len());
        core::str::from_utf8(&field[..end]).ok()
    }

    /// Converts a string to a null-terminated byte array suitable for a tar header field.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to convert
    ///
    /// # Returns
    ///
    /// * `Ok([u8; N])` - The null-terminated byte array
    /// * `Err(TarError)` - If the string is too long for the field
    pub fn to_null_terminated_field<const N: usize>(s: &str) -> Result<[u8; N], TarError> {
        let mut field = [0u8; N];
        let bytes = s.as_bytes();

        // Reserve one byte for the null terminator
        if bytes.len() + 1 > N {
            return Err(TarError::FieldTooLong);
        }

        field[..bytes.len()].copy_from_slice(bytes);
        field[bytes.len()] = 0; // Null terminator

        Ok(field)
    }

    /// Calculates the checksum of the header according to the POSIX specification.
    ///
    /// The checksum is calculated by summing all bytes in the header, with the
    /// checksum field itself treated as eight spaces (ASCII 32).
    ///
    /// # Returns
    ///
    /// The calculated checksum value
    pub fn calculate_checksum(&self) -> usize {
        let mut header_copy = *self;
        // Replace chksum field with spaces
        header_copy.chksum.fill(b' ');

        let bytes =
            unsafe { slice::from_raw_parts(&header_copy as *const _ as *const u8, HEADER_SIZE) };

        bytes.iter().map(|&b| b as usize).sum()
    }

    /// Validates the header's stored checksum against a calculated checksum.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the checksum is valid
    /// * `Err(TarError)` - If the checksum is invalid or malformed
    pub fn validate(&self) -> Result<(), TarError> {
        let recorded_str = match TarHeader::to_str(&self.chksum) {
            Some(cs) => cs.trim_end(),
            None => return Err(TarError::InvalidChecksum),
        };

        let recorded_checksum =
            usize::from_str_radix(recorded_str, 8).map_err(|_| TarError::InvalidChecksum)?;

        let actual_checksum = self.calculate_checksum();
        if actual_checksum != recorded_checksum {
            return Err(TarError::InvalidChecksum);
        }
        Ok(())
    }

    /// Creates a new TarHeaderBuilder for constructing a TarHeader.
    pub fn builder() -> TarHeaderBuilder {
        TarHeaderBuilder::new()
    }

    /// Returns a human-readable description of the file type.
    ///
    /// # Returns
    ///
    /// A string slice describing the type of file represented by this header.
    pub fn file_type(&self) -> &'static str {
        match self.typeflag {
            REGTYPE | AREGTYPE => "Regular File",
            DIRTYPE => "Directory",
            SYMTYPE => "Symbolic Link",
            LNKTYPE => "Hard Link",
            CHRTYPE => "Character Device",
            BLKTYPE => "Block Device",
            FIFOTYPE => "FIFO",
            CONTTYPE => "Contiguous File",
            _ => "Unknown",
        }
    }
}

#[derive(Debug)]
pub enum TarError {
    /// The checksum in the tar header is invalid.
    InvalidChecksum,
    /// An I/O error occurred.
    IoError,
    /// The tar header is invalid.
    InvalidHeader,
    /// The type flag in the tar header is unknown.
    UnknownTypeFlag(u8),
    /// An error occurred during UTF-8 conversion.
    InvalidEncoding,
    /// The input string exceeds the maximum allowed field size.
    FieldTooLong,
}
