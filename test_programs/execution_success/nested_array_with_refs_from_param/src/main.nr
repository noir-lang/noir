// Regression for issue #8803 (https://github.com/noir-lang/noir/issues/8803)
unconstrained fn main() -> pub bool {
    false >= *func_3(&mut false)[if *func_3(&mut true)[0][0] { 0 } else { 0 }][0]
}
fn func_3(b: &mut bool) -> [[&mut bool; 1]] {
    [[b]]
}
