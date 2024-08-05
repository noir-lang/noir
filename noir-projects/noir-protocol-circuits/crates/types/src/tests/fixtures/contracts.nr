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
  private_functions_root: 0x25d76df45434ec75a83321daf941cfc667ff3a9027942e17105da4f50d1d13f9,
  address: AztecAddress { inner: 0x0e66d7cd9692428c550b93c9ef5f49ca9f02c03e98cb3c922d8c773f78f79fed },
  partial_address: PartialAddress { inner: 0x0cf203c94c91bed28440b00ecd888d88cce1f86ddf2aa8d33acbb9b6fc06d382 },
  contract_class_id: ContractClassId { inner: 0x28e91aaf764bc6083e2796ff884079ad895d4b948d6ce8f37f01b29d0bc95a21 },
  public_keys_hash: PublicKeysHash { inner: 0x000000000000000000000000000000000000000000000000000000000000b26e },
  salted_initialization_hash: SaltedInitializationHash { inner: 0x13a939daa511233e5446905ed2cadbee14948fa75df183b53b5c14b612bffe88 },
  deployer: AztecAddress { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }
};

// taken from __snapshots__/noir_test_gen.test.ts.snap
global parent_contract = ContractData {
  contract_address_salt: 0x0000000000000000000000000000000000000000000000000000000000001618,
  artifact_hash: 0x00000000000000000000000000000000000000000000000000000000000004bc,
  public_bytecode_commitment: 0x0000000000000000000000000000000000000000000000000000000000000005,
  private_functions_root: 0x1228b39ba6702af03e595300e8484c6373f00790d0148cc3d4ff0fd1c778a83a,
  address: AztecAddress { inner: 0x24415b2e716d6c7099580ab8e383fd5b16dc9fb441aa308571d8e24a2257da24 },
  partial_address: PartialAddress { inner: 0x245df9f519d616473880260dd64b19a838081bb44dc17cd6ea5d870a63d2bf57 },
  contract_class_id: ContractClassId { inner: 0x00236b0dc6c537d5106543053c5b85c4cbe95b0474f8238b094bae63f1cbcfee },
  public_keys_hash: PublicKeysHash { inner: 0x00000000000000000000000000000000000000000000000000000000000011c1 },
  salted_initialization_hash: SaltedInitializationHash { inner: 0x24bd6ac7a182e2cf25e437c72f53544ef81dfd97d9afee23abb07a638e7be749 },
  deployer: AztecAddress { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }
};
