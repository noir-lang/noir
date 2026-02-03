mod foo {
    struct Struct {
        pub field: Field,
    }

    pub comptime mut global bar: u32 = 1;
    pub comptime mut global baz: Struct = Struct { field: 1 };
}

fn main() {
    comptime {
        foo::bar = 2;
        assert_eq(foo::bar, 2);

        foo::baz.field = 3;
        assert_eq(foo::baz.field, 3);
    }
}
