use dep::std;
fn main() {
    let a: str<2> = "ab";
    let b = a.as_bytes();
    assert(b[0]==97);
    assert(b[1]==98);
}
