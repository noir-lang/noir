mod contract_functions;
mod contract_tree;
mod contracts;
mod note_hash_tree;
mod read_requests;

use crate::{
    abis::{
        append_only_tree_snapshot::AppendOnlyTreeSnapshot,
        global_variables::GlobalVariables,
    },
    address::{
        AztecAddress,
        EthAddress,
    },
    constants::NUM_FIELDS_PER_SHA256,
    grumpkin_point::GrumpkinPoint,
    header::Header,
    partial_state_reference::PartialStateReference,
    state_reference::StateReference,
    tests::fixtures
};

global MSG_SENDER = AztecAddress { inner: 27 };

global PUBLIC_KEY = GrumpkinPoint { x: 123456789, y: 123456789 };

// Workaround for https://github.com/noir-lang/noir/issues/1440
fn empty_append_only_tree() -> AppendOnlyTreeSnapshot {
    AppendOnlyTreeSnapshot::zero()
}

// Workaround for https://github.com/noir-lang/noir/issues/1440
fn empty_eth_address() -> EthAddress {
    EthAddress::empty()
}

// Workaround for https://github.com/noir-lang/noir/issues/1440
fn empty_aztec_address() -> AztecAddress {
    AztecAddress::empty()
}

global HEADER = Header {
    last_archive: empty_append_only_tree(),
    body_hash: [0; NUM_FIELDS_PER_SHA256],
    state: StateReference {
        l1_to_l2_message_tree: empty_append_only_tree(),
        partial: PartialStateReference {
            note_hash_tree: AppendOnlyTreeSnapshot {
                root: fixtures::note_hash_tree::ROOT,
                next_available_leaf_index: 0, // TODO: should this be populated?
            },
            nullifier_tree: empty_append_only_tree(),
            contract_tree: AppendOnlyTreeSnapshot {
                root: fixtures::contract_tree::ROOT,
                next_available_leaf_index: 0, // TODO: should this be populated?
            },
            public_data_tree: empty_append_only_tree()
        }
    },
    global_variables: GlobalVariables {
        chain_id: 0,
        version: 0,
        block_number: 0,
        timestamp: 0,
        coinbase: empty_eth_address(),
        fee_recipient: empty_aztec_address(),
    }
};
