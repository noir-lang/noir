
    fn main() {
        let mut x = 3;
        let f = || { 
            let inner = || mutate(&mut x); 
            inner();
        };
        f();
        assert(x == 3);
    }

    fn mutate(x: &mut Field) {
        *x = 5;
    }
    