use dep::parity_lib::{BaseParityInputs, ParityPublicInputs};

#[recursive]
fn main(inputs: BaseParityInputs) -> pub ParityPublicInputs {
    inputs.base_parity_circuit()
}
