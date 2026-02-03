// This test ensures runtime and comptime shift overflow checking are consistent.
// Shifting u64 by 100 bits should overflow in both runtime and comptime.

fn main(x: u64) {
    let _result = x << 100;
}
