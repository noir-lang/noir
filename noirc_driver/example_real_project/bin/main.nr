   
   use crate::foo::bat as bar2;
   use std::hash;

   fn main(a : Witness, b : Witness, _out : Witness) {
          const one = 1;
          priv x = bar2::hello(b);
          let k = hash::sha256([a,b, x]); 
          priv _c : u32 = (k[0] + one) as u32;

   }