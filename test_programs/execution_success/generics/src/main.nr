struct Bar<T> {
    one: Field,
    two: Field,
    other: T,
}

fn foo<T>(bar: Bar<T>) {
    assert(bar.one == bar.two);
}

struct BigInt<let N: u32> {
    limbs: [u32; N],
}

impl<let N: u32> BigInt<N> {
    // `N` is in scope of all methods in the impl
    fn first(first: BigInt<N>, second: BigInt<N>) -> Self {
        assert(first.limbs != second.limbs);
        first
    }

    fn second(first: BigInt<N>, second: Self) -> Self {
        assert(first.limbs != second.limbs);
        second
    }
}

impl Bar<Field> {
    fn get_other(self) -> Field {
        self.other
    }
}

impl<T> Bar<T> {
    // This is to test that we can use turbofish on methods as well
    fn zeroed<A>(_self: Self) -> A {
        std::mem::zeroed()
    }
}

fn main(x: Field, y: Field) {
    let bar1: Bar<Field> = Bar { one: x, two: y, other: 0 };
    let bar2 = Bar { one: x, two: y, other: [0] };

    foo(bar1);
    foo(bar2);
    // Test generic impls
    let int1 = BigInt { limbs: [1] };
    let int2 = BigInt { limbs: [2] };
    let BigInt { limbs } = int1.second(int2).first(int1);
    assert(limbs == int2.limbs);
    // Test impl exclusively for Bar<Field>
    assert(bar1.get_other() == bar1.other);
    // Expected type error
    // assert(bar2.get_other() == bar2.other);
    let one = x;
    let two = y;
    let nested_generics: Bar<Bar<Field>> = Bar { one, two, other: Bar { one, two, other: 0 } };
    assert(nested_generics.other.other == bar1.get_other());

    // Test turbofish operator
    foo::<Field>(bar1);

    // Test that turbofish works on methods and that it uses the generics on the methods
    // While still handling the generic on the impl (T in this case) that is implicitly added
    // to the method.
    assert(bar1.zeroed::<u32>() == 0);

    let _ = regression_2055([1, 2, 3]);
}

fn regression_2055<let LEN: u32>(bytes: [u8; LEN]) -> Field {
    let mut f = 0;
    let mut b = 1;
    let mut len = LEN - 1; // FAILS
    for i in 0..LEN {
        let j = len - i;
        f += (bytes[j] as Field) * b;
        b *= 256;
    }
    f
}
