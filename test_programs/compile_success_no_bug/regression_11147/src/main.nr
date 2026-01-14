enum Foo<T> {
    A(T),
    B
}
fn main() {
    let fa: Foo<str<5>> = Foo::A("hello");
    let fb: Foo<str<5>> = Foo::B;
    foo(fa);
    foo(fb);
}

fn foo(_f: Foo<str<5>>) {}