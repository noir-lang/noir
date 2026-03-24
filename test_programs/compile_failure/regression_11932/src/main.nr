fn bar<A, A>(_x: A, _y: A) {}
fn foo<let N: u32, let N: u32>() {}

fn main() {
    foo::<1>();
    foo::<1, 2>();
    bar::<u32, u32>(0, 1);
}
