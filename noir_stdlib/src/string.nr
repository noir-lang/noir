impl<N> str<N> {
    /// return a byte slice of string
    #[builtin(str_as_bytes)]
    fn as_bytes(_self: Self) -> [u8] { }
} 