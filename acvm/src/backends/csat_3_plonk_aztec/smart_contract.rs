use acir::circuit::Circuit;
use aztec_backend::barretenberg_rs::composer::StandardComposer;

use crate::SmartContract;

use super::Plonk;

impl SmartContract for Plonk {
    fn eth_contract_from_cs(&self, circuit: Circuit) -> String {
        let constraint_system = aztec_backend::serialise_circuit(&circuit);

        let mut composer = StandardComposer::new(constraint_system.size());

        composer.smart_contract(&constraint_system)
    }
}
