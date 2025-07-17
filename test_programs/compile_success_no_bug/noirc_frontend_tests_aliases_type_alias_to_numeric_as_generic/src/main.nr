
    type Double<let N: u32>: u32 = N * 2;

    pub struct Foo<T, let N: u32> {
        a: T,
        b: [Field; N],
    }
    fn main(x: Field) {
        let a = foo::<4>(x);
        assert(a.a == x);
    }
    fn foo<let N:u32>(x: Field) -> Foo<Field, Double<N>> {
        Foo {
            a: x,
            b: [1; Double::<N>]
        }
    }
    