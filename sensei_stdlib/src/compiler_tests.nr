enum EnumTest {
    A,
    B(u256),
    C(u256, AlsoEnumTest)
}

enum AlsoEnumTest {
    Alpha(u256),
    Beta
}

#[test]
fn comptime_enum_test() {    
    comptime {
        let value = EnumTest::A;
        let result = match value {
            EnumTest::A => 1,
            EnumTest::B(_) => 2,
            EnumTest::C(_, _) => 3,
        };
        assert_eq(result, 1);

        let result = match value {
            EnumTest::B(_) => 2,
            _ => 3,
        };
        assert_eq(result, 3);

        let value = EnumTest::C(3, AlsoEnumTest::Alpha(10));
        let result = match value {
            EnumTest::A => 1,
            EnumTest::B(_) => 2,
            EnumTest::C(_, AlsoEnumTest::Beta) => 3,
            EnumTest::C(_, AlsoEnumTest::Alpha(x)) => x, 
        };
        assert_eq(result, 10);
    }
}