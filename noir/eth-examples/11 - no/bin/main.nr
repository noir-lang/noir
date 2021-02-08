// This is the Pedersen hash.
// It is hooked up, but I think I may be serialising it to the 
// proving system incorrectly, as there is a completeness error.
//
// XXX: Manually build constraint system and check this. 

use dep::std;

fn main(x : Witness, b : Witness, _output: Witness, c : Witness) {

     priv _result = std::hash::pedersen([x,b]);

     constrain _result == _output;
     
     constrain c == 0;
}