
    fn main() {
        let array = [1];
        mutate(&mut array);
    }

    fn mutate(_: &mut [Field; 1]) {}
    