enum Foo<let N: u32> {
    A([u32; N]),
    B
}

fn main() {
    let _: Foo<1> = Foo::B;
    let fb2: Foo<2> = Foo::B;
    let fa2: Foo<2> = Foo::A([1, 2]);
    foo(fb2);
    foo(fa2);
}

fn foo(_f: Foo<2>) {}