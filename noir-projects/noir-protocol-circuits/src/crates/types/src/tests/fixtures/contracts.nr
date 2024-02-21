use crate::abis::membership_witness::ContractLeafMembershipWitness;
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
}

// taken from __snapshots__/noir_test_gen.test.ts.snap
global default_contract = ContractData {
  contract_address_salt: 0x000000000000000000000000000000000000000000000000000000000000ddd5,
  artifact_hash: 0x0000000000000000000000000000000000000000000000000000000000003039,
  public_bytecode_commitment: 0x129a3438653fe147133b2c274757920e37896305e7664c8c1eb380be3efd5fed,
  private_functions_root: 0x19a3cc0b714976fb35d58b684ba36e86f82bac8b87517904a2727e5113fb4cba,
  address: AztecAddress { inner: 0x25de5f29a6d515e67d0ac85f399098acd03c5d4a4884b8c560aee68014715ef1 },
  partial_address: PartialAddress { inner: 0x1420f3a4c4a589be4ea00c0a131dc00127d566725b21dcd1fb421a724710e66b },
  portal_contract_address: EthAddress { inner: 0x0000000000000000000000000000000000005ba0 },
  contract_class_id: ContractClassId { inner: 0x0ce2a998337b1e6da1ac1d802a8bb9e10b7d705d210e61efb9642855009814a6 },
  public_keys_hash: PublicKeysHash { inner: 0x000000000000000000000000000000000000000000000000000000000000b26e },
  salted_initialization_hash: SaltedInitializationHash { inner: 0x2003637d02f08887d36dc2f27dd961f51f17e0a8a43dc0268dfcefcd96efc3a4 }
};

// taken from __snapshots__/noir_test_gen.test.ts.snap
global parent_contract = ContractData {
  contract_address_salt: 0x0000000000000000000000000000000000000000000000000000000000001618,
  artifact_hash: 0x00000000000000000000000000000000000000000000000000000000000004bc,
  public_bytecode_commitment: 0x1435ed970b275bebf95de3df53f23f3d2e97c9b54cf442bb03a3fa17a0ee3cd7,
  private_functions_root: 0x2c1c949cb226995de94b7b8b5aeaab440739f2dfeb06d358441f60932cf243a7,
  address: AztecAddress { inner: 0x178916e52e64b3880e7f837ab30e58222b2971f0150cbc16060bb3589fd96e23 },
  partial_address: PartialAddress { inner: 0x009a6e74b3cebfe0351ceec448d722aa973c1c9da63bbf9f902f1896f3fa1832 },
  portal_contract_address: EthAddress { inner: 0x0000000000000000000000000000000000000913 },
  contract_class_id: ContractClassId { inner: 0x1f1f963a350e2c883cc6730c19fc5d5b47a40694d805cbb0720fa76fe295df90 },
  public_keys_hash: PublicKeysHash { inner: 0x00000000000000000000000000000000000000000000000000000000000011c1 },
  salted_initialization_hash: SaltedInitializationHash { inner: 0x275e153c147f9cb61a5e7b354e81484696d6a54ffc251dcf4ed8276ab24868d2 }
};
