type Foo<T> = [T; 2];

type Bar = Field;

type Three = Two<u8>;
type Two<A> = One<A, u32>;
type One<A, B> = (A, B);

struct MyStruct {
    foo: Bar,
}

fn main(x : [Field; 2]) {
    let a: Foo<Field> = [1, 2];
    assert(a[0] != x[0]);

    let b: Bar = 2;
    assert_eq(x[0], b);

    let c: u8 = 1;
    let d: u32 = 2;
    let e: Three = (c, d);
    assert_eq(e.0, 1);

    let s = MyStruct { 
        foo: 10
    };
    assert_eq(s.foo, 10);
}
