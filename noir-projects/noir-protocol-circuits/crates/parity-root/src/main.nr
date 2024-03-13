use dep::parity_lib::{RootParityInputs, ParityPublicInputs};

fn main(inputs: RootParityInputs) -> pub ParityPublicInputs {
    inputs.root_parity_circuit()
}
