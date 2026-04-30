pub fn double(x: u32) -> u32 {
    x * 2
}

pub fn add(x: u32, y: u32) -> u32 {
    x + y
}

// Not covered by any test; should appear as uncovered in the report.
pub fn triple(x: u32) -> u32 {
    x * 3
}

#[test]
fn test_double() {
    assert_eq(double(3), 6);
}

#[test]
fn test_add() {
    assert_eq(add(2, 3), 5);
}
