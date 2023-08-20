use dep::std;

fn main(a: u32, mut c: [u32; 4]){
    if a == c[0] {
        std::assert_eq(c[0], 0);
    } else if a == c[1] {
        std::assert_eq(c[1], 0);
    } else if a == c[2] {
        std::assert_eq(c[2], 0);
    } else if a == c[3] {
        // expect to match this case
        std::assert_eq(c[3], 0);
    } else {
       std::assert_eq(c[0], 10);
    }
}

