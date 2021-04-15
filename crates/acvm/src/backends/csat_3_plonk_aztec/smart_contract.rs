use acir::circuit::Circuit;
use aztec_backend::barretenberg_rs::composer::StandardComposer;
use noir_field::Bn254Scalar;

use crate::SmartContract;

use super::Plonk;

impl SmartContract<noir_field::Bn254Scalar> for Plonk {
    fn eth_contract_from_cs(&self, circuit: Circuit<Bn254Scalar>) -> String {
        let constraint_system = aztec_backend::serialise_circuit(&circuit);

        let mut composer = StandardComposer::new(constraint_system);

        composer.smart_contract()
    }
}
