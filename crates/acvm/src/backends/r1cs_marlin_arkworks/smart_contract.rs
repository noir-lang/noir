use acir::circuit::Circuit;
use noir_field::Bn254Scalar;

use crate::SmartContract;

use super::Marlin;

impl SmartContract<Bn254Scalar> for Marlin {
    fn eth_contract_from_cs(&self, _circuit: Circuit<Bn254Scalar>) -> String {
        unimplemented!("The marlin backend for arkworks does not implement an ETH contract")
    }
}
