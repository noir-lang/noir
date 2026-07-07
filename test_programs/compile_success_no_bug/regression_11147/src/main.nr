enum Foo<T> {
    A(T),
    B,
}

struct Bar<T> {
    value: Foo<T>,
}

enum Either<A, B> {
    Left(A),
    Right(B),
}

fn main() {
    let _: Foo<bool> = Foo::B;
    let fa: Foo<str<5>> = Foo::A("hello");
    let fb: Foo<str<5>> = Foo::B;
    foo(fa);
    foo(fb);

    let mut b: Bar<bool> = Bar { value: Foo::B };
    b.value = Foo::B;

    let b: Bar<str<5>> = Bar { value: Foo::B };
    bar(b);

    let fs: [Foo<str<5>>; 2] = [Foo::B; 2];
    for i in 0..2 {
        foo(fs[i]);
    }

    baz(Either::Left([1, 2]));
    baz(Either::Right(3));
}

fn foo(_f: Foo<str<5>>) {}
fn bar(_b: Bar<str<5>>) {}
fn baz(_e: Either<[u32; 2], u32>) {}
