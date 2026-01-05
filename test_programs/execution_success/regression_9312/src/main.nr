fn main(a: bool) -> pub ((u32, Field), (u32, Field)) {
    let s0: [Field] = if a { #[10, 20] } else { #[30, 40, 50] };
    let (s1, f) = s0.pop_back();
    let s2 = s0.push_back(60);
    ((s1.len(), f), (s2.len(), s2[2]))
}
