enum Foo<let N: u32> {
    A(str<N>),
    B,
}

fn main() {
    let _: Foo<1> = Foo::B;
    let fb2: Foo<5> = Foo::B;
    let fa2: Foo<5> = Foo::A("hello");
    foo(fb2);
    foo(fa2);
}

fn foo(_f: Foo<5>) {}
