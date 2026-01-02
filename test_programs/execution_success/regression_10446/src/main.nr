fn main(a: bool) -> pub bool {
    if !a {
        let mut c: [str<2>] = &["IR"];
        for _ in 0..2 {
            c = if a {
                let _ = c.pop_back();
                &["QM", "HF"]
            } else {
                &["VD", "YH", "SE", "FB"]
            };
        }
        false
    } else {
        true
    }
}
