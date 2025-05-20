use crate::{
    address::{AztecAddress, PartialAddress, SaltedInitializationHash},
    contract_class_id::ContractClassId,
    hash::private_functions_root_from_siblings,
    public_keys::PublicKeys,
    tests::fixtures::contract_functions::get_protocol_contract_function,
};

pub struct ContractData {
    pub address: AztecAddress,
    pub artifact_hash: Field,
    pub contract_address_salt: Field,
    pub contract_class_id: ContractClassId,
    pub private_functions_root: Field,
    pub public_bytecode_commitment: Field,
    pub public_keys: PublicKeys,
    pub salted_initialization_hash: SaltedInitializationHash,
    pub partial_address: PartialAddress,
    pub deployer: AztecAddress,
}

// Copy-and-paste from __snapshots__/noir_test_gen.test.ts.snap (defaultContract 1)
pub global default_contract: ContractData = ContractData {
    contract_address_salt: 0x000000000000000000000000000000000000000000000000000000000000ddd5,
    artifact_hash: 0x0000000000000000000000000000000000000000000000000000000000003039,
    public_bytecode_commitment: 0x2e018d154e7faf1ddf8400a4b3605574e69afc04425f3e662e054dc094fb0428,
    private_functions_root: 0x2b18285717cd9bf8008715c0249d7934b5481da4688f0d3cba007b7e292d56d7,
    address: AztecAddress {
        inner: 0x1f790a6c3a1e3f3562b41f5ea05cfeb38dc5562c09520e75f7a127827129f918,
    },
    partial_address: PartialAddress {
        inner: 0x0bf3894bffb754bd07b13afb9e538bc800b6b3e404d4507b88ea72bb4edf41e7,
    },
    contract_class_id: ContractClassId {
        inner: 0x0d1476163c809d8c11cd7b66296eda5b653707d6d1a766845fd78bdb47de7a30,
    },
    public_keys: PublicKeys::default(),
    salted_initialization_hash: SaltedInitializationHash {
        inner: 0x13a939daa511233e5446905ed2cadbee14948fa75df183b53b5c14b612bffe88,
    },
    deployer: AztecAddress {
        inner: 0x0000000000000000000000000000000000000000000000000000000000000000,
    },
};

// Copy-and-paste from __snapshots__/noir_test_gen.test.ts.snap
pub global parent_contract: ContractData = ContractData {
    contract_address_salt: 0x0000000000000000000000000000000000000000000000000000000000001618,
    artifact_hash: 0x00000000000000000000000000000000000000000000000000000000000004bc,
    public_bytecode_commitment: 0x258ed5ce21c3a9c15a1c6108ef0e759bb0617f2b505b8d64e927027d30e71791,
    private_functions_root: 0x21ac15d1c4c74eb5ceb674a971451a6a314f00ee17571605d88555ebf6dc61c6,
    address: AztecAddress {
        inner: 0x0652e43af6645a9e5d9f370c30e0679e9933ca6db61e8e8892a2c8caf1742b22,
    },
    partial_address: PartialAddress {
        inner: 0x0af7fac4aad94b9f6142b45730616d34bf28d0b310376d2d144d0828df56e585,
    },
    contract_class_id: ContractClassId {
        inner: 0x03112baad91c865ba6d7d431703935f852934379a80932b8713ee5e5f17bb594,
    },
    public_keys: PublicKeys::default(),
    salted_initialization_hash: SaltedInitializationHash {
        inner: 0x24bd6ac7a182e2cf25e437c72f53544ef81dfd97d9afee23abb07a638e7be749,
    },
    deployer: AztecAddress {
        inner: 0x0000000000000000000000000000000000000000000000000000000000000000,
    },
};

pub fn get_protocol_contract(index: u32) -> ContractData {
    let seed = index as Field;
    let artifact_hash = 576576 + seed;
    let salted_initialization_hash = SaltedInitializationHash { inner: 281972 + seed };
    let public_bytecode_commitment = 38383 + seed;
    // Empty public keys here will throw an error when doing ec ops
    let public_keys = PublicKeys::default();

    let function = get_protocol_contract_function(index);
    let private_functions_root = private_functions_root_from_siblings(
        function.data.selector,
        function.vk_hash,
        function.membership_witness.leaf_index,
        function.membership_witness.sibling_path,
    );

    let contract_class_id = ContractClassId::compute(
        artifact_hash,
        private_functions_root,
        public_bytecode_commitment,
    );

    let partial_address = PartialAddress::compute_from_salted_initialization_hash(
        contract_class_id,
        salted_initialization_hash,
    );

    let address = AztecAddress::compute(public_keys, partial_address);

    ContractData {
        contract_address_salt: 1,
        artifact_hash,
        public_bytecode_commitment,
        private_functions_root,
        address,
        partial_address,
        contract_class_id,
        public_keys,
        salted_initialization_hash,
        deployer: AztecAddress { inner: 0 },
    }
}
