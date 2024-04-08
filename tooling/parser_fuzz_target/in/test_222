use dep::std;

fn main(x: [u8; 2]) {
    let mut y = x.as_slice();
    let digest1 = std::hash::sha256_slice(y);
    let mut v = y;
    if x[0] != 0 {
        v = y.push_back(x[0]);
    }
    let digest2 = std::hash::sha256_slice(v);
    assert(digest1 != digest2);
}
