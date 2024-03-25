mod import;
use crate::import::hello;
use import::foo::goodbye;

fn main(x: Field, y: Field) {
    let _k = dep::std::hash::pedersen_commitment([x]);
    let _l = hello(x);
    let _l = goodbye(x);

    assert(x != import::hello(y));
    assert(x != goodbye(y));
}
