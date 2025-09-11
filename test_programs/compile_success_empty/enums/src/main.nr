fn main() {
    primitive_tests();
    foo_tests();
    option_tests();
    abc_tests();
    match_on_structs();
}

fn primitive_tests() {
    let x: i32 = -2;
    match x {
        -3 => fail(),
        -2 => (),
        0 => fail(),
        2 => fail(),
        _ => fail(),
    }

    match true {
        false => fail(),
        true => (),
    }

    // Ensure we can match on MIN and use globals as constants
    let i64_min = I64_MIN;
    match i64_min {
        9_223_372_036_854_775_807 => fail(),
        -9_223_372_036_854_775_807 => fail(),
        0 => fail(),
        I64_MIN => (),
        _ => fail(),
    }
}

global I64_MIN: i64 = -9_223_372_036_854_775_808;

enum Foo<T> {
    A(Field, Field),
    B(u32),
    C(T),
    D(),
    E,
}

fn foo_tests() {
    let _a = Foo::A::<u8>(1, 2);
    let _b: Foo<u16> = Foo::B(3);
    let _c = Foo::C(4);

    let _d: fn() -> Foo<(i32, i32)> = Foo::D;
    let _d: Foo<(i32, i32)> = Foo::D();
    let _e: Foo<u16> = Foo::E;
    let _e: Foo<u32> = Foo::E; // Ensure we can still use Foo::E polymorphically

    // Enum variants are functions and can be passed around as such
    let _many_cs = [1, 2, 3].map(Foo::C);

    match _b {
        Foo::C(_) => fail(),
        Foo::B(x) => { assert_eq(x, 3); },
        _ => fail(),
    }

    match _c {
        Foo::A(1, _) => fail(),
        Foo::E => fail(),
        Foo::C(4) => (),
        Foo::C(_) => fail(),
        _ => fail(),
    }
}

fn fail() {
    assert(false);
}

enum MyOption<T> {
    None,
    Maybe,
    Some(T),
}

fn option_tests() {
    let opt = MyOption::Some(ABC::C);
    match opt {
        MyOption::Some(ABC::D) => fail(),
        MyOption::Some(x) => { assert_eq(x, ABC::C); },
        _ => (),
    }
}

enum ABC {
    A,
    B,
    C,
    D,
    E,
    F,
}

impl Eq for ABC {
    fn eq(self, other: ABC) -> bool {
        match (self, other) {
            (ABC::A, ABC::A) => true,
            (ABC::B, ABC::B) => true,
            (ABC::C, ABC::C) => true,
            (ABC::D, ABC::D) => true,
            (ABC::E, ABC::E) => true,
            (ABC::F, ABC::F) => true,
            _ => false,
        }
    }
}

fn abc_tests() {
    // Mut is only to throw the optimizer off a bit so we can see
    // the `eq`s that get generated before they're removed because each of these are constant
    let mut tuple = (ABC::A, ABC::B);
    let _ = match tuple {
        (ABC::A, _) => 1,
        (_, ABC::A) => 2,
        (_, ABC::B) => 3,
        (_, ABC::C) => 4,
        (_, ABC::D) => 5,
        (ABC::B, ABC::E) => 6,
        (ABC::C, ABC::F) => 7,
        _ => 0,
    };
}

fn match_on_structs() {
    let foo = MyStruct { x: 10, y: 20 };
    match foo {
        MyStruct { x, y } => {
            assert_eq(x, 10);
            assert_eq(y, 20);
        },
    }

    match MyOption::Some(foo) {
        MyOption::Some(MyStruct { x: x2, y: y2 }) => {
            assert_eq(x2, 10);
            assert_eq(y2, 20);
        },
        MyOption::None => fail(),
        MyOption::Maybe => fail(),
    }
}

struct MyStruct {
    x: i32,
    y: Field,
}

pub fn match_on_u32_type_var(a: u32) {
    for i in 0..5 {
        let _ = match i {
            1 => "one",
            2 => "two",
            _ => "???",
        };
    }
}
