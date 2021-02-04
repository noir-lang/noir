use dep::std;

fn main(x : Witness, b : Witness, _output: Witness, c : Witness) {

     priv _result = std::hash::pedersen([x,b]);

     constrain _result == _output;
     
     constrain c == 0;
}