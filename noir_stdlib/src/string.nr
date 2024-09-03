use crate::collections::vec::Vec;
use crate::convert::From;
use crate::append::Append;

struct String {
    bytes: [u8],
}

impl<let N: u32> str<N> {
    /// Converts the given string into a byte array
    #[builtin(str_as_bytes)]
    pub fn as_bytes(self) -> [u8; N] {}

    /// return a byte vector of the str content
    pub fn as_bytes_vec(self: Self) -> Vec<u8> {
        Vec::from_slice(self.as_bytes().as_slice())
    }
}

impl<let N: u32> From<[u8; N]> for str<N> {
    fn from(bytes: [u8; N]) -> Self {
        bytes.as_str_unchecked()
    }
}

impl<let N: u32> From<str<N>> for String {
    fn from(s: str<N>) -> Self {
        Self { bytes: s.as_bytes() }
    }
}

impl String {
    pub fn new() -> Self {
        String { bytes: &[] }
    }

    pub fn from_bytes(bytes: [u8]) -> Self {
        String { bytes }
    }

    pub fn len(self) -> u32 {
        self.bytes.len()
    }

    pub fn as_str<let N: u32>(self) -> str<N> {
        let len = self.bytes.len();
        assert_eq(N, len, f"Cannot convert a String of length {len} to a str<{N}>");
        self.bytes.as_array::<N>().as_str_unchecked()
    }

    pub fn push(&mut self, byte: u8) {
        self.bytes = self.bytes.push_back(byte);
    }
}

impl Append for String {
    fn empty() -> Self {
        String { bytes: &[] }
    }

    fn append(self, other: Self) -> Self {
        String { bytes: self.bytes.append(other.bytes) }
    }
}
