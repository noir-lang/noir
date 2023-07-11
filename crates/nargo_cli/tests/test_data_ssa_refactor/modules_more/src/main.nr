mod foo;

// An example of the module system
fn main(x: Field, y: Field) {
    assert(x != foo::bar::from_bar(y));
}
