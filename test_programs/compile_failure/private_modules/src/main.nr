mod foo;

fn main(x: Field) {
    // `hello` is a private function so we should not be able to call this.
    assert_eq(foo::hello(x), 42);
}
