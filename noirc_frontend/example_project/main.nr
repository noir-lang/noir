mod foo;

use crate::foo::bat as bar2;
use std::hash;
use coo4::coo as pop;
use coo3::wow;

fn main(a : Witness, b : Witness, _out : Witness) {
       const one = 1;
       priv z = pop::my_function(b);
       priv z1 = wow::identity(z);
       priv x = bar2::hello(z1);
       let k = hash::sha256([a,b, x]); 
       priv _c : u32 = (k[0] + one) as u32;
}