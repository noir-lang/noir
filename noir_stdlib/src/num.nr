impl u32 {

    #[builtin(u32_wrapping_add)]
    fn wrapping_add(_self: Self, rhs: u32) -> u32 {}

    #[builtin(u32_wrapping_sub)]
    fn wrapping_sub(_self: Self, rhs: u32) -> u32 {}
}