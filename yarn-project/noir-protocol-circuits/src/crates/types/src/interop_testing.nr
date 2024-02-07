use crate::grumpkin_point::GrumpkinPoint;
use crate::transaction::request::TxRequest;
use crate::address::{AztecAddress, EthAddress, PartialAddress, PublicKeysHash};
use crate::transaction::context::TxContext;
use crate::abis::function_data::FunctionData;
use crate::abis::function_leaf_preimage::FunctionLeafPreimage;
use crate::contrakt::deployment_data::ContractDeploymentData;
use crate::abis::function_selector::FunctionSelector;
use crate::hash::{compute_l2_to_l1_hash, sha256_to_field, hash_args};
use crate::abis::public_call_stack_item::PublicCallStackItem;
use crate::abis::public_circuit_public_inputs::PublicCircuitPublicInputs;
use crate::abis::side_effect::SideEffect;
use crate::contract_class::ContractClassId;

#[test]
fn compute_address() {
    let point = GrumpkinPoint { x: 1, y: 2 };
    let contract_address_salt = 3;
    let contract_class_id = ContractClassId::from_field(4);
    let initialization_hash = 5;
    let portal_contract_address = EthAddress::from_field(6);

    let address = AztecAddress::compute_from_public_key(
        point,
        contract_class_id,
        contract_address_salt,
        initialization_hash,
        portal_contract_address
    );

    assert(address.to_field() == 0x2fd71a4f0742364f194dd16d0ae32d2f47845ddc7f5d328f37d4148b565c4123);
}

#[test]
fn compute_public_keys_hash() {
    let point = GrumpkinPoint { x: 1, y: 2 };
    let actual = PublicKeysHash::compute(point);
    assert(actual.to_field() == 0x1923a6246e305720b6aaf751fde0342613e93c82e455c3831e28375c16dd40d8);
}

#[test]
fn compute_address_from_partial_and_pubkey() {
    let point = GrumpkinPoint { x: 1, y: 2 };
    let partial_address = PartialAddress::from_field(3);

    let address = AztecAddress::compute(PublicKeysHash::compute(point), partial_address);
    assert(address.to_field() == 0x0447f893197175723deb223696e2e96dbba1e707ee8507766373558877e74197);
}

#[test]
fn compute_var_args_hash() {
    let mut input = [0; 800];
    for i in 0..800 {
        input[i] = i as Field;
    }
    let hash = hash_args(input);
    assert(hash == 1557627899280963684159398665725097926236612957540256425197580046184563077271);
}

#[test]
fn compute_tx_request_hash() {
    let tx_request = TxRequest {
        origin: AztecAddress::from_field(1),
        args_hash: 3,
        tx_context: TxContext {
            is_fee_payment_tx: false,
            is_rebate_payment_tx: false,
            is_contract_deployment_tx: true,
            contract_deployment_data: ContractDeploymentData {
                public_key: GrumpkinPoint { x: 1, y: 2 },
                initialization_hash: 1,
                contract_class_id: ContractClassId::from_field(2),
                contract_address_salt: 3,
                portal_contract_address: EthAddress::from_field(1)
            },
            chain_id: 0,
            version: 0
        },
        function_data: FunctionData { selector: FunctionSelector::from_u32(2), is_internal: false, is_private: true, is_constructor: true }
    };
    assert(tx_request.hash() == 0x0b487ff2900ae1178e131bfe333fdbc351beef658f7c0d62db2801429b1aab75);
}

#[test]
fn compute_l2_l1_hash() {
    // All zeroes
    let hash_result = compute_l2_to_l1_hash(AztecAddress::from_field(0), 0, EthAddress::from_field(0), 0, 0);
    assert(hash_result == 0x2266ac2f9f0c19c015239ef5ea85862fc6fac00db73779b220a4d49c4856c2e1);

    // Non-zero case
    let hash_result = compute_l2_to_l1_hash(AztecAddress::from_field(1), 2, EthAddress::from_field(3), 4, 5);
    assert(hash_result == 0x0f24729168d4450a5681beafa5e3a899ac28bd17bf5a4877dab37bcd834e1634);
}

#[test]
fn smoke_sha256_to_field() {
    let full_buffer = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
        20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
        40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59,
        60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79,
        80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99,
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119,
        120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139,
        140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159
    ];
    let result = sha256_to_field(full_buffer);
    assert(result == 0x142a6d57007171f6eaa33d55976d9dbe739c889c8e920f115f7808dea184c718);
}

#[test]
fn compute_function_leaf() {
    let leaf = FunctionLeafPreimage { selector: FunctionSelector::from_u32(27), is_internal: false, is_private: true, vk_hash: 1, acir_hash: 2 };

    assert_eq(leaf.hash(), 0x1ad8ece7f40e63d011ae47c6ce6cdaf31d632a23f5cf35bbeaaf69c8302afdbc);
}

#[test]
fn compute_call_stack_item_request_hash() {
    let contract_address = AztecAddress::from_field(1);
    let function_data = FunctionData { selector: FunctionSelector::from_u32(2), is_internal: false, is_private: false, is_constructor: false };

    let mut public_inputs: PublicCircuitPublicInputs = dep::std::unsafe::zeroed();
    public_inputs.new_commitments[0] = SideEffect{
        value: 1,
        counter: 0,
    };

    let call_stack_item = PublicCallStackItem { contract_address, public_inputs, is_execution_request: true, function_data };

    // Value from abis.test.ts "Computes a callstack item request hash" test
    assert_eq(call_stack_item.hash(), 0x28cb4c264eb11e13c77a5e60b251a6b3edfb4cfed786c4f1ec5e1749fede9b78);
}

#[test]
fn compute_call_stack_item_hash() {
    let contract_address = AztecAddress::from_field(1);
    let function_data = FunctionData { selector: FunctionSelector::from_u32(2), is_internal: false, is_private: false, is_constructor: false };

    let mut public_inputs: PublicCircuitPublicInputs = dep::std::unsafe::zeroed();
    public_inputs.new_commitments[0] = SideEffect{
        value: 1,
        counter: 0,
    };

    let call_stack_item = PublicCallStackItem { contract_address, public_inputs, is_execution_request: false, function_data };

    // Value from abis.test.ts "Computes a callstack item hash" test
    assert_eq(call_stack_item.hash(), 0x17095bbb9e3e104dc520b6eec9338e9b6ce1fdc0f3a66b06d5ca90b15909ad0e);
}
