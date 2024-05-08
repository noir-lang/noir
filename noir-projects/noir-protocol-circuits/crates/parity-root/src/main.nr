use dep::parity_lib::{RootParityInputs, ParityPublicInputs};

#[recursive]
fn main(inputs: RootParityInputs) -> pub ParityPublicInputs {
    inputs.root_parity_circuit()
}
