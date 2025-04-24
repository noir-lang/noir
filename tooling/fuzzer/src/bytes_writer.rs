use std::{cell::RefCell, rc::Rc};

/// Similar to [`Vec<u8>`], which implements [`std::io::Write`],
/// but also implements [`Clone`].
#[derive(Clone, Default)]
pub struct BytesWriter {
    bytes: Rc<RefCell<Vec<u8>>>,
}

impl BytesWriter {
    pub fn into_bytes(self) -> Vec<u8> {
        Rc::try_unwrap(self.bytes).unwrap().into_inner()
    }
}

impl std::io::Write for BytesWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.bytes.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.bytes.borrow_mut().flush()
    }
}
