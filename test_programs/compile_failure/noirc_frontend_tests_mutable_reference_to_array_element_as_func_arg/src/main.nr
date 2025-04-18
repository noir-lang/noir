
    fn foo(x: &mut u32) {
        *x += 1;
    }
    fn main() {
        let mut state: [u32; 4] = [1, 2, 3, 4];
        foo(&mut state[0]);
        assert_eq(state[0], 2); // expect:2 got:1
    }
    