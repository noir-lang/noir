use crate::point::Point;
use crate::address::AztecAddress;
use crate::hash::{compute_partial_address,compute_contract_address_from_partial};

struct CompleteAddress{
    address : AztecAddress,
    public_key : Point, 
    // TODO(David): Can we type this as AztecAddress instead of Field?
    partial_address: Field,
}

impl CompleteAddress{
    fn assert_is_zero(self) {
        self.address.assert_is_zero();
        self.public_key.assert_is_zero();
        assert(self.partial_address == 0);
    }
    
    pub fn compute(point : Point, contract_address_salt : Field, function_tree_root : Field, constructor_hash : Field) -> CompleteAddress {
        let partial_address = compute_partial_address(contract_address_salt, function_tree_root, constructor_hash);
        
        CompleteAddress{
            address : compute_contract_address_from_partial(point, partial_address),
            public_key : point, 
            partial_address,
        }
    }
}
