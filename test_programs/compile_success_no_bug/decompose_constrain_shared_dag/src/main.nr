// Each iteration builds two values which both read the previous pair, so the asserted value is
// the root of a DAG in which every value below the top is reachable along many distinct routes.
// Decomposing `assert(x)` by enumerating those routes rather than the values costs 2^N: at N=24
// this took over seven minutes and 11.5GB of memory to compile.
fn main(a: bool, b: bool) {
    let mut x = a & b;
    let mut y = b & a;
    for _ in 0..24 {
        let nx = x & y;
        let ny = y & x;
        x = nx;
        y = ny;
    }
    assert(x);
}
