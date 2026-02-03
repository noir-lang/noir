fn main(c: bool) -> pub Field {
    let f = || &mut 1;
    let g = || &mut 2;
    let h = if c { f } else { g };
    let x = h();
    *x
}
