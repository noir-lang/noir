global G_C: [[str<1>; 4]; 4] =
    [["a", "a", "a", "a"], ["a", "a", "a", "a"], ["a", "a", "a", "a"], ["a", "a", "a", "a"]];
unconstrained fn main(a: [[str<1>; 4]; 4]) {
    let mut f = a;
    f[0] = G_C[3];
}
