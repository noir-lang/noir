// https://github.com/noir-lang/noir/issues/9852
fn main() -> pub (bool, bool) {
    let b: [(bool, bool); 2] = [(false, false), (false, true)];
    let c = if b[4098222575_u32].1 {
        b
    } else {
        [(false, false), (true, true)]
    };
    c[0]
}