
    pub fn double<let N: u32>() -> u32 {
        // Used as an expression
        N * 2
    }

    pub fn double_numeric_generics_test() {
        // Example usage of a numeric generic arguments.
        assert(double::<9>() == 18);
        assert(double::<7 + 8>() == 30);
    }

    fn main() { }
    