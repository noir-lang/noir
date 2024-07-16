struct Foo { 
    some_member: Field
}

fn main() {
    let mut foo = Foo { some_member: 1 };
    foo.some_member = 2;
    let _ = foo.some_member;

    let Foo { some_member } = foo;
    let Foo { some_member: some_var } = foo;
}
