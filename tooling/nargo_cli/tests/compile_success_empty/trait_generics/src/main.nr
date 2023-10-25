
struct Empty<T> {}

trait Foo {
    fn foo(self) -> u32;
}

impl Foo for Empty<u32> {
    fn foo(_self: Self) -> u32 { 32 }
}

impl Foo for Empty<u64> {
    fn foo(_self: Self) -> u32 { 64 }
}

fn main() {
    let x: Empty<u32> = Empty {};
    let y: Empty<u64> = Empty {};
    let z = Empty {};

    assert(x.foo() == 32);
    assert(y.foo() == 64);

    // Types matching multiple impls will currently choose 
    // the first matching one instead of erroring
    assert(z.foo() == 32);
}
