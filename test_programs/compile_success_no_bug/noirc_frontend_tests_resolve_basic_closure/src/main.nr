
        fn main(x : Field) -> pub Field {
            let closure = |y| y + x;
            closure(x)
        }
    