fn main(c: bool) -> pub bool {
    let mut e: &mut bool = (&mut false);
    if c { e = (&mut true); };
    *e
}
