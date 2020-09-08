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
pub enum Gate {
    Arithmetic(Arithmetic),
    Range(Witness, u32),
    And(AndGate),
    Xor(XorGate),
}
