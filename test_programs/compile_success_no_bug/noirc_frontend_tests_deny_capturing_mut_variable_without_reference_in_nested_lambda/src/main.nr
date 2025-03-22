
    fn main() {
        let mut x = 3;
        let f = || {
            let inner = || {
                x += 2;
            };
            inner();
        };
        f();
        assert(x == 5);
    }
    