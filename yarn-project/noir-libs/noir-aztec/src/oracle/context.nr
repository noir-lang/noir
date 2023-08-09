#[oracle(getPortalContractAddress)]
fn _get_portal_address(_contract_address: Field) -> Field {}

unconstrained fn get_portal_address(contract_address: Field) -> Field {
    let portal_address = _get_portal_address(contract_address);
    portal_address
}