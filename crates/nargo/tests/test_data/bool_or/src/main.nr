use dep::std;
fn main(x: Field, y: Field) {
    constrain std::bool_or(x, y) == 1;
}
    
