struct AccountContractInterface {
    address: Field,
}

impl AccountContractInterface {
    pub fn at(address: Field) -> Self {
        AccountContractInterface { address }
    }

    pub fn send_rewards(_self: Self, _rewards: u8) {}
}
