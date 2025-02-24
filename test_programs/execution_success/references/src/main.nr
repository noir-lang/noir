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
    let mut c = C { foo: 0, bar: &mut C2 { array: &mut [1, 2] } };
    *c.bar.array = [3, 4];
    assert(*c.bar.array == [3, 4]);
    regression_1887();
    regression_2054();
    regression_2030();
    regression_2255();
    regression_6443();
    assert(x == 3);
    regression_2218_if_inner_if(x, 10);
    regression_2218_if_inner_else(20, x);
    regression_2218_else(x, 3);
    regression_2218_loop(x, 10);
    regression_2560(s_ref);
}

fn add1(x: &mut Field) {
    *x += 1;
}

struct S {
    y: Field,
}

struct Nested {
    y: &mut &mut Field,
}

struct C {
    foo: Field,
    bar: &mut C2,
}

struct C2 {
    array: &mut [Field; 2],
}

impl S {
    fn add2(&mut self) {
        self.y += 2;
    }

    fn get_y(self) -> Field {
        self.y
    }
}

fn mutate_copy(mut a: Field) {
    a = 7;
}
// Previously the `foo.bar` in `foo.bar.mutate()` would insert an automatic dereference
// of `foo` which caused the method to wrongly be mutating a copy of bar rather than the original.
fn regression_1887() {
    let foo = &mut Foo { bar: Bar { x: 0 } };
    foo.bar.mutate();
    assert(foo.bar.x == 32);
}

struct Foo {
    bar: Bar,
}
struct Bar {
    x: Field,
}

impl Bar {
    fn mutate(&mut self) {
        self.x = 32;
    }
}
// Ensure that mutating a variable does not also mutate its copy
fn regression_2054() {
    let mut x = 2;
    let z = x;

    x += 1;
    assert(z == 2);
}
// The compiler was still trying to convert an LValue from an array of structs to struct of arrays indexing,
// even though this conversion was mostly removed elsewhere.
fn regression_2030() {
    let ref = &mut 0;
    let mut array = [ref, ref];
    let _ = *array[0];
    *array[0] = 1;
}

// The `mut x: &mut ...` caught a bug handling lvalues where a double-dereference would occur internally
// in one step rather than being tracked by two separate steps. This lead to assigning the 1 value to the
// incorrect outer `mut` reference rather than the correct `&mut` reference.
fn regression_2255() {
    let x = &mut 0;
    regression_2255_helper(x);
    assert(*x == 1);
}

fn regression_2255_helper(mut x: &mut Field) {
    *x = 1;
}

// Similar to `regression_2255` but without the double-dereferencing.
// The test checks that `mem2reg` does not eliminate storing to a reference passed as a parameter.
fn regression_6443() {
    let x = &mut 0;
    regression_6443_helper(x);
    assert(*x == 1);
}

fn regression_6443_helper(x: &mut Field) {
    *x = 1;
}

fn regression_2218(x: Field, y: Field) -> Field {
    let q = &mut &mut 0;
    let q1 = *q;
    let q2 = *q;

    if x != y {
        *q1 = 1;
        // Make sure that we correct load reference aliases through multiple blocks
        if x != 20 {
            *q1 = 10;
            *q2 = 2; // now we'd expect q1 == q2 == 2
            assert(*q1 == 2);
        } else {
            *q2 = 15;
            assert(*q1 == 15);
        }
    } else {
        *q2 = 20;
        assert(*q1 == 20);
    }
    // Have to assign value to return it
    let value = *q1;
    value
}

fn regression_2218_if_inner_if(x: Field, y: Field) {
    let value = regression_2218(x, y);
    assert(value == 2);
}

fn regression_2218_if_inner_else(x: Field, y: Field) {
    let value = regression_2218(x, y);
    assert(value == 15);
}

fn regression_2218_else(x: Field, y: Field) {
    let value = regression_2218(x, y);
    assert(value == 20);
}

fn regression_2218_loop(x: Field, y: Field) {
    let q = &mut &mut 0;
    let q1 = *q;
    let q2 = *q;

    for _ in 0..1 {
        if x != y {
            *q1 = 10;
            *q2 = 2; // now we'd expect q1 == q2 == 2
            assert(*q1 == 2);
        } else {
            *q2 = 20;
            assert(*q1 == 20);
        }
    }
    assert(*q1 == 2);

    for _ in 0..1 {
        for _ in 0..5 {
            if x != y {
                *q1 = 1;
                // Make sure that we correct load reference aliases through multiple blocks
                if x != 20 {
                    *q1 = 10;
                    *q2 = 2; // now we'd expect q1 == q2 == 2
                    assert(*q1 == 2);
                }
            } else {
                *q2 = 20;
                assert(*q1 == 20);
            }
        }
        if x != y {
            *q1 = 1;
            for _ in 0..5 {
                // Make sure that we correct load reference aliases through multiple blocks
                if x != 20 {
                    *q1 = 10;
                    *q2 = 2; // now we'd expect q1 == q2 == 2
                    assert(*q1 == 2);
                }
            }
        } else {
            *q2 = 20;
            assert(*q1 == 20);
        }
    }
    assert(*q1 == 2);

    if x != y {
        for _ in 0..5 {
            if x != y {
                *q1 = 1;
                // Make sure that we correct load reference aliases through multiple blocks
                if x != 20 {
                    *q1 = 10;
                    *q2 = 2; // now we'd expect q1 == q2 == 2
                    assert(*q1 == 2);
                }
            }
        }
    } else {
        *q2 = 20;
        assert(*q1 == 20);
    }
    assert(*q1 == 2);
}
// This is more a feature test than a proper regression.
// Before, we never automatically dereferenced objects in method calls to their value types.
// Now, we insert as many `*` as necessary to get to `S`.
fn regression_2560(s_ref: &mut S) {
    assert(s_ref.get_y() == 7);
}
