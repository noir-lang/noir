
        fn main() {
            let _zero = Even::Zero;
        }

        enum Even {
            Zero,
            Succ(Odd),
        }

        enum Odd {
            One,
            Succ(Even),
        }
        