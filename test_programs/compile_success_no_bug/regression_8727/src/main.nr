global G_A: (str<2>, [str<2>; 2], [bool; 0]) = ("IU", ["TC", "JC"], []);
unconstrained fn main() -> pub str<2> {
    (*func_2(G_A)[0])
}
unconstrained fn func_2(mut b: (str<2>, [str<2>; 2], [bool; 0])) -> [&mut str<2>; 1] {
    let _c: [str<2>; 1] = [b.0];
    [(&mut b.0)]
}
