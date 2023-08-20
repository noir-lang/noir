use dep::std;

fn main() {
    let z1 = 0;
    let z2 = 1;
    let cl_outer = |x| {
        let cl_inner = |y| {
            x + y + z2
        };
        cl_inner(1) + z1
    };
    let result = cl_outer(1);
    std::assert_eq(result, 3);
}
