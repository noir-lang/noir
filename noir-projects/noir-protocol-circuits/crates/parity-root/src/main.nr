use dep::parity_lib::{RootParityInputs, ParityPublicInputs};

#[recursive]
fn main(inputs: RootParityInputs) -> distinct pub ParityPublicInputs {
    inputs.root_parity_circuit()
}
