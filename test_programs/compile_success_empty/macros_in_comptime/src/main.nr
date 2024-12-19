use std::field::modulus_num_bits;
use std::meta::unquote;

// Numeric generics default to u32
global three_field: Field = 3;

fn main() {
    comptime {
        unsafe {
            //@safety: testing context
            foo::<three_field>(5)
        };
        submodule::bar();
    }
}

// Call a different function from the interpreter, then have the
// elaborator switch to the middle of foo from its previous scope in main
comptime fn foo<let N: Field>(x: Field) {
    assert(modulus_num_bits() != 0);

    let cond = quote { modulus_num_bits() != 0 };
    assert(unquote!(cond));

    // Use a comptime parameter in scope
    assert_eq(5, x);
    assert_eq(5, unquote!(quote { x }));

    // Use a generic in scope
    assert_eq(3, N);
    assert_eq(3, unquote!(quote { N }));

    // Use `break` which only unconstrained functions can do.
    // This ensures the elaborator knows we're switching from `main` to `foo`
    for _ in 0..0 {
        break;
    }

    let loop = quote { for _ in 0..0 { break; } };
    unquote!(loop);
}

mod submodule {
    use std::field::modulus_be_bytes;
    use std::meta::unquote;

    pub comptime fn bar() {
        // Use a function only in scope in this module
        assert(modulus_be_bytes().len() != 0);

        let cond = quote { modulus_be_bytes().len() != 0 };
        assert(unquote!(cond));
    }
}
