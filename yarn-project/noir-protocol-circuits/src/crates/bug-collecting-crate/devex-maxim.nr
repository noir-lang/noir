Ability to template println and assert statements.

assert(foo == bar, "foo was equal to {}, but bar was equal to {}", foo, bar);

and:

fn foo(foo: Field, bar : Field, msg : string) {
    assert(foo == bar, msg)
}