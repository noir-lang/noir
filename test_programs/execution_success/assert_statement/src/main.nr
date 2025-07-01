// // Tests a very simple program.
// //
// // The features being tested is assertion
// fn main(x: Field, y: pub Field) {
//     assert(x == y, "x and y are not equal");
//     assert_eq(x, y, "x and y are not equal");
// }


pub fn main(x: Field) -> pub u8 {
    let x_bytes: [u8; 16] = x.to_le_bytes(); // this fails, unless I change the type to `[u8; 32]` (instead of `[u8; 16]`)
    println(x);
    println(x_bytes);
    x_bytes[0]
}

#[test]
fn test_main() {
    // let _x = main(0x05);
    let _x = main(0x0f84cda3598b8e2712c91f5c6a2b347f2289c66663137b9c2f02f05253c31ce1);
}