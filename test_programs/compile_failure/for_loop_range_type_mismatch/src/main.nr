fn main() {
    // Range bounds must have the same type
    comptime {
        for _ in 1_i8..2_u8 {}
    }
}
