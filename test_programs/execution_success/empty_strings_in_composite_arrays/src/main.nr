unconstrained fn main() -> pub bool {
    func_2([(true, "", "")]) != func_2([(true, "", "")])
}
unconstrained fn func_2(b: [(bool, str<0>, str<0>); 1]) -> (str<0>, str<0>) {
    println(b);
    (b[0].1, b[0].2)
}
