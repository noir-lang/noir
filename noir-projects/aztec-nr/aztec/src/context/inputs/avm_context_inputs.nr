use dep::protocol_types::traits::Empty;

struct AvmContextInputs {
    selector: Field,
    args_hash: Field,
    is_static_call: bool
}

impl Empty for AvmContextInputs {
    fn empty() -> Self {
        AvmContextInputs {
            selector: 0,
            args_hash: 0,
            is_static_call: false
        }
    }
}
