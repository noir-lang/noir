/// Stdout is a wrapper around [`std::io::stdout`] that implements
/// [`std::io::Write`] and [`Clone`].
#[derive(Copy, Clone)]
pub struct Stdout {}

impl std::io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        std::io::stdout().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()
    }
}
