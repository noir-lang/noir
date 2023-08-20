use dep::std;

fn main() {
    let unsorted: [u8; 3] = [3,1,2];
    let sorted = unsorted.sort();
    std::assert_eq(sorted[0], 1);
    std::assert_eq(sorted[1], 2);
    std::assert_eq(sorted[2], 3);
}
