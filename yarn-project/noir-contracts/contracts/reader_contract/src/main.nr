contract Reader {
    use dep::aztec::protocol_types::{
        address::AztecAddress,
        abis::function_selector::FunctionSelector,
    };

    use dep::compressed_string::{FieldCompressedString, FieldCompressedStringSerializationMethods};

    #[aztec(private)]

    fn constructor() {}

    #[aztec(public)]
    fn check_name(who: AztecAddress, what: str<31>) {
        let selector = FunctionSelector::from_signature("public_get_name()");
        let ret = context.call_public_function_no_args(who, selector);
        let name = FieldCompressedString::from_field(ret[0]);
        let _what = FieldCompressedString::from_string(what);
        assert(name.is_eq(_what));
    }

    #[aztec(public)]
    fn check_symbol(who: AztecAddress, what: str<31>) {
        let selector = FunctionSelector::from_signature("public_get_symbol()");
        let ret = context.call_public_function_no_args(who, selector);
        let symbol = FieldCompressedString::from_field(ret[0]);
        let _what = FieldCompressedString::from_string(what);
        assert(symbol.is_eq(_what));
    }

    #[aztec(public)]
    fn check_decimals(who: AztecAddress, what: u8) {
        let selector = FunctionSelector::from_signature("public_get_decimals()");
        let ret = context.call_public_function_no_args(who, selector);
        assert(ret[0] as u8 == what);
    }
}
