use dep::std;

// Simple example of adding a public input in the middle of main
fn main(a : Field, pub_x : pub Field, pub_y : pub Field) {
     let res = std::scalar_mul::fixed_base(a);
     constrain res[0] == pub_x;
     constrain res[1] == pub_y;
}