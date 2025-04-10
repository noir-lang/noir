#[add_foo]
mod moo {
    pub comptime fn add_fn(f: FunctionDefinition, name: Quoted) -> Quoted {
        assert(f.has_named_attribute("add_fn"));
        quote {
            pub fn $name() {}
        }
    }
}

comptime fn add_foo(_: Module) -> Quoted {
    let func = moo::add_fn;
    quote {
        #[$func(quote { bar })]
        pub fn foo() {}
    }
}

fn main() {
    moo::foo();
    moo::bar();
}
