fn main() {
    primitive_tests();
    foo_tests();
    option_tests();
    abc_tests();
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
}

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
    match tuple {
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
