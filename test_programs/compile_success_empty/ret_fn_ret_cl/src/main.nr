fn f(x: Field) -> Field {
    x + 1
}

fn ret_fn() -> fn(Field) -> Field {
    f
}
// TODO: in the advanced implicitly generic function with closures branch
// which would support higher-order functions in a better way
// support returning closures:
//
// fn ret_closure() -> fn(Field) -> Field {
//     let y = 1;
//     let inner_closure = |z| -> Field{
//         z + y
//     };
//     inner_closure
// }
fn ret_lambda() -> fn(Field) -> Field {
    let cl = |z: Field| -> Field { z + 1 };
    cl
}

fn main(x: Field) {
    let result_fn = ret_fn();
    assert(result_fn(x) == x + 1);
    // let result_closure = ret_closure();
    // assert(result_closure(x) == x + 1);
    let result_lambda = ret_lambda();
    assert(result_lambda(x) == x + 1);
}
