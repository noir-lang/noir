fn main() {
    for idx_c in 98_u8..100_u8 {
        for idx_d in 0_u8..2_u8 {
            if ((1_u8 >> idx_c) == idx_d) {} else {};
        }
    }
}
