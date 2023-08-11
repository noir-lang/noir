use dep::std;

// Simple example of checking where two arrays are equal
fn main(a: [Field; 32], b: [Field; 32]) {
     std::assert_eq(a, b);
}
