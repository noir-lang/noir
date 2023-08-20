use dep::std;

fn main(x: u1, y: u1) {
    std::assert_eq(x | y, 1);

    std::assert_eq(x | y | x, 1);
}
    
