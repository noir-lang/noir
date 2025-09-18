fn main() {
    let b = unsafe { func_3() };
    if b {
        let c = unsafe { func_4() }[1532016929_u32].3;
        if c {
            let d = unsafe { func_3() };
        }
    }
}

unconstrained fn func_3() -> bool {
    true
}
unconstrained fn func_4() -> [(str<3>, str<3>, bool, bool); 4] {
    [
        ("ABC", "CDE", false, true),
        ("FGH", "IJK", false, true),
        ("LMN", "OPQ", false, true),
        ("RST", "UVW", false, true),
    ]
}
