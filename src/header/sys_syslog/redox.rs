use crate::{
    error::{Errno, Result},
    fs::File,
    header::fcntl,
    io::BufWriter,
};

use super::logger::LogSink;

/// Write logs to Redox's log scheme.
pub struct LogFile(BufWriter<File>);

impl LogSink for LogFile {
    type Sink = BufWriter<File>;

    #[inline(always)]
    fn open() -> Result<Self> {
        File::open(c"/scheme/log".into(), fcntl::O_WRONLY).map(|file| Self(BufWriter::new(file)))
    }

    #[inline(always)]
    fn writer(&mut self) -> &mut Self::Sink {
        &mut self.0
    }
}
