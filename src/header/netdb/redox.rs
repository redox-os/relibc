use crate::{
    error::Errno,
    fs::File,
    header::{errno, fcntl},
    io::Read,
};
use alloc::string::String;

pub fn get_dns_server() -> Result<String, Errno> {
    let mut string = String::new();
    let mut file = File::open(c"/etc/net/dns".into(), fcntl::O_RDONLY)?;
    file.read_to_string(&mut string)
        .map_err(|_| Errno(errno::EIO).sync())?;

    Ok(string)
}
