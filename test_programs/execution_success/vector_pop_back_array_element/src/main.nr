// Regression test: pop_back on a vector with array element types.
// In ACIR, vectors store data flat, so pop_back must reconstruct the
// composite element from individual scalar values.
global G: [[u8; 2]] = @[[76, 78]];

fn main(mut b: (bool, [u8; 2])) -> pub [u8; 2] {
    b = {
        let i = G.pop_back();
        (b.0, i.1)
    };
    b.1
}
