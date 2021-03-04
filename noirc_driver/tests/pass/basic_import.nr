mod import;
use dep::std;

use crate::import::hello;

fn main(x : Field, y : Field) {
    priv _k = std::hash::pedersen([x]);
    priv _l = hello(x);

    constrain x != import::hello(y);
}