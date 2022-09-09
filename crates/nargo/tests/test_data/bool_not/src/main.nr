use dep::std;
fn main(x: Field) {
    constrain std::bool_not(x) == 0;
}
    
