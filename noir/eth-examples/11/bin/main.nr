// This is the Pedersen hash function.

use dep::std;

fn main(x : Witness, b : Witness, _output: Witness) {

     priv _result = std::hash::pedersen([x,b]);

     constrain _result == _output;
}