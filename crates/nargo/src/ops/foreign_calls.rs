/// This enumeration represents the Brillig foreign calls that are natively supported by nargo.
/// After resolution of a foreign call, nargo will restart execution of the ACVM
pub enum ForeignCall {
    Println,
    PrintlnFormat,
    Sequence,
    ReverseSequence,
}