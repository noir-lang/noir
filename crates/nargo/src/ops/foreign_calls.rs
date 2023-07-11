/// This enumeration represents the Brillig foreign calls that are natively supported by nargo.
/// After resolution of a foreign call, nargo will restart execution of the ACVM
pub enum ForeignCall {
    Println,
    PrintlnFormat,
    Sequence,
    ReverseSequence,
}

impl std::fmt::Display for ForeignCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl ForeignCall {
    pub fn name(&self) -> &'static str {
        match self {
            ForeignCall::Println => "println",
            ForeignCall::PrintlnFormat => "println_format",
            ForeignCall::Sequence => "get_number_sequence",
            ForeignCall::ReverseSequence => "get_reverse_number_sequence",
        }
    }

    pub fn lookup(op_name: &str) -> Option<ForeignCall> {
        match op_name {
            "println" => Some(ForeignCall::Println),
            "println_format" => Some(ForeignCall::PrintlnFormat),
            "get_number_sequence" => Some(ForeignCall::Sequence),
            "get_reverse_number_sequence" => Some(ForeignCall::ReverseSequence),
            _ => None,

        }
    }
}