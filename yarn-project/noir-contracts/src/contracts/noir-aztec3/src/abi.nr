global MAX_ARGS: comptime Field = 8;
global MAX_RETURN_VALUES: comptime Field = 4;
global MAX_LOGS: comptime Field = 4;
global MAX_NEW_COMMITMENTS: comptime Field = 4;
global MAX_NEW_NULLIFIERS: comptime Field = 4;
global MAX_PRIVATE_CALL_STACK : comptime Field = 4;
global MAX_PUBLIC_CALL_STACK: comptime Field = 4;
global MAX_L1_MSG_STACK : comptime Field = 2;

use crate::note::Note;
use crate::vec::BoundedVec;

// Inputs are expected to be provided to each contract function
struct Inputs {
    call_context : CallContext,

    old_nullifier_tree_root : Field,
    old_contract_tree_root : Field,

    contract_deployment_data: ContractDeploymentData,
    args: [Field; MAX_ARGS],
}

struct CallContext {
    msg_sender : Field,
    storage_contract_address : Field,
    portal_contract_address : Field,

    is_delegate_call : bool,
    is_static_call : bool,
    is_contract_deployment: bool,
}

struct ContractDeploymentData {
    constructor_vk_hash : Field,
    function_tree_root : Field,
    contract_address_salt : Field,
    portal_contract_address : Field,
    hide_private_function_data : bool,
}

struct Outputs {
    return_values: [Field; MAX_RETURN_VALUES],
    logs: [Field; MAX_LOGS],

    new_commitments: [Field; MAX_NEW_COMMITMENTS],
    new_nullifiers: [Field; MAX_NEW_NULLIFIERS],

    private_call_stack: [Field; MAX_PRIVATE_CALL_STACK],
    public_call_stack: [Field; MAX_PUBLIC_CALL_STACK],
    l1_message_stack: [Field; MAX_L1_MSG_STACK],

    old_private_data_tree_root: Field,

    new_note_values: [Field; 4],
    new_note_owner_xs: [Field; 4],
    new_note_owner_ys: [Field; 4],
    new_note_randomness: [Field; 4],
    new_note_nonce: [Field; 4],
    new_note_is_dummies: [bool; 4],

    nullified_note_values: [Field; 4],
    nullified_note_owner_xs: [Field; 4],
    nullified_note_owner_ys: [Field; 4],
    nullified_note_randomness: [Field; 4],
    nullified_note_nonce: [Field; 4],
    nullified_note_is_dummies: [bool; 4],
}

// Equivalent to Outputs, but uses BoundedVecs instead of arrays to
// make it easier to push onto the ends of the arrays.
// When finished, one can call .finish() to convert back to Outputs
struct IntermediateOutputs {
    return_values : BoundedVec<Field, MAX_RETURN_VALUES>,
    logs : BoundedVec<Field, MAX_LOGS>,

    new_commitments: BoundedVec<Field, MAX_NEW_COMMITMENTS>,
    new_nullifiers: BoundedVec<Field, MAX_NEW_NULLIFIERS>,

    private_call_stack : BoundedVec<Field, MAX_PRIVATE_CALL_STACK>,
    public_call_stack : BoundedVec<Field, MAX_PUBLIC_CALL_STACK>,
    l1_message_stack : BoundedVec<Field, MAX_L1_MSG_STACK>,

    old_private_data_tree_root : Field,

    new_note_values: BoundedVec<Field, MAX_NEW_COMMITMENTS>,
    new_note_owner_xs: BoundedVec<Field, MAX_NEW_COMMITMENTS>,
    new_note_owner_ys: BoundedVec<Field, MAX_NEW_COMMITMENTS>,
    new_note_randomness: BoundedVec<Field, MAX_NEW_COMMITMENTS>,
    new_note_nonce: BoundedVec<Field, MAX_NEW_COMMITMENTS>,
    new_note_is_dummies: BoundedVec<bool, MAX_NEW_COMMITMENTS>,

    nullified_note_values: BoundedVec<Field, MAX_NEW_NULLIFIERS>,
    nullified_note_owner_xs: BoundedVec<Field, MAX_NEW_NULLIFIERS>,
    nullified_note_owner_ys: BoundedVec<Field, MAX_NEW_NULLIFIERS>,
    nullified_note_randomness: BoundedVec<Field, MAX_NEW_NULLIFIERS>,
    nullified_note_nonce: BoundedVec<Field, MAX_NEW_NULLIFIERS>,
    nullified_note_is_dummies: BoundedVec<bool, MAX_NEW_NULLIFIERS>,
}

// fn no_op(x: Field) -> Field {
//     let y = x + 1;
//     let z = 0 + 1;
//     ((y - 1) * z * z * z * z) + z - 1
//     // x
// }

