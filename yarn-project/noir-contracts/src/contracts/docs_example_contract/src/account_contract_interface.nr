struct AccountContractInterface {
    address: Field,
}

impl AccountContractInterface {
    fn at(address: Field) -> Self {
        AccountContractInterface { address }
    }

      fn send_rewards(_self: Self, _rewards: u8) {
    }
}