use dep::std;

fn main(xs : [u8; 3]) {
    let sorted = xs.sort();
    std::assert_eq(sorted[0], 1);
    std::assert_eq(sorted[1], 2);
    std::assert_eq(sorted[2], 3);
}