// fn no_op_arr_8(xs: [Field; 8]) -> [Field; 8] {
//     let mut out: [Field; 8] = [0,0,0,0,0,0,0,0];
//     for i in 0..xs.len() {
//         out[i] = no_op(xs[i]);
//     }
//     // xs.map(|x| {
//     //     let y = x + 1;
//     //     y - 1
//     // })
//     out
// }

// fn no_op_arr_4(xs: [Field; 4]) -> [Field; 4] {
//     let mut out: [Field; 4] = [0,0,0,0];
//     for i in 0..xs.len() {
//         out[i] = no_op(xs[i]);
//     }
//     // xs.map(|x| {
//     //     let y = x + 1;
//     //     y - 1
//     // })
//     out
// }

// fn no_op_arr_2(xs: [Field; 2]) -> [Field; 2] {
//     let mut out: [Field; 2] = [0,0];
//     for i in 0..xs.len() {
//         out[i] = no_op(xs[i]);
//     }
//     // xs.map(|x| {
//     //     let y = x + 1;
//     //     y - 1
//     // })
//     out
// }

impl Outputs {
    // We're a bit tricky here in that we return IntermediateOutputs
    // instead of Outputs to hide this implementation detail from users
    fn new() -> IntermediateOutputs {
        IntermediateOutputs::new()
    }
}


struct FinalOutputs {
    ord_00_msg_sender : Field,
    ord_01_storage_contract_address : Field,
    ord_02_portal_contract_address : Field,

    ord_03_is_delegate_call : Field,
    ord_04_is_static_call : Field,
    ord_05_is_contract_deployment: Field,

    // ord_11_call_context: CallContext,
    ord_12_args: [Field; MAX_ARGS],
    ord_13_return_values: [Field; MAX_RETURN_VALUES],
    ord_14_logs: [Field; MAX_LOGS],
    ord_15_new_commitments: [Field; MAX_NEW_COMMITMENTS],
    ord_16_new_nullifiers: [Field; MAX_NEW_NULLIFIERS],
    ord_17_private_call_stack: [Field; MAX_PRIVATE_CALL_STACK],
    ord_18_public_call_stack: [Field; MAX_PUBLIC_CALL_STACK],
    ord_19_l1_message_stack: [Field; MAX_L1_MSG_STACK],
    ord_20_old_private_data_tree_root: Field,
    ord_21_old_nullifier_tree_root: Field,
    ord_22_old_contract_tree_root: Field,
    // ord_23_contract_deployment_data: ContractDeploymentData,
    ord_30_constructor_vk_hash : Field,
    ord_31_function_tree_root : Field,
    ord_32_contract_address_salt : Field,
    ord_33_portal_contract_address : Field,
}

impl IntermediateOutputs {
    fn new() -> IntermediateOutputs {
        IntermediateOutputs {
            return_values: BoundedVec::new(0),
            logs: BoundedVec::new(0),

            new_commitments: BoundedVec::new(0),
            new_nullifiers: BoundedVec::new(0),

            private_call_stack: BoundedVec::new(0),
            public_call_stack: BoundedVec::new(0),
            l1_message_stack: BoundedVec::new(0),

            old_private_data_tree_root: 0,

            new_note_values: BoundedVec::new(0),
            new_note_owner_xs: BoundedVec::new(0),
            new_note_owner_ys: BoundedVec::new(0),
            new_note_randomness: BoundedVec::new(0),
            new_note_nonce: BoundedVec::new(0),
            new_note_is_dummies: BoundedVec::new(true),

            nullified_note_values: BoundedVec::new(0),
            nullified_note_owner_xs: BoundedVec::new(0),
            nullified_note_owner_ys: BoundedVec::new(0),
            nullified_note_randomness: BoundedVec::new(0),
            nullified_note_nonce: BoundedVec::new(0),
            nullified_note_is_dummies: BoundedVec::new(true),
        }
    }

