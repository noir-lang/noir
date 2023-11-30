fn main() {
    let x: u32 = 0;
    x.trait_fn();
}

trait Trait {
    fn trait_fn<T>(x: T) -> T {}
}

impl Trait for u32 {
    fn trait_fn<A, B>(x: A) -> A { x }
}
