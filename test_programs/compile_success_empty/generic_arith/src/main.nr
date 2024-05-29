
fn foo<N>(_x: [Field; N]) -> [Field; N + 1] {
    [0; N + 1]
}

// TODO: working test cases
//
// fn foo_2<N>(x: [Field; N]) -> [Field; N + 1] {
//     foo(x)
// }
//
// fn bar<N>(_x: [Field; N]) -> [Field; N - 1] {
//     [0; N - 1]
// }
//
// fn tail<T, N>(xs: [T; N]) -> [T; N - 1] {
//     let mut out = [dep::std::unsafe::zeroed(); N - 1];
//     for i in 1..N {
//         out[i] = xs[i];
//     }
//     out
// }
//
// fn split_first<T, N>(x: [T; N]) -> (T, [T; N - 1]) {
//     (x[0], tail(x))
// }
//
// struct WrappedArray<T, N> {
//     inner: [T; N]
// }
//
// impl<T, N> WrappedArray<T, N> {
//     fn split_first(self) -> (T, WrappedArray<T, N - 1>) {
//         let result = split_first(self.inner);
//         (result.0, WrappedArray { inner: result.1 })
//     }
// }
//
// trait MyTrait<T> {
//     fn my_trait_method(self) -> [Field; 3] {
//         [0; 3]
//     }
// }
//
// impl MyTrait<0> for () { }
//
// // // expected to fail with
// // // impl's are not allowed to have generic arithmetic arguments
// // impl<T, N> MyTrait<N + 1> for WrappedArray<T, N> { }
//
// // // expected to fail with:
// // // error: impl's are not allowed on generic arithmetic
// // impl<T, N> MyTrait<N> for WrappedArray<T, N + 1> { }

fn main() {

    let input_array: [Field; 2] = [1, 2];
    let foo_result: [Field; 3] = foo(input_array);
    assert(foo_result == [0, 0, 0]);

    // assert(tail([1,2,3]) == [2, 3]);

    // let wrapped_array = WrappedArray { inner: [1,2,3] };
    // assert(wrapped_array.split_first() == (1, [2, 3]));

}


// TODO
//
// trait Serialize<N> {
//     fn serialize(self) -> [Field; N];
// }
//
// struct Foo<T> {
//     a: T,
//     b: T,
// }
//
// impl<T, N> Serialize<N * 2> for Foo<T> {
//     fn serialize(self) -> [Field; N * 2] where T: Serialize<N> {
//       ...
//     }
// }

