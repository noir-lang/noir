use crate::address::{AztecAddress, EthAddress, PublicKeysHash, SaltedInitializationHash, PartialAddress};
use crate::tests::fixtures;
use crate::contract_class_id::ContractClassId;

struct ContractData {
    address: AztecAddress,
    artifact_hash: Field,
    contract_address_salt: Field,
    contract_class_id: ContractClassId, 
    portal_contract_address: EthAddress,
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
  public_bytecode_commitment: 0x129a3438653fe147133b2c274757920e37896305e7664c8c1eb380be3efd5fed,
  private_functions_root: 0x19a3cc0b714976fb35d58b684ba36e86f82bac8b87517904a2727e5113fb4cba,
  address: AztecAddress { inner: 0x2d941148ee5adeece35991d32acbcf4200742991c61990dee965bedf729d21a9 },
  partial_address: PartialAddress { inner: 0x23a6933a485200a8d34b9929d61868c9635793f878d67ce86a1b1355c0ab0d47 },
  portal_contract_address: EthAddress { inner: 0x0000000000000000000000000000000000005ba0 },
  contract_class_id: ContractClassId { inner: 0x0ce2a998337b1e6da1ac1d802a8bb9e10b7d705d210e61efb9642855009814a6 },
  public_keys_hash: PublicKeysHash { inner: 0x000000000000000000000000000000000000000000000000000000000000b26e },
  salted_initialization_hash: SaltedInitializationHash { inner: 0x0b095458845137ebf1e6061c8c0ba1d907241a3b56dc1d3e73d2fea78f04a036 },
  deployer: AztecAddress { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }
};

// taken from __snapshots__/noir_test_gen.test.ts.snap
global parent_contract = ContractData {
  contract_address_salt: 0x0000000000000000000000000000000000000000000000000000000000001618,
  artifact_hash: 0x00000000000000000000000000000000000000000000000000000000000004bc,
  public_bytecode_commitment: 0x1435ed970b275bebf95de3df53f23f3d2e97c9b54cf442bb03a3fa17a0ee3cd7,
  private_functions_root: 0x2c1c949cb226995de94b7b8b5aeaab440739f2dfeb06d358441f60932cf243a7,
  address: AztecAddress { inner: 0x24692d7dbb532557c7466e8782d1fe99077e4787570414bd1a5e8fa5300caad8 },
  partial_address: PartialAddress { inner: 0x127bbd73a3cf497fb2d85342571695d894985b449a9343eec55485e9cbc514f8 },
  portal_contract_address: EthAddress { inner: 0x0000000000000000000000000000000000000913 },
  contract_class_id: ContractClassId { inner: 0x1f1f963a350e2c883cc6730c19fc5d5b47a40694d805cbb0720fa76fe295df90 },
  public_keys_hash: PublicKeysHash { inner: 0x00000000000000000000000000000000000000000000000000000000000011c1 },
  salted_initialization_hash: SaltedInitializationHash { inner: 0x04643e65513869350552499ed3412df59540dffe3cd698203deee8900b53bcec },
  deployer: AztecAddress { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }
};
