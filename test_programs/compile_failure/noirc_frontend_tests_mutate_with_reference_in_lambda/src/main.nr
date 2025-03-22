
    fn main() {
        let x = &mut 3;
        let f = || {
            *x += 2;
        };
        f();
        assert(*x == 5);
    }
    