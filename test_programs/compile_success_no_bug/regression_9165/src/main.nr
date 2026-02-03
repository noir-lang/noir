unconstrained fn main() {
    for idx_a in 4048979427..4048979430 {
        assert(
            idx_a
                == match idx_a {
                    198384605 => idx_a,
                    _ => 3138276502,
                },
        );
    }
}
