global G_C: [[str<0>; 4]; 4] = [["", "", "", ""], ["", "", "", ""], ["", "", "", ""], ["", "", "", ""]];
unconstrained fn main(a: [[str<0>; 4]; 4]) {
    let mut f = a;
    f[0] = G_C[3];
}