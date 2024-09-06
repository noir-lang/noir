mod foo {
    #[attr]
    struct Foo {}

    fn attr(s: StructDefinition) {
        assert_eq(s.module().name(), quote { foo });
    }
}

fn main() {}

