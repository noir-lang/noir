// This test should be made to pass and moved to the `compile_success_empty` directory
// https://github.com/noir-lang/noir/issues/5273
fn main() {
    comptime {
       assert_eq(&[1], &[1]);
    }
}
