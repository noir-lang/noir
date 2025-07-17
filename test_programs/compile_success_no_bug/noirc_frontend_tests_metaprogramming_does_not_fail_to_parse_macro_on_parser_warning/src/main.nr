
    #[make_bar]
    pub unconstrained fn foo() {}

    comptime fn make_bar(_: FunctionDefinition) -> Quoted {
        quote {
            pub fn bar() {
                unsafe { 
                    foo();
                }
            }
        }
    }

    fn main() {
        bar()
    }
    