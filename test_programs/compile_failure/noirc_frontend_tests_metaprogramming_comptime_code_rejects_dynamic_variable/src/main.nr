
    fn main(x: Field) {
        comptime let my_var = (x - x) + 2;
        assert_eq(my_var, 2);
    }
    