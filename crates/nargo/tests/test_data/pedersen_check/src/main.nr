use dep::std;

fn main(x: Field, y: Field, out_x: Field, out_y: Field ) {
    let res = std::hash::pedersen([x, y]);
    constrain res[0] == out_x;
    constrain res[1] == out_y;
}
    
