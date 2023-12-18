use dep::protocol_types::address::{
    AztecAddress,
    EthAddress,
};

#[oracle(getPortalContractAddress)]
fn _get_portal_address(_contract_address: AztecAddress) -> EthAddress {}

unconstrained pub fn get_portal_address(contract_address: AztecAddress) -> EthAddress {
    let portal_address = _get_portal_address(contract_address);
    portal_address
}
