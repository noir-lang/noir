fn main() {
    let cl_outer = |x| {
        let cl_inner = |y| {
            x + y
        };
        cl_inner(1)
    };
    let result = cl_outer(1);
    assert(result == 2);
}