    // Convert back to Outputs so it can be returned
    fn finish(self, inputs: Inputs) -> FinalOutputs {
        FinalOutputs {
            ord_00_msg_sender: inputs.call_context.msg_sender ,
            ord_01_storage_contract_address:  inputs.call_context.storage_contract_address,
            ord_02_portal_contract_address:  inputs.call_context.portal_contract_address,

            ord_03_is_delegate_call : inputs.call_context.is_delegate_call as Field,
            ord_04_is_static_call : inputs.call_context.is_static_call as Field,
            ord_05_is_contract_deployment: inputs.call_context.is_contract_deployment as Field,

            // ord_01_call_context: inputs.call_context,
            ord_12_args: inputs.args,
            ord_13_return_values: self.return_values.storage,
            ord_14_logs: self.logs.storage,

            ord_15_new_commitments: self.new_commitments.storage,
            ord_16_new_nullifiers: self.new_nullifiers.storage,

            ord_17_private_call_stack: self.private_call_stack.storage,
            ord_18_public_call_stack: self.public_call_stack.storage,
            ord_19_l1_message_stack: self.l1_message_stack.storage,

            ord_20_old_private_data_tree_root: self.old_private_data_tree_root,

            ord_21_old_nullifier_tree_root: inputs.old_nullifier_tree_root,
            ord_22_old_contract_tree_root: inputs.old_contract_tree_root,

            // ord_13_contract_deployment_data: inputs.contract_deployment_data,

            ord_30_constructor_vk_hash : inputs.contract_deployment_data.constructor_vk_hash,
            ord_31_function_tree_root : inputs.contract_deployment_data.function_tree_root ,
            ord_32_contract_address_salt : inputs.contract_deployment_data.contract_address_salt ,
            ord_33_portal_contract_address : inputs.contract_deployment_data.portal_contract_address ,
            // ord_00_msg_sender :no_op( inputs.call_context.msg_sender as Field),
            // ord_01_storage_contract_address : inputs.call_context.storage_contract_address,
            // ord_02_portal_contract_address : inputs.call_context.portal_contract_address,

            // ord_03_is_delegate_call :no_op( inputs.call_context.is_delegate_call as Field),
            // ord_04_is_static_call : no_op(inputs.call_context.is_static_call as Field),
            // ord_05_is_contract_deployment: no_op(inputs.call_context.is_contract_deployment as Field),

            // // ord_01_call_context: inputs.call_context,
            // ord_12_args: no_op_arr_8(inputs.args),
            // ord_13_return_values: self.return_values.storage,
            // ord_14_logs: self.logs.storage,

            // ord_15_new_commitments: self.new_commitments.storage,
            // ord_16_new_nullifiers: self.new_nullifiers.storage,

            // ord_17_private_call_stack: self.private_call_stack.storage,
            // ord_18_public_call_stack: self.public_call_stack.storage,
            // ord_19_l1_message_stack: self.l1_message_stack.storage,

            // ord_20_old_private_data_tree_root: no_op(self.old_private_data_tree_root as Field),

            // ord_21_old_nullifier_tree_root: no_op(inputs.old_nullifier_tree_root as Field),
            // ord_22_old_contract_tree_root: no_op(inputs.old_contract_tree_root as Field),

            // // ord_13_contract_deployment_data: inputs.contract_deployment_data,

            // ord_30_constructor_vk_hash :no_op( inputs.contract_deployment_data.constructor_vk_hash),
            // ord_31_function_tree_root : no_op(inputs.contract_deployment_data.function_tree_root as Field),
            // ord_32_contract_address_salt : no_op(inputs.contract_deployment_data.contract_address_salt as Field),
            // ord_33_portal_contract_address : no_op(inputs.contract_deployment_data.portal_contract_address as Field),


            // new_note_values: self.new_note_values.storage,
            // new_note_owner_xs: self.new_note_owner_xs.storage,
            // new_note_owner_ys: self.new_note_owner_ys.storage,
            // new_note_randomness: self.new_note_randomness.storage,
            // new_note_nonce: self.new_note_nonce.storage,
            // new_note_is_dummies: self.new_note_is_dummies.storage,

            // nullified_note_values: self.nullified_note_values.storage,
            // nullified_note_owner_xs: self.nullified_note_owner_xs.storage,
            // nullified_note_owner_ys: self.nullified_note_owner_ys.storage,
            // nullified_note_randomness: self.nullified_note_randomness.storage,
            // nullified_note_nonce: self.nullified_note_nonce.storage,
            // nullified_note_is_dummies: self.nullified_note_is_dummies.storage,
        }
    }

    fn push_new_note_hash(mut self: Self, note_hash: Field) -> Self {
        self.new_commitments = self.new_commitments.push(note_hash);
        self
    }

    fn push_new_nullifier(mut self: Self, nullifier: Field) -> Self {
        self.new_nullifiers = self.new_nullifiers.push(nullifier);
        self
    }

    fn push_new_note(mut self: Self, note: Note) -> Self {
        self.new_note_values = self.new_note_values.push(note.value);
        self.new_note_owner_xs = self.new_note_owner_xs.push(note.owner.x);
        self.new_note_owner_ys = self.new_note_owner_ys.push(note.owner.y);
        self.new_note_randomness = self.new_note_randomness.push(note.randomness);
        self.new_note_nonce = self.new_note_nonce.push(note.nonce);
        self.new_note_is_dummies = self.new_note_is_dummies.push(note.is_dummy);
        self
    }

    fn push_nullified_note(mut self: Self, note: Note) -> Self {
        self.nullified_note_values = self.nullified_note_values.push(note.value);
        self.nullified_note_owner_xs = self.nullified_note_owner_xs.push(note.owner.x);
        self.nullified_note_owner_ys = self.nullified_note_owner_ys.push(note.owner.y);
        self.nullified_note_randomness = self.nullified_note_randomness.push(note.randomness);
        self.nullified_note_nonce = self.nullified_note_nonce.push(note.nonce);
        self.nullified_note_is_dummies = self.nullified_note_is_dummies.push(note.is_dummy);
        self
    }
}
