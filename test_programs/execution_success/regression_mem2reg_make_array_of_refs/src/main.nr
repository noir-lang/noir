fn main() -> pub bool {
    foo()
}
fn foo() -> bool {
    bar([(&mut true)])
}
fn bar(a: [&mut bool; 1]) -> bool {
    let c: [&mut bool; 2] = [a[0], (&mut true)];
    let idx: u32 = if (*c[0]) { 1 } else { 0 };
    *c[idx]
}
