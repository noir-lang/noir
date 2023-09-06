type Foo<T> = [T; 2];

type Bar = Field;

type One<A, B> = (A, B);
type Two<A> = One<A, u32>;
type Three = Two<u8>;

struct MyStruct {
    foo: Bar,
}

fn main(x : [Field; 2]) {
    let a: Foo<Field> = [1, 2];
    assert(a[0] != x[0]);

    let b: Bar = 2;
    assert(x[0] == b);

    let c: u8 = 1;
    let d: u32 = 2;
    let e: Three = (c, d);
    assert(e.0 == 1);

    let s = MyStruct { 
        foo: 10
    };
    assert(s.foo == 10);

    let _regression2502: Regression2502Alias = Regression2502 {};
}

// An ICE was occurring if a type alias referred to a struct before it was initialized
// during name resolution. The fix was to initialize structs during def collection instead.
type Regression2502Alias = Regression2502;
struct Regression2502 {}
