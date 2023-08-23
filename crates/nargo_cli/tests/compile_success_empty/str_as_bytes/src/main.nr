use dep::std;
fn main() {
    let a = "hello";
    // let b = a.as_bytes();
    // assert(b[0]==104);
    // assert(b[1]==101);
    // assert(b[2]==108);
    // assert(b[3]==108);
    // assert(b[4]==111);
    // assert(b.len()==5);
    let mut c = a.as_bytes_vec();
    assert(c.get(0)==104);
    // assert(c.get(1)==101);
    // assert(c.get(2)==108);
    // assert(c.get(3)==108);
    // assert(c.get(4)==111);
    assert(c.len()==5);
}
