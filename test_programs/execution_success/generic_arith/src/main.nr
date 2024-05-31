// BEGIN basic function tests
fn foo<N>(_x: [Field; N]) -> [Field; N + 1] {
    [0; N + 1]
}

fn foo_2<N>(x: [Field; N]) -> [Field; N + 1] {
    foo(x)
}

fn bar<N>(_x: [Field; N]) -> [Field; N - 1] {
    [0; N - 1]
}

fn tail<T, N>(xs: [T; N]) -> [T; N - 1] {
    let mut out: [T; N - 1] = [dep::std::unsafe::zeroed(); N - 1];
    for i in 1..N {
        out[i-1] = xs[i];
    }
    out
}

fn backwards_tail<T, N>(xs: [T; N + 1]) -> [T; N] {
    let mut out: [T; N] = [dep::std::unsafe::zeroed(); N];
    for i in 1..(N+1) {
        out[i-1] = xs[i];
    }
    out
}

fn cons<T, N>(x: T, xs: [T; N]) -> [T; N + 1] {
    let mut out: [T; N + 1] = [dep::std::unsafe::zeroed(); N + 1];
    out[0] = x;
    for i in 1..(N+1) {
        out[i] = xs[i-1];
    }
    out
}

fn split_first<T, N>(x: [T; N]) -> (T, [T; N - 1]) {
    (x[0], tail(x))
}
// END basic function tests

// BEGIN trait tests
struct WrappedArray<T, N> {
    inner: [T; N]
}

impl<T, N> WrappedArray<T, N> {
    fn split_first(self) -> (T, WrappedArray<T, N - 1>) {
        let result = split_first(self.inner);
        (result.0, WrappedArray { inner: result.1 })
    }
}

impl<T, N> Eq for WrappedArray<T, N> where T: Eq {
    fn eq(self, other: WrappedArray<T, N>) -> bool {
        self.inner.eq(other.inner)
    }
}

trait MyTrait<T> {
    fn my_trait_method(self) -> [Field; 3] {
        [0; 3]
    }
}

impl MyTrait<0> for () { }
// END trait tests

// BEGIN serialize test
trait Serialize<N> {
    fn serialize(self) -> [Field; N];
}

impl Serialize<0> for () {
    fn serialize(self) -> [Field; 0] {
        []
    }
}

impl Serialize<1> for Field {
    fn serialize(self) -> [Field; 1] {
        [self]
    }
}

struct Foo<T> {
    a: T,
    b: T,
}

// error: impl's are not allowed to have generic arithmetic arguments
//
// impl<T, N> Serialize<N * 2> for Foo<T> {
//     fn serialize_foo<T, N>(x: Foo<T>) -> [Field; N * 2] where T: Serialize<N> {
//         [0; N * 2]
//     }
// }
fn serialize_foo<T, N>(x: Foo<T>) -> [Field; N * 2] where T: Serialize<N> {
    let out_a: [Field; N] = x.a.serialize();
    let out_b: [Field; N] = x.b.serialize();
    let mut result: [Field; N * 2] = [0; N * 2];
    for i in 0..out_a.len() {
        result[i] = out_a[i];
        result[out_a.len() + i] = out_b[i];
    }
    result
}
// END serialize test

fn main() {
    let input_array: [Field; 2] = [1, 2];
    let foo_result: [Field; 3] = foo(input_array);
    assert(foo_result == [0, 0, 0]);


    // tail
    let input_array: [Field; 3] = [1, 2, 3];
    let result: [Field; 2] = tail(input_array);
    assert(result == [2, 3]);

    let result: [Field; 1] = tail(result);
    assert(result == [3]);


    // backwards_tail
    let input_array: [Field; 3] = [1, 2, 3];
    let result: [Field; 2] = backwards_tail(input_array);
    assert(result == [2, 3]);

    let result: [Field; 1] = backwards_tail(result);
    assert(result == [3]);


    // cons
    let input_array: [Field; 0] = [];
    let result: [Field; 1] = cons(2, input_array);
    assert(result == [2]);

    // NOTE: fixed in the elaborator
    //
    // unexpected error when running these in one line:
    //
    // error: Failed to prove generic arithmetic equivalent:
    // Generic arithmetic evaluated differently: Ok(2) != Err(UnboundVariable { .. })
    // test_programs/compile_success_empty/generic_arith/src/main.nr:15:22
    // assert(tail([1,2,3]) == [2, 3]);

    // it also fails when using turbofish 
    //
    // assert(tail::<Field, 3>([1,2,3]) == [2, 3]);

    // it also fails when compared to another function output
    //
    // assert(tail([1,2,3]) == tail([2, 2, 3]));

    let wrapped_array = WrappedArray { inner: [1, 2, 3] };
    let result: (Field, WrappedArray<Field, 2>) = wrapped_array.split_first();
    let expected_result: (Field, WrappedArray<Field, 2>) = (1, WrappedArray { inner: [2, 3] });
    assert(result == expected_result);

    // this fails with what appears to be the same error,
    // viz. that the location is in the _function_ that has already type-checked:
    //
    // error: Failed to prove generic arithmetic equivalent:
    // Generic arithmetic evaluated differently: Ok(2) != Err(UnboundVariable { .. })
    // test_programs/compile_success_empty/generic_arith/src/main.nr:31:49
    //
    // assert(wrapped_array.split_first() == (1, WrappedArray { inner: [2, 3] }));

    // serialize test
    let test_foo = Foo { a: 2, b: 3 };
    let result: [Field; 2] = serialize_foo(test_foo);
    assert(result == [2, 3]);
}
