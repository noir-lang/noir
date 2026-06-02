fn main(a: bool) -> pub bool {
    if !a {
        let mut c: [str<2>] = ["IR"].as_vector();
        for _ in 0..2 {
            c = if a {
                let _ = c.pop_back();
                ["QM", "HF"].as_vector()
            } else {
                ["VD", "YH", "SE", "FB"].as_vector()
            };
        }
        false
    } else {
        true
    }
}
