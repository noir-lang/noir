use crate::{
    hash::poseidon2_hash,
    merkle_tree::leaf_preimage::{IndexedTreeLeafPreimage, LeafPreimage},
    traits::Empty,
};

/**
    Exists to show:
        - Membership of a computed protocol contract address in the tree, or
        - Non-membership of a computed non-protocol contract address in the tree
    in private_call_data_validator/validate_contract_address.nr.
    The tree is created never updated within the kernels, so we don't need the functions which update leaves.
*/

pub struct ProtocolContractLeafPreimage {
    pub address: Field,
    pub next_address: Field,
}

impl Empty for ProtocolContractLeafPreimage {
    fn empty() -> Self {
        Self { address: 0, next_address: 0 }
    }
}

impl LeafPreimage for ProtocolContractLeafPreimage {
    fn get_key(self) -> Field {
        self.address
    }

    fn as_leaf(self) -> Field {
        poseidon2_hash([self.address, self.next_address])
    }
}

impl IndexedTreeLeafPreimage<Field> for ProtocolContractLeafPreimage {
    fn get_next_key(self) -> Field {
        self.next_address
    }

    fn points_to_infinity(self) -> bool {
        // Unimplemented, not required
        false
    }

    fn update_pointers(self, _next_key: Field, _next_index: u32) -> Self {
        assert(false, "Tried to update a static protocol contract index");
        Self::empty()
    }

    fn update_value(self, _value: Field) -> Self {
        assert(false, "Tried to update a static protocol contract address");
        Self::empty()
    }

    fn build_insertion_leaf(_value: Field, _low_leaf: Self) -> Self {
        assert(false, "Tried to update a static protocol contract address");
        Self::empty()
    }
}

impl Eq for ProtocolContractLeafPreimage {
    fn eq(self, other: Self) -> bool {
        (self.address == other.address) & (self.next_address == other.next_address)
    }
}
