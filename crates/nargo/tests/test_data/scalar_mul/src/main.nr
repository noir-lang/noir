use dep::std;

fn main(
     a: Field, 
     a_pub_x: pub Field, 
     a_pub_y: pub Field,
     b: Field,
     b_pub_x: pub Field,
     b_pub_y: pub Field
) {
     let mut priv_key = a;
     let mut pub_x: Field = a_pub_x;
     let mut pub_y: Field = a_pub_y;
     if a != 1 { // Change `a` in Prover.toml to test input `b`
          priv_key = b;
          pub_x = b_pub_x;
          pub_y = b_pub_y;
     }
     let res = std::scalar_mul::fixed_base(priv_key);
     constrain res[0] == pub_x;
     constrain res[1] == pub_y;
}
