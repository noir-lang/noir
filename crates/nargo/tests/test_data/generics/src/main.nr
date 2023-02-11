struct Bar<T> {
    one: Field,
    two: Field,
    other: T,
}

fn foo<T>(bar: Bar<T>) {
    constrain bar.one == bar.two;
}

struct BigInt<N> {
    limbs: [u32; N],
}

impl<N> BigInt<N> {
    // `N` is in scope of all methods in the impl
    fn first(first: BigInt<N>, second: BigInt<N>) -> Self {
        constrain first.limbs != second.limbs;
        first
    }

    fn second(first: BigInt<N>, second: Self) -> Self {
        constrain first.limbs != second.limbs;
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
    constrain limbs == int2.limbs;

    // Test impl exclusively for Bar<Field>
    constrain bar1.get_other() == bar1.other;

    // Expected type error
    // constrain bar2.get_other() == bar2.other;
}
