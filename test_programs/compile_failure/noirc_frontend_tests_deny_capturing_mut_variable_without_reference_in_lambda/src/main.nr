
    fn main() {
        let mut x = 3;
        let f = || {
            x += 2;
        };
        f();
        assert(x == 5);
    }
    