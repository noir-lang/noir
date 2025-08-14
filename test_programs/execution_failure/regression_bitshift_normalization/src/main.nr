fn main() {
    for idx_c in 98_u16..100_u16 {
        for idx_d in 0_u16..2_u16 {
            if ((1_u16 >> idx_c) == idx_d) {} else {};
        }
    }
}
