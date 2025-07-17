// Regression test for bug that appears to be fixed by https://github.com/noir-lang/noir/pull/7570
pub enum Foo {
    // at least three variants are required, i.e.
    // if e.g. only `A` and `B` are included, the bug goes away
    A,
    B,
    C,
}

fn main() {
    // - the error occured with or without values in the array
    // - the error goes away if `x` is directly defined as e.g. `Foo::A`
    let arena: [Foo; 1] = [Foo::A];
    let x = arena[0];

    // this needs to be in a loop with a positive bound for the error to occur
    for _ in 0..1 {
        match x {
            Foo::A => {
                // the error goes away if this match is removed
                match x {
                    // the error goes away if we only match on Foo::A and/or '_'
                    Foo::B => (),
                    _ => (),
                }
            },
            _ => (),
        }
    }
}
