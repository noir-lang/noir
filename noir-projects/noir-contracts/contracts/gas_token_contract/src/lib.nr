use dep::aztec::context::PublicContext;
use dep::aztec::protocol_types::{address::{AztecAddress, EthAddress}, hash::sha256_to_field};

pub fn calculate_fee(_context: PublicContext) -> U128 {
    U128::from_integer(1)
}

pub fn get_bridge_gas_msg_hash(owner: AztecAddress, amount: Field, canceller: EthAddress) -> Field {
    let mut hash_bytes: [u8; 100] = [0; 100];
    let recipient_bytes = owner.to_field().to_be_bytes(32);
    let amount_bytes = amount.to_be_bytes(32);
    let canceller_bytes = canceller.to_field().to_be_bytes(32);

    for i in 0..32 {
        hash_bytes[i + 4] = recipient_bytes[i];
        hash_bytes[i + 36] = amount_bytes[i];
        hash_bytes[i + 68] = canceller_bytes[i];
    }

    // Function selector: 0xefc2aae6 keccak256('mint_public(bytes32,uint256,address)')
    hash_bytes[0] = 0xef;
    hash_bytes[1] = 0xc2;
    hash_bytes[2] = 0xaa;
    hash_bytes[3] = 0xe6;

    let content_hash = sha256_to_field(hash_bytes);
    content_hash
}
