use crate::baz::in_baz_mod;
use crate::foo::bar::in_bar_mod;
use crate::foo::in_foo_mod;

mod foo;
mod baz;

fn main() {
    assert(in_foo_mod() != in_bar_mod());
    assert(in_baz_mod());
}
