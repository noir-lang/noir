use dep::aztec::prelude::{AztecAddress, EthAddress};
use dep::aztec::context::interface::PublicContextInterface;
use dep::aztec::protocol_types::hash::sha256_to_field;

pub fn calculate_fee<TPublicContext>(context: TPublicContext) -> Field where TPublicContext: PublicContextInterface {
    context.transaction_fee()
}

pub fn get_bridge_gas_msg_hash(owner: AztecAddress, amount: Field) -> Field {
    let mut hash_bytes = [0; 68];
    let recipient_bytes = owner.to_field().to_be_bytes(32);
    let amount_bytes = amount.to_be_bytes(32);

    for i in 0..32 {
        hash_bytes[i + 4] = recipient_bytes[i];
        hash_bytes[i + 36] = amount_bytes[i];
    }

    // Function selector: 0x3e87b9be keccak256('mint_public(bytes32,uint256)')
    hash_bytes[0] = 0x3e;
    hash_bytes[1] = 0x87;
    hash_bytes[2] = 0xb9;
    hash_bytes[3] = 0xbe;

    let content_hash = sha256_to_field(hash_bytes);
    content_hash
}
