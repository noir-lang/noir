mod import;
use crate::import::hello;

fn main(x: Field, y: Field) {
    let _k = dep::std::hash::pedersen_commitment([x]);
    let _l = hello(x);

    assert(x != import::hello(y));
}
