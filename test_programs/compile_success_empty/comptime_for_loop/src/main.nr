fn main() {
    comptime {
        let u128_val: u128 = 100;
        for _ in 0..u128_val {}

        let i64_val: i64 = 100;
        for _ in 0..i64_val {}
    }
}
