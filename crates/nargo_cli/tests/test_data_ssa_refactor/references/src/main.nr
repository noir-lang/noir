fn main(mut x: Field) {
    add1(&mut x);
    assert(x == 3);

    let mut s = S { y: x };
    s.add2();
    assert(s.y == 5);

    // Regression for #1946: Method resolution error when calling &mut methods with a variable of type &mut T
    let s_ref = &mut s;
    s_ref.add2();
    assert(s.y == 7);

    // Test that normal mutable variables are still copied
    let mut a = 0;
    mutate_copy(a);
    assert(a == 0);

    // Test something 3 allocations deep
    let mut nested_allocations = Nested { y: &mut &mut 0 };
    add1(*nested_allocations.y);
    assert(**nested_allocations.y == 1);

    // Test nested struct allocations with a mutable reference to an array.
    let mut c = C {
        foo: 0,
        bar: &mut C2 {
            array: &mut [1, 2],
        },
    };
    *c.bar.array = [3, 4];
    assert(*c.bar.array == [3, 4]);

    regression_2054();
}

// Ensure that mutating a variable does not also mutate its copy
fn regression_2054() {
    let mut x = 2;
    let z = x;

    x += 1;
    assert(z == 2);
}

fn add1(x: &mut Field) {
    *x += 1;
}

struct S { y: Field }

struct Nested { y: &mut &mut Field }

struct C {
    foo: Field,
    bar: &mut C2,
}

struct C2 {
    array: &mut [Field; 2]
}

impl S {
    fn add2(&mut self) {
        self.y += 2;
    }
}

fn mutate_copy(mut a: Field) {
    a = 7;
}
