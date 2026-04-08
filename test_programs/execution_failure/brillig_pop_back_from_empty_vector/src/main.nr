// Regression test for https://github.com/noir-lang/noir/actions/runs/23630420013
// Brillig was reporting "Index out of bounds" instead of "Attempt to pop from an empty vector"
// when popping from an empty vector.
unconstrained fn main(a: pub bool) -> pub u16 {
    let v: [u16] = if a {
        @[669, 41348]
    } else {
        @[30223; 0]
    };
    let (_, val) = v.pop_back();
    val
}
