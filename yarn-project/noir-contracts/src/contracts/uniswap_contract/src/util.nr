use dep::std::hash::{pedersen_with_separator, sha256};
use dep::aztec::constants_gen::GENERATOR_INDEX__SIGNATURE_PAYLOAD;

fn compute_message_hash<N>(args: [Field; N]) -> Field {
  // @todo @lherskind We should probably use a separate generator for this, 
  //                  to avoid any potential collisions with payloads.
  pedersen_with_separator(args, GENERATOR_INDEX__SIGNATURE_PAYLOAD)[0]
}

// This method computes the L2 to L1 message content hash for the private
// refer `l1-contracts/test/portals/UniswapPortal.sol` on how L2 to L1 message is expected
fn compute_swap_private_content_hash(
    input_asset_bridge_portal_address: Field,
    input_amount: Field,
    uniswap_fee_tier: Field,
    output_asset_bridge_portal_address: Field,
    minimum_output_amount: Field,
    secret_hash_for_redeeming_minted_notes: Field,
    secret_hash_for_L1_to_l2_message: Field,
    deadline_for_L1_to_l2_message: Field,
    canceller_for_L1_to_L2_message: Field,
    caller_on_L1: Field,
) -> Field {
    let mut hash_bytes: [u8; 324] = [0; 324]; // 10 fields of 32 bytes each + 4 bytes fn selector

    let input_token_portal_bytes = input_asset_bridge_portal_address.to_be_bytes(32);
    let in_amount_bytes = input_amount.to_be_bytes(32);
    let uniswap_fee_tier_bytes = uniswap_fee_tier.to_be_bytes(32);
    let output_token_portal_bytes = output_asset_bridge_portal_address.to_be_bytes(32);
    let amount_out_min_bytes = minimum_output_amount.to_be_bytes(32);
    let secret_hash_for_redeeming_minted_notes_bytes = secret_hash_for_redeeming_minted_notes.to_be_bytes(32);
    let secret_hash_for_L1_to_l2_message_bytes = secret_hash_for_L1_to_l2_message.to_be_bytes(32);
    let deadline_for_L1_to_l2_message_bytes = deadline_for_L1_to_l2_message.to_be_bytes(32);
    let canceller_bytes = canceller_for_L1_to_L2_message.to_be_bytes(32);
    let caller_on_L1_bytes = caller_on_L1.to_be_bytes(32);

    // function selector: 0xbd87d14b keccak256("swap_private(address,uint256,uint24,address,uint256,bytes32,bytes32,uint32,address,address)")
    hash_bytes[0] = 0xbd;
    hash_bytes[1] = 0x87;
    hash_bytes[2] = 0xd1;
    hash_bytes[3] = 0x4b;

    for i in 0..32 {
        hash_bytes[i + 4] = input_token_portal_bytes[i];
        hash_bytes[i + 36] = in_amount_bytes[i];
        hash_bytes[i + 68] = uniswap_fee_tier_bytes[i];
        hash_bytes[i + 100] = output_token_portal_bytes[i];
        hash_bytes[i + 132] = amount_out_min_bytes[i];
        hash_bytes[i + 164] = secret_hash_for_redeeming_minted_notes_bytes[i];
        hash_bytes[i + 196] = secret_hash_for_L1_to_l2_message_bytes[i];
        hash_bytes[i + 228] = deadline_for_L1_to_l2_message_bytes[i];
        hash_bytes[i + 260] = canceller_bytes[i];
        hash_bytes[i + 292] = caller_on_L1_bytes[i];
    }

    let content_sha256 = sha256(hash_bytes);

    // Convert the content_sha256 to a field element
    let mut v = 1;
    let mut high = 0 as Field;
    let mut low = 0 as Field;

    for i in 0..16 {
        high = high + (content_sha256[15 - i] as Field) * v;
        low = low + (content_sha256[16 + 15 - i] as Field) * v;
        v = v * 256;
    }

    // Abuse that a % p + b % p = (a + b) % p and that low < p
    let content_hash = low + high * v;

    content_hash
}
