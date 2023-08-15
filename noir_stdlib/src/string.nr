use crate::collections::vec::Vec;
impl<N> str<N> {
    /// Converts the given string into a byte array
    #[builtin(str_as_bytes)]
    fn as_bytes(_self: Self) -> [u8; N] { }

    /// return a byte vector of the str content
    fn as_bytes_vec(self: Self) -> Vec<u8> {
        Vec::from_slice(self.as_bytes())
    }
} 