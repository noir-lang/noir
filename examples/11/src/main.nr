// This is the Pedersen hash function.

use dep::std;

fn main(x : Field, b : Field, _output: Field) {

     priv _result = std::hash::pedersen([x,b]);

     constrain _result == _output;
}