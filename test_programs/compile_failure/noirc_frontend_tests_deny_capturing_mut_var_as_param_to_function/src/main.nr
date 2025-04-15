
    fn main() {
        let mut x = 3;
        let f = || mutate(&mut x);
        f();
        assert(x == 3);
    }

    fn mutate(x: &mut Field) {
        *x = 5;
    }
    