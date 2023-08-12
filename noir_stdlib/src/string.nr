impl<N> str<N> {
    #[builtin(str_as_bytes)]
    fn as_bytes(_self: Self) -> [u8] { }
} 