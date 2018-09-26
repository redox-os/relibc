use alloc::string::String;
use c_str::CString;
use header::fcntl;
use fs::File;
use io::{BufRead, BufReader};

pub fn get_dns_server() -> String {
    let file = match File::open(
        &CString::new("/etc/resolv.conf").unwrap(),
        fcntl::O_RDONLY
    ) {
        Ok(file) => file,
        Err(_) => return String::new(), // TODO: better error handling
    };
    let mut file = BufReader::new(file);

    for line in file.split(b'\n') {
        let mut line = match line {
            Ok(line) => line,
            Err(_) => return String::new() // TODO: pls handle errors
        };
        if line.starts_with(b"nameserver ") {
            line.drain(..11);
            return String::from_utf8(line).unwrap_or_default();
        }
    }

    // TODO: better error handling
    String::new()
}
