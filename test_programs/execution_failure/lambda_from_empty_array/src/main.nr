// Regression from issue #5503 (https://github.com/noir-lang/noir/issues/5503)
fn main() {
    let lambdas: [fn((u32)) -> (u32, [u32; 2]); 0] = [];
    let _ = lambdas[0]((0));

    let lambdas: [fn((u32)) -> (&mut u32); 0] = [];
    let _ = lambdas[0]((1));

    let lambdas: [fn((u32)) -> (fn((u32)) -> ()); 0] = [];
    let _ = lambdas[0]((2));
}
