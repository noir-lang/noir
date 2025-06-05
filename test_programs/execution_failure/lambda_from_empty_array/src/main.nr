// Regression from issue #5503 (https://github.com/noir-lang/noir/issues/5503)
fn main() {
    let lambdas: [fn(()) -> (); 0] = [];
    lambdas[0](());
}
