
        fn main() {
            comptime {
                concat!(quote { assert( }, quote { false); });
            }
        }

        comptime fn concat(a: Quoted, b: Quoted) -> Quoted {
            quote { $a $b }
        }
    