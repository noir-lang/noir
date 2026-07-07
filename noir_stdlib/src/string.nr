use crate::convert::From;

impl<let N: u32> str<N> {
    /// Converts the given string into a byte array
    #[builtin(str_as_bytes)]
    pub fn as_bytes(self) -> [u8; N] {}

    /// return a byte vector of the str content
    pub fn as_bytes_vec(self: Self) -> [u8] {
        self.as_bytes().as_vector()
    }
}

impl<let N: u32> From<[u8; N]> for str<N> {
    fn from(bytes: [u8; N]) -> Self {
        bytes.as_str_unchecked()
    }
}

mod tests {
    #[test]
    fn as_bytes() {
        assert_eq("Noir".as_bytes(), [78_u8, 111, 105, 114]);
    }

    #[test]
    fn as_bytes_vec() {
        assert_eq("Noir".as_bytes_vec(), @[78_u8, 111, 105, 114]);
    }

    #[test]
    fn from_bytes() {
        let str: str<4> = crate::convert::From::from([78_u8, 111, 105, 114]);
        assert_eq(str, "Noir");
    }
}
