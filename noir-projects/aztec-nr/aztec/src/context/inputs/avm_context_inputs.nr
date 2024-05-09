use dep::protocol_types::traits::Empty;

struct AvmContextInputs {
    selector: Field,
    args_hash: Field,
}

impl Empty for AvmContextInputs {
    fn empty() -> Self {
        AvmContextInputs {
            selector: 0,
            args_hash: 0,
        }
    }
}
