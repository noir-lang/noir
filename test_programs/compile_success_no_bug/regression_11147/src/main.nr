enum Foo<T> {
    A(T),
    B
}

struct Bar<T> {
    value: Foo<T>
}

fn main() {
    let _: Foo<bool> = Foo::B;
    let fa: Foo<str<5>> = Foo::A("hello");
    let fb: Foo<str<5>> = Foo::B;
    foo(fa);
    foo(fb);

    let _: Bar<bool> = Bar { value: Foo::B };
    let b: Bar<str<5>> = Bar { value: Foo::B };
    bar(b);
}

fn foo(_f: Foo<str<5>>) {}

fn bar(_b: Bar<str<5>>) {}