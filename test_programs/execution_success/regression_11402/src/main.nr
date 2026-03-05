fn main(mut a: [(bool, str<2>); 2], b: str<1>, active: bool, index: u32) -> pub str<2> {
    a[index].1 = "cc";
    if active {
        // safety: testing
        let c = unsafe { foo(bar(), (0, b, a[0].1)) };
        assert(c == 0);
    };
    a[0].1
}
unconstrained fn bar() -> str<1> {
    "d"
}
unconstrained fn foo(_a: str<1>, b: (u32, str<1>, str<2>)) -> u32 {
    let (_, _, _) = b;
    b.0
}
