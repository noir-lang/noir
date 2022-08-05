use acvm::acir::circuit::Circuit;

use acvm::SmartContract;

use super::Marlin;

impl SmartContract for Marlin {
    fn eth_contract_from_cs(&self, _circuit: Circuit) -> String {
        unimplemented!("The marlin backend for arkworks does not implement an ETH contract")
    }
}
