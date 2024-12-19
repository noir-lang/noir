mod foo;
mod baz;

global M: u32 = 32;
global L: Field = 10; // Unused globals currently allowed
global N: u32 = 5;
global T_LEN: u32 = 2;

// Globals can reference other globals
global DERIVED: Field = M as Field + L;

struct Dummy {
    x: [Field; N],
    y: [Field; foo::MAGIC_NUMBER],
}

struct Test {
    v: Field,
}
global VALS: [Test; 1] = [Test { v: 100 }];
global NESTED: [[Test; 1]; 2] = [VALS, VALS];

unconstrained fn calculate_global_value() -> Field {
    42
}

// Regression test for https://github.com/noir-lang/noir/issues/4318
global CALCULATED_GLOBAL: Field = unsafe {
    //@safety: testing context
    calculate_global_value()
};

fn main(
    a: [Field; M + N - N],
    b: [Field; 30 + N / 2],
    c: pub [Field; foo::MAGIC_NUMBER],
    d: [Field; foo::bar::N],
) {
    let test_struct = Dummy { x: d, y: c };

    for i in 0..foo::MAGIC_NUMBER {
        assert(c[i] == foo::MAGIC_NUMBER as Field);
        assert(test_struct.y[i] == foo::MAGIC_NUMBER as Field);
        assert(test_struct.y[i] != NESTED[1][0].v);
    }

    assert(N as Field != M as Field);

    let expected: u32 = 42;
    assert(foo::TYPE_INFERRED == expected);

    let mut y = 5;
    let mut x = M;
    for i in 0..N * N {
        let M: u32 = 10;
        x = M;

        y = i;
    }
    assert(y == 24);
    assert(x == 10);

    let q = multiplyByM(3);
    assert(q == 96);

    arrays_neq(a, b);

    let t: [Field; T_LEN] = [N as Field, M as Field];
    assert(t[1] == 32);

    assert(15 == my_submodule::my_helper());

    let add_submodules_N = my_submodule::N as Field + foo::bar::N as Field;
    assert(15 == add_submodules_N);
    let add_from_bar_N = my_submodule::N as Field + foo::bar::from_bar(1);
    assert(15 == add_from_bar_N);
    // Example showing an array filled with (my_submodule::N + 2) 0's
    let sugared = [0; my_submodule::N + 2];
    assert(sugared[my_submodule::N + 1] == 0);

    let arr: [Field; my_submodule::N] = [N as Field; 10];
    assert((arr[0] == 5) & (arr[9] == 5));

    foo::from_foo(d);
    baz::from_baz(c);
    assert(DERIVED == M as Field + L);

    assert(CALCULATED_GLOBAL == 42);
}

fn multiplyByM(x: Field) -> Field {
    x * M as Field
}

fn arrays_neq(a: [Field; M], b: [Field; M]) {
    assert(a != b);
}

mod my_submodule {
    global N: u32 = 10;
    global L: Field = 50;

    fn my_bool_or(x: u1, y: u1) {
        assert(x | y == 1);
    }

    pub fn my_helper() -> Field {
        let N: u32 = 15; // Like in Rust, local variables override globals
        let x = N as Field;
        x
    }
}

struct Foo {
    a: Field,
}

struct Bar {}

impl Bar {
    fn get_a() -> Field {
        1
    }
}

// Regression for #1440
global foo: Foo = Foo { a: Bar::get_a() };
