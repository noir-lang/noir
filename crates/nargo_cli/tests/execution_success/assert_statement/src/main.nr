use dep::std;

// Tests a very simple program.
// 
// The features being tested is assertion
fn main(x : Field, y : Field)  {
    std::assert_eq(x, y);
}
