use dep::aztec::hash::sha256_to_field;

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
    let content_hash = sha256_to_field(hash_bytes);
    content_hash
}

// This method computes the L2 to L1 message content hash for the public
// refer `l1-contracts/test/portals/UniswapPortal.sol` on how L2 to L1 message is expected
fn compute_swap_public_content_hash(
    input_asset_bridge_portal_address: Field,
    input_amount: Field,
    uniswap_fee_tier: Field,
    output_asset_bridge_portal_address: Field,
    minimum_output_amount: Field,
    aztec_recipient: Field,
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
    let aztec_recipient_bytes = aztec_recipient.to_be_bytes(32);
    let secret_hash_for_L1_to_l2_message_bytes = secret_hash_for_L1_to_l2_message.to_be_bytes(32);
    let deadline_for_L1_to_l2_message_bytes = deadline_for_L1_to_l2_message.to_be_bytes(32);
    let canceller_bytes = canceller_for_L1_to_L2_message.to_be_bytes(32);
    let caller_on_L1_bytes = caller_on_L1.to_be_bytes(32);

    // function selector: 0xf3068cac keccak256("swap_public(address,uint256,uint24,address,uint256,bytes32,bytes32,uint32,address,address)")
    hash_bytes[0] = 0xf3;
    hash_bytes[1] = 0x06;
    hash_bytes[2] = 0x8c;
    hash_bytes[3] = 0xac;

    for i in 0..32 {
        hash_bytes[i + 4] = input_token_portal_bytes[i];
        hash_bytes[i + 36] = in_amount_bytes[i];
        hash_bytes[i + 68] = uniswap_fee_tier_bytes[i];
        hash_bytes[i + 100] = output_token_portal_bytes[i];
        hash_bytes[i + 132] = amount_out_min_bytes[i];
        hash_bytes[i + 164] = aztec_recipient_bytes[i];
        hash_bytes[i + 196] = secret_hash_for_L1_to_l2_message_bytes[i];
        hash_bytes[i + 228] = deadline_for_L1_to_l2_message_bytes[i];
        hash_bytes[i + 260] = canceller_bytes[i];
        hash_bytes[i + 292] = caller_on_L1_bytes[i];
    }

    let content_hash = sha256_to_field(hash_bytes);
    content_hash
}
