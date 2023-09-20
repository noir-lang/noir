use futures::{AsyncRead, AsyncWrite};
use std::io::{self, IoSlice, IoSliceMut, Read, StdinLock, StdoutLock, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

// WASI-compatible stdin/stdout that implements AsyncRead and AsyncWrite
// Based on the PipeStdin and PipeStdout in async_lsp
// TODO: Upstream to async_lsp
#[derive(Debug)]
pub struct PipeStdin {
    inner: StdinLock<'static>,
}

impl PipeStdin {
    pub fn lock() -> io::Result<Self> {
        let inner = io::stdin().lock();
        Ok(Self { inner })
    }
}

impl AsFd for PipeStdin {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

impl AsRawFd for PipeStdin {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl Read for &'_ PipeStdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        rustix::io::read(self, buf).map_err(Into::into)
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        rustix::io::readv(self, bufs).map_err(Into::into)
    }
}

impl Read for PipeStdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        <&PipeStdin>::read(&mut &*self, buf)
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        <&PipeStdin>::read_vectored(&mut &*self, bufs)
    }
}

#[derive(Debug)]
pub struct PipeStdout {
    inner: StdoutLock<'static>,
}

impl PipeStdout {
    pub fn lock() -> io::Result<Self> {
        let inner = io::stdout().lock();
        Ok(Self { inner })
    }
}

impl AsFd for PipeStdout {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

impl AsRawFd for PipeStdout {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl Write for &'_ PipeStdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        rustix::io::write(self, buf).map_err(Into::into)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        rustix::io::writev(self, bufs).map_err(Into::into)
    }
}

impl Write for PipeStdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        <&PipeStdout>::write(&mut &*self, buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        <&PipeStdout>::flush(&mut &*self)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        <&PipeStdout>::write_vectored(&mut &*self, bufs)
    }
}

impl AsyncRead for PipeStdin {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            match (*self).read(buf) {
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
                res => return Poll::Ready(res),
            }
        }
    }

    fn poll_read_vectored(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        loop {
            match (*self).read_vectored(bufs) {
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
                res => return Poll::Ready(res),
            }
        }
    }
}

impl AsyncWrite for PipeStdout {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            match (*self).write(buf) {
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
                res => return Poll::Ready(res),
            }
        }
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        loop {
            match (*self).write_vectored(bufs) {
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
                res => return Poll::Ready(res),
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        loop {
            match (*self).flush() {
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
                res => return Poll::Ready(res),
            }
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.poll_flush(cx)
    }
}
