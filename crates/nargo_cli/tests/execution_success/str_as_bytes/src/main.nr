use dep::std;
fn main() {
    let a = "hello";
    let b = a.as_bytes();
    assert(b[0]==104);
    assert(b[1]==101);
    assert(b[2]==108);
    assert(b[3]==108);
    assert(b[4]==111);
}
