use core::fmt::Write;

#[unsafe(no_mangle)]
pub extern "C" fn relibc_panic(pi: &::core::panic::PanicInfo) -> ! {
    use core::fmt::Write;

    let mut w = PanicWriter::new();
    let _ = w.write_fmt(format_args!("RELIBC PANIC: {}\n", pi));

    core::intrinsics::abort();
}

struct PanicWriter;

impl PanicWriter {
    pub fn new() -> Self {
        Self
    }
}

impl Write for PanicWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // Cannot use Sys::write, Tcb might not be available
        #[cfg(target_os = "redox")]
        {
            // TODO: init stderr ourself if it not ready.
            //       Workaround: If stderr is not being prepared,
            //       add debugging log at kernel write() syscall
            let _ = syscall::write(2, s.as_bytes());
            return Ok(());
        }

        // Non-redox platforms can directly write to stderr
        let mut w = crate::platform::FileWriter::new(2);
        w.write_str(s)
    }
}
