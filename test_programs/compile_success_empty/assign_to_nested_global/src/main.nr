mod foo {
    pub comptime mut global bar: u32 = 1;
}

fn main() {
    comptime {
        foo::bar = 2;
        assert_eq(foo::bar, 2);
    }

    comptime {
        foo::bar = 3;
        assert_eq(foo::bar, 3);
    }
}
