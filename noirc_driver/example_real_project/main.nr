mod foo;

use foo::bat as bar2;
use dep::std::hash;
use dep::coo4::coo as pop;
use dep::coo4::coo::my_function as pop2;
use dep::coo3::wow;

fn main(a : Witness, b : Witness, _out : Witness) {
       const one = 1;
       
       priv z = pop2(b);

       priv z1 = wow::identity(z);
       
       priv x = bar2::hello(z1);
       
       let k = hash::sha256([a,b,x]); 
       
       priv _c : u32 = (k[0] + one) as u32;
}