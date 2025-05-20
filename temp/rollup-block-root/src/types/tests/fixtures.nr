pub(crate) mod contract_functions;
pub(crate) mod contracts;
pub(crate) mod merkle_tree;
pub(crate) mod protocol_contract_tree;
pub(crate) mod vk_tree;
pub(crate) mod public_data_tree;

use crate::types::address::AztecAddress;

pub(crate) global MSG_SENDER: AztecAddress = AztecAddress { inner: 27 };

pub(crate) global CHAIN_ID: Field = 1;

pub(crate) global VERSION: Field = 3;
