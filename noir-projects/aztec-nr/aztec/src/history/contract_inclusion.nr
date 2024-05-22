use dep::protocol_types::{header::Header, address::AztecAddress, hash::silo_nullifier, constants::DEPLOYER_CONTRACT_ADDRESS};

trait ProveContractDeployment {
    fn prove_contract_deployment(header: Header, contract_address: AztecAddress);
}

impl ProveContractDeployment for Header {
    fn prove_contract_deployment(self, contract_address: AztecAddress) {
        // Compute deployment nullifier
        let nullifier = silo_nullifier(
            AztecAddress::from_field(DEPLOYER_CONTRACT_ADDRESS),
            contract_address.to_field()
        );

        self.prove_nullifier_inclusion(nullifier);
    }
}

trait ProveContractNonDeployment {
    fn prove_contract_non_deployment(header: Header, contract_address: AztecAddress);
}

impl ProveContractNonDeployment for Header {
    fn prove_contract_non_deployment(self, contract_address: AztecAddress) {
        // Compute deployment nullifier
        let nullifier = silo_nullifier(
            AztecAddress::from_field(DEPLOYER_CONTRACT_ADDRESS),
            contract_address.to_field()
        );

        self.prove_nullifier_non_inclusion(nullifier);
    }
}

trait ProveContractInitialization {
    fn prove_contract_initialization(header: Header, contract_address: AztecAddress);
}

impl ProveContractInitialization for Header {
    fn prove_contract_initialization(self, contract_address: AztecAddress) {
        // Compute initialization nullifier
        let nullifier = silo_nullifier(contract_address, contract_address.to_field());

        self.prove_nullifier_inclusion(nullifier);
    }
}

trait ProveContractNonInitialization {
    fn prove_contract_non_initialization(header: Header, contract_address: AztecAddress);
}

impl ProveContractNonInitialization for Header {
    fn prove_contract_non_initialization(self, contract_address: AztecAddress) {
        // Compute initialization nullifier
        let nullifier = silo_nullifier(contract_address, contract_address.to_field());

        self.prove_nullifier_non_inclusion(nullifier);
    }
}
