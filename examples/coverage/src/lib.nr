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
