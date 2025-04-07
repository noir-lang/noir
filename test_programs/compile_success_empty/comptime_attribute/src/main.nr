#[add_foo]
mod moo {
    pub comptime fn add_fn(_: FunctionDefinition, name: Quoted) -> Quoted {
        quote {
            pub fn $name() {}
        }
    }
}

comptime fn add_foo(_: Module) -> Quoted {
    let add_fn = moo::add_fn;
    quote {
        #[$add_fn(quote { bar })]
        pub fn foo() {}
    }
}

fn main() {
    moo::foo();
    moo::bar();
}
