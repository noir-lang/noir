global G_A: (str<3>, [str<3>; 2]) = ("AWQ", ["LNV", "ZIH"]);
unconstrained fn main(a: [str<3>; 2], b: bool) -> pub [[str<3>; 2]; 1] {
    [if (!b) { a } else { G_A.1 }]
}
