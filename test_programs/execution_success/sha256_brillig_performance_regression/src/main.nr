// Performance regression extracted from an aztec protocol contract.

unconstrained fn main(
    input_asset_bridge_portal_address: EthAddress,
    input_amount: Field,
    uniswap_fee_tier: Field,
    output_asset_bridge_portal_address: EthAddress,
    minimum_output_amount: Field,
    aztec_recipient: AztecAddress,
    secret_hash_for_L1_to_l2_message: Field,
    caller_on_L1: EthAddress,
) -> pub Field {
    let mut hash_bytes = [0; 260]; // 8 fields of 32 bytes each + 4 bytes fn selector
    let input_token_portal_bytes: [u8; 32] =
        input_asset_bridge_portal_address.to_field().to_be_bytes();
    let in_amount_bytes: [u8; 32] = input_amount.to_be_bytes();
    let uniswap_fee_tier_bytes: [u8; 32] = uniswap_fee_tier.to_be_bytes();
    let output_token_portal_bytes: [u8; 32] =
        output_asset_bridge_portal_address.to_field().to_be_bytes();
    let amount_out_min_bytes: [u8; 32] = minimum_output_amount.to_be_bytes();
    let aztec_recipient_bytes: [u8; 32] = aztec_recipient.to_field().to_be_bytes();
    let secret_hash_for_L1_to_l2_message_bytes: [u8; 32] =
        secret_hash_for_L1_to_l2_message.to_be_bytes();
    let caller_on_L1_bytes: [u8; 32] = caller_on_L1.to_field().to_be_bytes();

    // The purpose of including the following selector is to make the message unique to that specific call. Note that
    // it has nothing to do with calling the function.
    let selector = comptime {
        std::hash::keccak256(
            "swap_public(address,uint256,uint24,address,uint256,bytes32,bytes32,address)".as_bytes(),
            75,
        )
    };

    hash_bytes[0] = selector[0];
    hash_bytes[1] = selector[1];
    hash_bytes[2] = selector[2];
    hash_bytes[3] = selector[3];

    for i in 0..32 {
        hash_bytes[i + 4] = input_token_portal_bytes[i];
        hash_bytes[i + 36] = in_amount_bytes[i];
        hash_bytes[i + 68] = uniswap_fee_tier_bytes[i];
        hash_bytes[i + 100] = output_token_portal_bytes[i];
        hash_bytes[i + 132] = amount_out_min_bytes[i];
        hash_bytes[i + 164] = aztec_recipient_bytes[i];
        hash_bytes[i + 196] = secret_hash_for_L1_to_l2_message_bytes[i];
        hash_bytes[i + 228] = caller_on_L1_bytes[i];
    }

    let content_hash = sha256_to_field(hash_bytes);
    content_hash
}

// Convert a 32 byte array to a field element by truncating the final byte
pub fn field_from_bytes_32_trunc(bytes32: [u8; 32]) -> Field {
    // Convert it to a field element
    let mut v = 1;
    let mut high = 0 as Field;
    let mut low = 0 as Field;

    for i in 0..15 {
        // covers bytes 16..30 (31 is truncated and ignored)
        low = low + (bytes32[15 + 15 - i] as Field) * v;
        v = v * 256;
        // covers bytes 0..14
        high = high + (bytes32[14 - i] as Field) * v;
    }
    // covers byte 15
    low = low + (bytes32[15] as Field) * v;

    low + high * v
}

pub fn sha256_to_field<let N: u32>(bytes_to_hash: [u8; N]) -> Field {
    let sha256_hashed = std::hash::sha256(bytes_to_hash);
    let hash_in_a_field = field_from_bytes_32_trunc(sha256_hashed);

    hash_in_a_field
}

pub trait ToField {
    fn to_field(self) -> Field;
}

pub struct EthAddress {
    inner: Field,
}

impl ToField for EthAddress {
    fn to_field(self) -> Field {
        self.inner
    }
}

pub struct AztecAddress {
    pub inner: Field,
}

impl ToField for AztecAddress {
    fn to_field(self) -> Field {
        self.inner
    }
}
