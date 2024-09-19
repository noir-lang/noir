fn main() {
    comptime
    {
        let signature = "hello".as_ctstring();
        let string = signature.as_quoted_str!();
        let result = half(string);
        assert_eq(result, 2);
    }
}

comptime fn half<let N: u32>(_s: str<N>) -> u32 {
    N / 2
}
