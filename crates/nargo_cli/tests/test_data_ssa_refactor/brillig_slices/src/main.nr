use dep::std::slice;
use dep::std;

unconstrained fn main() {
    let mut slice: [Field] = [];
    assert(slice.len() == 0);

    slice = slice.push_back(1);
    assert(slice.len() == 1);
}

