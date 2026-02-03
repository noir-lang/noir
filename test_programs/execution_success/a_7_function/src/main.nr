//Tests for function calling
fn f1(mut x: Field) -> Field {
    x = x + 1;
    x = f2(x);
    x
}

fn f2(mut x: Field) -> Field {
    x += 2;
    x
}
// Simple example
fn test0(mut a: Field) {
    a = f2(a);
    assert(a == 3);
}
// Nested call
fn test1(mut a: Field) {
    a = f1(a);
    assert(a == 4);
}

fn test2(z: Field, t: u32) {
    let a = z + t as Field;
    assert(a == 64);
    let e = pow(z, t as Field);
    assert(e == 714924299);
}

fn pow(base: Field, exponent: Field) -> Field {
    let mut r = 1 as Field;
    let b: [u1; 32] = exponent.to_le_bits();
    for i in 1..33 {
        r = r * r;
        r = (b[32 - i] as Field) * (r * base) + (1 - b[32 - i] as Field) * r;
    }
    r
}

fn test3(x: [u8; 3]) -> [u8; 3] {
    let mut buffer = [0 as u8; 3];
    for i in 0..3 {
        buffer[i] = x[i];
    }
    assert(buffer == x);
    buffer
}

fn test_multiple(x: u32, y: u32) -> (u32, u32) {
    (y, x)
}

fn test_multiple2() -> my_struct {
    my_struct { a: 5 as u32, b: 7 as u32 }
}

fn test_multiple3(x: u32, y: u32) {
    assert(x == y);
}

struct my_struct {
    a: u32,
    b: u32,
}

struct my2 {
    aa: my_struct,
    bb: my_struct,
}

fn test_multiple4(s: my_struct) {
    assert(s.a == s.b + 2);
}

fn test_multiple5(a: (u32, u32)) {
    assert(a.0 == a.1 + 2);
}

fn test_multiple6(a: my2, b: my_struct, c: (my2, my_struct)) {
    test_multiple4(a.aa);
    test_multiple5((b.a, b.b));
    assert(c.0.aa.a == c.1.a);
}

fn foo<let N: u32>(a: [Field; N]) -> [Field; N] {
    a
}

fn bar() -> [Field; 1] {
    foo([0])
}

fn main(x: u32, y: u32, a: Field, arr1: [u32; 9], arr2: [u32; 9]) {
    let mut ss: my_struct = my_struct { b: x, a: x + 2 };
    test_multiple4(ss);
    test_multiple5((ss.a, ss.b));
    let my = my2 { aa: ss, bb: ss };
    ss.a = 61;
    test_multiple6(my, ss, (my, ss));

    let my_block = {
        let mut ab = f2(a);
        ab = ab + a;
        (x, ab)
    };
    assert(my_block.1 == 4);

    test0(a);
    test1(a);
    test2(x as Field, y);
    assert(bar()[0] == 0);

    let mut b = [0 as u8, 5 as u8, 2 as u8];
    let c = test3(b);
    assert(b == c);
    b[0] = 1 as u8;
    let cc = test3(b);
    assert(c != cc);
    let e = test_multiple(x, y);
    assert(e.1 == e.0 + 54 as u32);
    let d = test_multiple2();
    assert(d.b == d.a + 2 as u32);
    test_multiple3(y, y);
    //Regression test for issue #628:
    let result = first(arr_to_field(arr1), arr_to_field(arr2));
    assert(result[0] == arr1[0] as Field);
}
// Issue #628
fn arr_to_field(arr: [u32; 9]) -> [Field; 9] {
    let mut as_field: [Field; 9] = [0 as Field; 9];
    for i in 0..9 {
        as_field[i] = arr[i] as Field;
    }
    as_field
}

fn first(a: [Field; 9], _b: [Field; 9]) -> [Field; 9] {
    a
}
