fn main(b: [i8; 2]) -> pub i8 {
    let i = foo(b);
    let c = comptime { foo([10, 20]) };
    assert_eq(i, c);
    i
}

fn foo(mut b: [i8; 2]) -> i8 {
    b[{
        let d = 1;
        for _ in 0..2 {
            b[0] = b[1] + d;
        }
        0
    }]
}
