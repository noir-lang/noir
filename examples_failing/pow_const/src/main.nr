use dep::std;

fn main(_x : Field) {
    constrain std::pow_const(2,3) == _x;
}
    
