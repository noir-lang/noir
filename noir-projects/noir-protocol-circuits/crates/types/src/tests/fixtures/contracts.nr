use crate::address::{AztecAddress, EthAddress, PublicKeysHash, SaltedInitializationHash, PartialAddress};
use crate::tests::fixtures;
use crate::contract_class_id::ContractClassId;

struct ContractData {
    address: AztecAddress,
    artifact_hash: Field,
    contract_address_salt: Field,
    contract_class_id: ContractClassId, 
    private_functions_root: Field,
    public_bytecode_commitment: Field,
    public_keys_hash: PublicKeysHash,
    salted_initialization_hash: SaltedInitializationHash,
    partial_address: PartialAddress,
    deployer: AztecAddress,
}

// taken from __snapshots__/noir_test_gen.test.ts.snap
global default_contract = ContractData {
  contract_address_salt: 0x000000000000000000000000000000000000000000000000000000000000ddd5,
  artifact_hash: 0x0000000000000000000000000000000000000000000000000000000000003039,
  public_bytecode_commitment: 0x0000000000000000000000000000000000000000000000000000000000000005,
  private_functions_root: 0x19a3cc0b714976fb35d58b684ba36e86f82bac8b87517904a2727e5113fb4cba,
  address: AztecAddress { inner: 0x24e2561e4216c843ff11bf77d4f8a68247e537980273ce54b09b505f7352f6bb },
  partial_address: PartialAddress { inner: 0x27ab8475fe4647b48ffc4df7a6cc42bf1125f000ff113d9a0d6a11b952626761 },
  contract_class_id: ContractClassId { inner: 0x29f4bda24f38507064a90f7505dc0381c9d83c97271c5c2e92a4261d300861bf },
  public_keys_hash: PublicKeysHash { inner: 0x000000000000000000000000000000000000000000000000000000000000b26e },
  salted_initialization_hash: SaltedInitializationHash { inner: 0x25765504545d2cdaaa6544eb24bc78a3e20384452f2525669f196a1a42f45906 },
  deployer: AztecAddress { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }
};

// taken from __snapshots__/noir_test_gen.test.ts.snap
global parent_contract = ContractData {
  contract_address_salt: 0x0000000000000000000000000000000000000000000000000000000000001618,
  artifact_hash: 0x00000000000000000000000000000000000000000000000000000000000004bc,
  public_bytecode_commitment: 0x0000000000000000000000000000000000000000000000000000000000000005,
  private_functions_root: 0x2c1c949cb226995de94b7b8b5aeaab440739f2dfeb06d358441f60932cf243a7,
  address: AztecAddress { inner: 0x24692d7dbb532557c7466e8782d1fe99077e4787570414bd1a5e8fa5300caad8 },
  partial_address: PartialAddress { inner: 0x127bbd73a3cf497fb2d85342571695d894985b449a9343eec55485e9cbc514f8 },
  contract_class_id: ContractClassId { inner: 0x037a09515a79a2b8ebe5139dae1ab7c433523ac1fd5631836890df2148df51c7 },
  public_keys_hash: PublicKeysHash { inner: 0x00000000000000000000000000000000000000000000000000000000000011c1 },
  salted_initialization_hash: SaltedInitializationHash { inner: 0x04643e65513869350552499ed3412df59540dffe3cd698203deee8900b53bcec },
  deployer: AztecAddress { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }
};
