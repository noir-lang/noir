use dep::sumLib::secretSum;

#[recursive]
fn main(a: pub u64, b: pub u64) -> pub u64 {
    secretSum(a, b)
}

#[test]
fn test_not_equal() {
    assert(main(1, 3) == 4);
}
