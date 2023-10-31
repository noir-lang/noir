use crate::{
    abis::private_circuit_public_inputs::PrivateCircuitPublicInputs,
    tests::testing_harness::{PrivateCircuitPublicInputsBuilder, PrivateAppInputs},
    utils::uint128::U128,
    hash::NUM_FIELDS_PER_SHA256,
};

struct ConstructorParams {}

pub fn constructor_app(inputs: PrivateAppInputs, params: ConstructorParams) -> PrivateCircuitPublicInputs {
    PrivateCircuitPublicInputsBuilder::new(inputs).finish()
}