// A macro library that keeps state across the items of one compilation, the way a
// note or event registry does.
pub comptime mut global REGISTERED: u32 = 0;

pub comptime fn register(_s: TypeDefinition) -> Quoted {
    REGISTERED += 1;
    quote {}
}

pub comptime fn expect_registered(_s: TypeDefinition, expected: u32) -> Quoted {
    let registered = REGISTERED;
    assert_eq(registered, expected, f"expected {expected} registered types but found {registered}");
    quote {}
}
