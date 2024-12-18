#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::tar::{TarError, TarHeader, DIRTYPE, NAME_SIZE, SYMTYPE};

    #[test]
    fn test_header_checksum() {
        let mut header = TarHeader::default();

        // Set fields to valid octal strings for mode, size, mtime
        // mode: 0000644 (octal), commonly used in tar archives
        header.mode = *b"0000644\0";

        // size: 12 chars total, octal "000000000123" fits exactly
        // (no null terminator needed if fully used)
        header.size = *b"000000000123";

        // mtime: 12 chars total, octal time "00000001234\0"
        // Counting chars: '0'(1)'0'(2)'0'(3)'0'(4)'0'(5)'0'(6)'0'(7)'1'(8)'2'(9)'3'(10)'4'(11)'\0'(12)
        header.mtime = *b"00000001234\0";

        // Calculate the checksum
        let checksum = header.calculate_checksum();

        // According to tar specification, the checksum field should contains
        // an octal number followed by a null and a space if possible.
        // For an 8-byte field: 6 octal digits, a null, and a space :
        //
        // Example: "000000\0 " (sum in octal)
        let checksum_str = format!("{:06o}\0 ", checksum);
        header.chksum.copy_from_slice(checksum_str.as_bytes());

        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_header_builder() {
        let header = TarHeader::builder().name("test/file.txt").unwrap().build();

        // The builder sets a null terminator after "test/file.txt"
        let expected = {
            let mut arr = [0u8; NAME_SIZE];
            arr[.."test/file.txt".len()].copy_from_slice(b"test/file.txt");
            arr["test/file.txt".len()] = 0;
            arr
        };

        assert_eq!(header.name, expected);
    }

    #[test]
    fn test_field_too_long() {
        // NAME_SIZE = 100, so a string longer than 99 chars fails
        let long_string = "a".repeat(200);
        let result = TarHeader::to_null_terminated_field::<NAME_SIZE>(&long_string);

        assert!(matches!(result, Err(TarError::FieldTooLong)));
    }

    #[test]
    fn test_invalid_checksum() {
        let mut header = TarHeader::default();
        // Set chksum to something invalid
        header.chksum = *b"99999999"; // Not a valid octal, likely fails
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_file_type() {
        let mut header = TarHeader::default();
        assert_eq!(header.file_type(), "Regular File");

        header.typeflag = DIRTYPE;
        assert_eq!(header.file_type(), "Directory");

        header.typeflag = SYMTYPE;
        assert_eq!(header.file_type(), "Symbolic Link");
    }
}
