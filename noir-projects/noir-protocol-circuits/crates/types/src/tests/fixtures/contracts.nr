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
  private_functions_root: 0x04c207a8a83f20ee6c6dbeb40636e8132bc18e971927f7d81f18cb92705ee817,
  address: AztecAddress { inner: 0x10f0c308adf33bceef3ed638bf8a5b403691eefc4ee7a0c20f0438c8a7096226 },
  partial_address: PartialAddress { inner: 0x08e858be915646809621b06c4fcea0bfcf2817b07c1885e9a0497554638be93f },
  contract_class_id: ContractClassId { inner: 0x227e7a3a22b2a95ce3b91fbb47a80669d7d68ee34db7bf998cb9457ee59f3179 },
  public_keys_hash: PublicKeysHash { inner: 0x000000000000000000000000000000000000000000000000000000000000b26e },
  salted_initialization_hash: SaltedInitializationHash { inner: 0x13a939daa511233e5446905ed2cadbee14948fa75df183b53b5c14b612bffe88 },
  deployer: AztecAddress { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }
};

// taken from __snapshots__/noir_test_gen.test.ts.snap
global parent_contract = ContractData {
  contract_address_salt: 0x0000000000000000000000000000000000000000000000000000000000001618,
  artifact_hash: 0x00000000000000000000000000000000000000000000000000000000000004bc,
  public_bytecode_commitment: 0x0000000000000000000000000000000000000000000000000000000000000005,
  private_functions_root: 0x13911dea3b1d55fe459c95739f42df2cd7c2c610ab16a5632ab649760f3d0522,
  address: AztecAddress { inner: 0x0df89b4b442718bc22a06f5ac353e83562adb5c48465dd2663e90648067ab87b },
  partial_address: PartialAddress { inner: 0x2e150667b5cd21814d880d7c853a637d3818fc545209a8a90521079351e836c7 },
  contract_class_id: ContractClassId { inner: 0x198333ba87796984608fe55a13c975fd3fa8ce721ee4d4b97b6886818a1d8fdb },
  public_keys_hash: PublicKeysHash { inner: 0x00000000000000000000000000000000000000000000000000000000000011c1 },
  salted_initialization_hash: SaltedInitializationHash { inner: 0x24bd6ac7a182e2cf25e437c72f53544ef81dfd97d9afee23abb07a638e7be749 },
  deployer: AztecAddress { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }
};
