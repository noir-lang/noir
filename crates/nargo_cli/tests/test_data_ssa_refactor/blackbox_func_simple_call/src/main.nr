// The feature being tested is a call
// to a black box function.
use dep::std;

fn main(x: Field) {
    std::scalar_mul::fixed_base(x);
}
