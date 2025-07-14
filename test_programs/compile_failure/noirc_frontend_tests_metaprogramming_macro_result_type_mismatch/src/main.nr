
        fn main() {
            comptime {
                let x = unquote!(quote { "test" });
                let _: Field = x;
            }
        }

        comptime fn unquote(q: Quoted) -> Quoted {
            q
        }
    