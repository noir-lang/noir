use dep::std;

fn main(x: Field, y: Field) {
    let z = x == y;
    let t = z as u8;
    std::assert_eq(t, 1);
}
    
