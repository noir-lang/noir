
    fn main() {
        let mut x = &mut 3;
        let f = || {
            *x += 2;
        };
        f();
        assert(*x == 5);
    }
    