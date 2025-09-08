fn main(x: u128) {
    let const: u128 = 340282366920938463463374607431768211455;
    for y in 0..127 {
        let dynamic = x << y;
        let static = const << y;
        assert_eq(dynamic, static, f"shift: {y}, static: {static}, dynamic: {dynamic}");

        let dynamic = x >> y;
        let static = const >> y;
        assert_eq(dynamic, static, f"shift: {y}, static: {static}, dynamic: {dynamic}");
    }

    comptime {
        assert_eq(
            340282366920938463463374607431768211455_u128 << 126,
            255211775190703847597530955573826158592,
        );
        assert_eq(
            340282366920938463463374607431768211455_u128 << 127,
            170141183460469231731687303715884105728,
        );
    }
}
