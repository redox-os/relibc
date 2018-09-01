use alloc::string::String;
use platform::rawfile::RawFile;
use platform::Line;
use platform::rlb::RawLineBuffer;
use c_str::CString;

pub fn get_dns_server() -> String {
    let fd = match RawFile::open(&CString::new("/etc/resolv.conf").unwrap(), 0, 0) {
        Ok(fd) => fd,
        Err(_) => return String::new() // TODO: better error handling
    };

    let mut rlb = RawLineBuffer::new(*fd);
    while let Line::Some(line) = rlb.next() {
        if line.starts_with(b"nameserver ") {
            return String::from_utf8(line[11..].to_vec()).unwrap_or_default();
        }
    }

    // TODO: better error handling
    String::new()
}
