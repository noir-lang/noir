// A numeric type alias's parameter used as a value must keep its numeric type through
// `nargo expand` (which prints it as a constant): if the constant loses its `u8` type,
// inference picks `impl Which for Field` instead and `which` returns the wrong value.
trait Which {
    fn which(self) -> Field;
}

impl Which for u8 {
    fn which(self) -> Field {
        1
    }
}

impl Which for Field {
    fn which(self) -> Field {
        2
    }
}

type AliasN<let N: u8>: u8 = N;

fn takes<T>(value: T) -> T
where
    T: Which,
{
    value
}

fn main(x: Field) {
    assert(takes(AliasN::<200u8>).which() == 1);
    assert(x == 5);
}
