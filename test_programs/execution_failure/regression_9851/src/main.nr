fn main() -> pub bool {
    let mut _b = {
        let g: [(bool, str<2>); 1] = { [(false, "KO")] };
        g[10].0
    };
    true
}
