use dep::std;
fn main(x: u1, y: u1) {
    constrain x | y == 1;

    constrain x | y | x == 1;
}
    
