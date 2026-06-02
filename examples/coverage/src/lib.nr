pub fn double(x: u32) -> u32 {
    x * 2
}

pub fn add(x: u32, y: u32) -> u32 {
    x + y
}

mod moo {
    // Not covered by any test; should appear as uncovered in the report.
    pub fn triple(x: u32) -> u32 {
        x * 3
    }
}

// Function declaration spanning multiple lines. We should see:
// * FN as well as DA for the function name (DA visibly marks it as covered/uncovered)
// * No DA for the opening `{` brace of the function body
// * Array items as well as `[` highlighted.
pub fn multi_line_function_decl<T>(
    param_1: T,
    param_2: T,
    param_3: T,
    param_4: T,
    param_5: T,
    param_6: T,
    param_7: T,
    param_8: T,
) -> [T; 10]
where
    T: Default,
{
    [
        T::default(),
        param_1,
        param_2,
        param_3,
        param_4,
        param_5,
        param_6,
        param_7,
        param_8,
        T::default(),
    ]
}

// Two impls of the same trait method live in this file. They should appear in the
// coverage report under distinct, fully-qualified names so the entries don't collide.
// `quadrupled` is a trait function with a default body and is reported under the trait.
pub trait Quadruple {
    fn doubled(self) -> Self;
    fn quadrupled(self) -> Self {
        self.doubled().doubled()
    }
}

impl Quadruple for u32 {
    fn doubled(self) -> u32 {
        self * 2
    }
}

impl Quadruple for u64 {
    fn doubled(self) -> u64 {
        self * 2
    }
}

// Two inherent impls of `new` on different structs. Their entries should be qualified by
// the receiver type so they don't collide on the bare name `new`.
pub struct First {
    value: u32,
}

impl First {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

pub struct Second {
    value: u32,
}

impl Second {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

trait Convert<T> {
    fn convert(self) -> T;
}

impl Convert<u8> for u8 {
    fn convert(self) -> u8 {
        0
    }
}

impl Convert<u16> for u8 {
    fn convert(self) -> u16 {
        0
    }
}

impl Convert<u8> for u16 {
    fn convert(self) -> u8 {
        0
    }
}

impl Convert<u16> for u16 {
    fn convert(self) -> u16 {
        0
    }
}

#[test]
fn test() {
    let _: u8 = 0_u8.convert();
    let _: u16 = 0_u8.convert();
    let _: u8 = 0_u16.convert();
    let _: u16 = 0_u16.convert();
}

#[test]
fn test_inherent_news() {
    let _ = First::new(1);
    let _ = Second::new(2);
}

#[test]
fn test_double() {
    assert_eq(double(3), 6);
}

#[test]
fn test_add() {
    assert_eq(add(2, 3), 5);
}

#[test]
fn test_quadrupled() {
    assert_eq(3u32.quadrupled(), 12);
    assert_eq(3u64.quadrupled(), 12);
}

#[test]
fn test_convert() {
    assert_eq(0_u8.convert(), 0_u8);
    assert_eq(0_u8.convert(), 0_u16);
    assert_eq(0_u16.convert(), 0_u8);
    assert_eq(0_u16.convert(), 0_u16);
}
