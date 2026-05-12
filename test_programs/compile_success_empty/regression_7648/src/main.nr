// https://github.com/noir-lang/noir/issues/7648
trait Foo<let N: u32> {}

impl<let N: u32> Foo<N> for [Field; N] {}

fn my_fn<let N: u32>(_input: impl Foo<N>) {}

fn main() {
    my_fn::<0>([]);
}
