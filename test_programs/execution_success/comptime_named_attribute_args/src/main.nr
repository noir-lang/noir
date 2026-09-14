// Demonstrates `TypeDefinition::named_attribute_args` (https://github.com/noir-lang/noir/issues/13187):
// one attribute reads the *arguments* of a sibling attribute and splices them into generated code.
//
// Here `#[in_range(0, 100)]` records the bounds, and `#[validate]` reads those bounds and emits a
// `validate_*` checker from them (a hand-rolled `Validate`, no `#[derive]`).

#[validate]
#[in_range(0, 100)]
pub struct Percent {
    value: u32,
}

// Carries the bounds as attribute arguments; does nothing on its own.
comptime fn in_range(_s: TypeDefinition, _lo: u32, _hi: u32) {}

comptime fn validate(s: TypeDefinition) -> Quoted {
    // Read the bounds from the single `#[in_range(..)]` occurrence.
    let bounds = s.named_attribute_args("in_range")[0];
    let lo = bounds[0];
    let hi = bounds[1];

    let typ = s.as_type();
    quote {
        pub fn validate_percent(p: $typ) {
            assert(p.value >= $lo);
            assert(p.value <= $hi);
        }
    }
}

fn main(value: u32) {
    let p = Percent { value };
    // `validate_percent` was generated from the `#[in_range(0, 100)]` arguments.
    validate_percent(p);
}
