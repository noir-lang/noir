pub mod contract_functions;
pub mod contracts;
pub mod merkle_tree;
pub mod protocol_contract_tree;
pub mod vk_tree;
pub mod public_data_tree;

use crate::address::AztecAddress;

pub global MSG_SENDER: AztecAddress = AztecAddress { inner: 27 };

pub global CHAIN_ID: Field = 1;

pub global VERSION: Field = 3;
