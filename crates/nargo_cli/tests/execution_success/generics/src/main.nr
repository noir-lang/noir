use dep::std;

struct Bar<T> {
    one: Field,
    two: Field,
    other: T,
}

fn foo<T>(bar: Bar<T>) {
    std::assert_eq(bar.one, bar.two);
}

struct BigInt<N> {
    limbs: [u32; N],
}

impl<N> BigInt<N> {
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

fn main(x: Field, y: Field) {
    let bar1: Bar<Field> = Bar { one: x, two: y, other: 0 };
    let bar2 = Bar { one: x, two: y, other: [0] };

    foo(bar1);
    foo(bar2);

    // Test generic impls
    let int1 = BigInt { limbs: [1] };
    let int2 = BigInt { limbs: [2] };
    let BigInt { limbs } = int1.second(int2).first(int1);
    std::assert_eq(limbs, int2.limbs);

    // Test impl exclusively for Bar<Field>
    std::assert_eq(bar1.get_other(), bar1.other);

    // Expected type error
    // std::assert_eq(bar2.get_other(), bar2.other);

    let one = x;
    let two = y;
    let nested_generics: Bar<Bar<Field>> = Bar { one, two, other: Bar { one, two, other: 0 } };
    std::assert_eq(nested_generics.other.other, bar1.get_other());

    let _ = regression_2055([1, 2, 3]);
}

fn regression_2055<LEN>(bytes: [u8; LEN]) -> Field {
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
