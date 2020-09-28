use low_level_std_lib::LowLevelStandardLibrary;
use crate::polynomial::Arithmetic;
use crate::Witness;

#[derive(Clone, Debug)]
pub struct AndGate {
    pub a: Witness,
    pub b: Witness,
    pub result: Witness,
    pub num_bits: u32,
}
#[derive(Clone, Debug)]
pub struct XorGate {
    pub a: Witness,
    pub b: Witness,
    pub result: Witness,
    pub num_bits: u32,
}

#[derive(Clone, Debug)]
// XXX: Gate does not capture what this is anymore. I think IR would be a better name
pub enum Gate {
    Arithmetic(Arithmetic),
    Range(Witness, u32),
    And(AndGate),
    Xor(XorGate),
    GadgetCall(GadgetCall),
    GadgetDefinition(GadgetDefinition), // XXX: Maybe we can have this in another place? Maybe put it along with the Circuit Definition
}

// Descriptor as to whether the input/output is fixed or variable
// Example: The input for Sha256 is Variable and the output is fixed at 2 witnesses, each holding 128 bits of the actual sha256 function
#[derive(Clone, Debug)]
pub enum FanSize {
    Variable,
    Fixed(u128),
}

// Note: Some gadgets will not use all of the witness
// So we need to supply how many bits of the witness is needed
#[derive(Clone, Debug)]
pub struct GadgetInput {
    pub witness: Witness,
    pub num_bits: u32,
}

#[derive(Clone, Debug)]
pub struct GadgetCall {
    pub name: LowLevelStandardLibrary,
    pub inputs: Vec<GadgetInput>,
    pub outputs: Vec<Witness>,
}

#[derive(Clone, Debug)]
// Specs for how many inputs/outputs the method takes.
// XXX: Is this needed?
pub struct GadgetDefinition {
    pub name: String,
    pub inputs: FanSize,
    pub outputs: FanSize,
}
