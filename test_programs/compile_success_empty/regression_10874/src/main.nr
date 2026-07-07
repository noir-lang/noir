pub trait Trait {
    type Assoc;
}

#[foo]
comptime fn foo(_: FunctionDefinition) -> Quoted {
    let q = quote { Trait<Assoc = i32> };
    let trait_constraint = q.as_trait_constraint();
    quote {
        impl $trait_constraint for Field {}
    }
}

fn main() {}
