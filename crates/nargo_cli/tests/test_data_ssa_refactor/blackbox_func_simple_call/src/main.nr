// The feature being tested is a call
// to a black box function.
// One with array input, one with a numeric input.
use dep::std;

fn main(x: Field) {
    let _p1 = std::scalar_mul::fixed_base(x);
    let _p2 = std::hash::pedersen([x]);
}
