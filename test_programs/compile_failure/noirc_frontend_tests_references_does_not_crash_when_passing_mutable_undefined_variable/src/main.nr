
    fn main() {
        mutate(&mut undefined);
    }

    fn mutate(foo: &mut Field) {
        *foo = 1;
    }
    