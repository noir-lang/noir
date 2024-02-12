contract Reader {
    use dep::aztec::protocol_types::{address::AztecAddress, abis::function_selector::FunctionSelector};

    use dep::compressed_string::FieldCompressedString;

    #[aztec(private)]
    fn constructor() {}

    #[aztec(public)]
    fn check_name_public(who: AztecAddress, what: str<31>) {
        let selector = FunctionSelector::from_signature("public_get_name()");
        let ret = context.call_public_function_no_args(who, selector);
        let name = FieldCompressedString::from_field(ret[0]);
        let _what = FieldCompressedString::from_string(what);
        assert(name.is_eq(_what));
    }

    #[aztec(private)]
    fn check_name_private(who: AztecAddress, what: str<31>) {
        let selector = FunctionSelector::from_signature("private_get_name()");
        let ret = context.call_private_function_no_args(who, selector);
        let name = FieldCompressedString::from_field(ret[0]);
        let _what = FieldCompressedString::from_string(what);
        assert(name.is_eq(_what));
    }

    unconstrained fn get_name(who: AztecAddress) -> pub str<6> {
        // We cannot yet call an unconstrained function from another
        "Reader"
    }

    #[aztec(public)]
    fn check_symbol_public(who: AztecAddress, what: str<31>) {
        let selector = FunctionSelector::from_signature("public_get_symbol()");
        let ret = context.call_public_function_no_args(who, selector);
        let symbol = FieldCompressedString::from_field(ret[0]);
        let _what = FieldCompressedString::from_string(what);
        assert(symbol.is_eq(_what));
    }

    #[aztec(private)]
    fn check_symbol_private(who: AztecAddress, what: str<31>) {
        let selector = FunctionSelector::from_signature("private_get_symbol()");
        let ret = context.call_private_function_no_args(who, selector);
        let symbol = FieldCompressedString::from_field(ret[0]);
        let _what = FieldCompressedString::from_string(what);
        assert(symbol.is_eq(_what));
    }

    unconstrained fn get_symbol(who: AztecAddress) -> pub str<3> {
        // We cannot yet call an unconstrained function from another
        "RDR"
    }

    #[aztec(public)]
    fn check_decimals_public(who: AztecAddress, what: u8) {
        let selector = FunctionSelector::from_signature("public_get_decimals()");
        let ret = context.call_public_function_no_args(who, selector);
        assert(ret[0] as u8 == what);
    }

    #[aztec(private)]
    fn check_decimals_private(who: AztecAddress, what: u8) {
        let selector = FunctionSelector::from_signature("private_get_decimals()");
        let ret = context.call_private_function_no_args(who, selector);
        assert(ret[0] as u8 == what);
    }

    unconstrained fn get_decimals(who: AztecAddress) -> pub u8 {
        // We cannot yet call an unconstrained function from another
        18
    }
}
