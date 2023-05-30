// This is a test for simple return with a constant.
//
// ACIR generation checks if there is no return 
// type by checking to see if the return type is a numeric
// constant with type None.
// 
// This is just a low hanging test to check this.
//
// Note: There's a possibility in the future that
// a test before ACIR gen optimizes this test.
fn main(_x : Field) -> pub Field {
    5
}