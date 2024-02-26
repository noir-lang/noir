mod import;
use crate::import::hello;

fn main(x: field, y: field) {
    let _k = dep::std::hash::pedersen_commitment([x]);
    let _l = hello(x);

    assert(x != import::hello(y));
}
