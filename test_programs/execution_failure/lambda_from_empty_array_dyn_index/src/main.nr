// Regression from issue #5503 (https://github.com/noir-lang/noir/issues/5503),
// except this test uses a dynamic index for the array access.
fn main(x: u32) {
    let lambdas: [fn(()) -> (); 0] = [];
    lambdas[x - 1](());
}

