struct Bar<T> {
    one: Field,
    two: Field,
    other: T,
}

fn foo<T>(bar: Bar<T>) {
    constrain bar.one == bar.two;
}

fn main(x : Field, y : Field) {
    let bar1: Bar<Field> = Bar { one: x, two: y, other: 0 };
    let bar2 = Bar { one: x, two: y, other: [0] };

    foo(bar1);
    foo(bar2);
}
