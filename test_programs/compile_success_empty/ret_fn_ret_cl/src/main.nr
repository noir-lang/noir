fn f(x: field) -> field {
    x + 1
}

fn ret_fn() -> fn(field) -> field {
    f
}
// TODO: in the advanced implicitly generic function with closures branch
// which would support higher-order functions in a better way
// support returning closures:
//
// fn ret_closure() -> fn(field) -> field {
//     let y = 1;
//     let inner_closure = |z| -> field{
//         z + y
//     };
//     inner_closure
// }
fn ret_lambda() -> fn(field) -> field {
    let cl = |z: field| -> field {
        z + 1
    };
    cl
}

fn main(x: field) {
    let result_fn = ret_fn();
    assert(result_fn(x) == x + 1);
    // let result_closure = ret_closure();
    // assert(result_closure(x) == x + 1);
    let result_lambda = ret_lambda();
    assert(result_lambda(x) == x + 1);
}
