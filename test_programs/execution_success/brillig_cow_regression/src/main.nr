// Tests a performance regression found in aztec-packages with brillig cow optimization
global MAX_NOTE_HASHES_PER_TX: u32 = 64;
global MAX_NULLIFIERS_PER_TX: u32 = 64;
global MAX_L2_TO_L1_MSGS_PER_TX: u32 = 2;
global MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX: u32 = 16;
global MAX_NEW_CONTRACTS_PER_TX: u32 = 1;
global NUM_ENCRYPTED_LOGS_HASHES_PER_TX: u32 = 1;
global NUM_UNENCRYPTED_LOGS_HASHES_PER_TX: u32 = 1;
global NUM_FIELDS_PER_SHA256: u32 = 2;
global TX_EFFECT_HASH_INPUT_SIZE: u32 = 169;
global TX_EFFECT_HASH_LOG_FIELDS: u32 = 4;
global TX_EFFECT_HASH_FULL_FIELDS: u32 = 165;

struct PublicDataUpdateRequest {
    leaf_slot: Field,
    old_value: Field,
    new_value: Field,
}

struct NewContractData {
    contract_address: Field,
    portal_contract_address: Field,
}

impl NewContractData {
    fn hash(self) -> Field {
        std::hash::pedersen_hash([self.contract_address, self.portal_contract_address])
    }
}

struct DataToHash {
    new_note_hashes: [Field; MAX_NOTE_HASHES_PER_TX],
    new_nullifiers: [Field; MAX_NULLIFIERS_PER_TX],
    public_data_update_requests: [PublicDataUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    new_l2_to_l1_msgs: [Field; MAX_L2_TO_L1_MSGS_PER_TX],
    encrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],
    unencrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],
    new_contracts: [NewContractData; MAX_NEW_CONTRACTS_PER_TX],
}

struct U256 {
    // This is in big-endian order, typically because
    // sha256 is usually in big endian order.
    // Note: this means that inner[0] has the most significant 64 bits.
    inner: [u64; 4],
}

impl U256 {
    pub fn from_bytes32(bytes: [u8; 32]) -> U256 {
        // We use addition rather than a bitwise OR as the bitshifts ensure that none of the bytes overlap each other.
        let high_0 = ((bytes[0] as u64) << 56)
            + ((bytes[1] as u64) << 48)
            + ((bytes[2] as u64) << 40)
            + ((bytes[3] as u64) << 32)
            + ((bytes[4] as u64) << 24)
            + ((bytes[5] as u64) << 16)
            + ((bytes[6] as u64) << 8)
            + (bytes[7] as u64);

        let high_1 = ((bytes[8] as u64) << 56)
            + ((bytes[9] as u64) << 48)
            + ((bytes[10] as u64) << 40)
            + ((bytes[11] as u64) << 32)
            + ((bytes[12] as u64) << 24)
            + ((bytes[13] as u64) << 16)
            + ((bytes[14] as u64) << 8)
            + (bytes[15] as u64);

        let low_0 = ((bytes[16] as u64) << 56)
            + ((bytes[17] as u64) << 48)
            + ((bytes[18] as u64) << 40)
            + ((bytes[19] as u64) << 32)
            + ((bytes[20] as u64) << 24)
            + ((bytes[21] as u64) << 16)
            + ((bytes[22] as u64) << 8)
            + (bytes[23] as u64);

        let low_1 = ((bytes[24] as u64) << 56)
            + ((bytes[25] as u64) << 48)
            + ((bytes[26] as u64) << 40)
            + ((bytes[27] as u64) << 32)
            + ((bytes[28] as u64) << 24)
            + ((bytes[29] as u64) << 16)
            + ((bytes[30] as u64) << 8)
            + (bytes[31] as u64);

        U256 { inner: [high_0, high_1, low_0, low_1] }
    }

    pub fn to_u128_limbs(self) -> [Field; 2] {
        let two_pow_64 = 2.pow_32(64);

        let high = (self.inner[0] as Field) * two_pow_64 + self.inner[1] as Field;
        let low = (self.inner[2] as Field) * two_pow_64 + self.inner[3] as Field;

        [high, low]
    }
}

unconstrained fn main(kernel_data: DataToHash) -> pub [Field; NUM_FIELDS_PER_SHA256] {
    let mut tx_effects_hash_inputs = [0; TX_EFFECT_HASH_INPUT_SIZE];

    let new_note_hashes = kernel_data.new_note_hashes;
    let new_nullifiers = kernel_data.new_nullifiers;
    let public_data_update_requests = kernel_data.public_data_update_requests;
    let l2ToL1Msgs = kernel_data.new_l2_to_l1_msgs;
    let encryptedLogsHash = kernel_data.encrypted_logs_hash;
    let unencryptedLogsHash = kernel_data.unencrypted_logs_hash;

    let mut offset = 0;

    for j in 0..MAX_NOTE_HASHES_PER_TX {
        tx_effects_hash_inputs[offset + j] = new_note_hashes[j];
    }
    offset += MAX_NOTE_HASHES_PER_TX;

    for j in 0..MAX_NULLIFIERS_PER_TX {
        tx_effects_hash_inputs[offset + j] = new_nullifiers[j];
    }
    offset += MAX_NULLIFIERS_PER_TX;

    for j in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX {
        tx_effects_hash_inputs[offset + j * 2] = public_data_update_requests[j].leaf_slot;
        tx_effects_hash_inputs[offset + j * 2 + 1] = public_data_update_requests[j].new_value;
    }
    offset += MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 2;

    for j in 0..MAX_L2_TO_L1_MSGS_PER_TX {
        tx_effects_hash_inputs[offset + j] = l2ToL1Msgs[j];
    }
    offset += MAX_L2_TO_L1_MSGS_PER_TX;

    let contract_leaf = kernel_data.new_contracts[0];
    tx_effects_hash_inputs[offset] = contract_leaf.hash();

    offset += MAX_NEW_CONTRACTS_PER_TX;

    let new_contracts = kernel_data.new_contracts;
    tx_effects_hash_inputs[offset] = new_contracts[0].contract_address;

    tx_effects_hash_inputs[offset + 1] = new_contracts[0].portal_contract_address;

    offset += MAX_NEW_CONTRACTS_PER_TX * 2;

    for j in 0..NUM_FIELDS_PER_SHA256 {
        tx_effects_hash_inputs[offset + j] = encryptedLogsHash[j];
    }

    offset += NUM_ENCRYPTED_LOGS_HASHES_PER_TX * NUM_FIELDS_PER_SHA256;

    for j in 0..NUM_FIELDS_PER_SHA256 {
        tx_effects_hash_inputs[offset + j] = unencryptedLogsHash[j];
    }

    offset += NUM_UNENCRYPTED_LOGS_HASHES_PER_TX * NUM_FIELDS_PER_SHA256;
    assert_eq(offset, TX_EFFECT_HASH_INPUT_SIZE); // Sanity check
    let mut hash_input_flattened =
        [0; TX_EFFECT_HASH_FULL_FIELDS * 32 + TX_EFFECT_HASH_LOG_FIELDS * 16];
    for offset in 0..TX_EFFECT_HASH_FULL_FIELDS {
        let input_as_bytes: [u8; 32] = tx_effects_hash_inputs[offset].to_be_bytes();
        for byte_index in 0..32 {
            hash_input_flattened[offset * 32 + byte_index] = input_as_bytes[byte_index];
        }
    }

    for log_field_index in 0..TX_EFFECT_HASH_LOG_FIELDS {
        let input_as_bytes: [u8; 16] =
            tx_effects_hash_inputs[TX_EFFECT_HASH_FULL_FIELDS + log_field_index].to_be_bytes();
        for byte_index in 0..16 {
            hash_input_flattened[TX_EFFECT_HASH_FULL_FIELDS * 32 + log_field_index * 16 + byte_index] =
                input_as_bytes[byte_index];
        }
    }

    let sha_digest = std::hash::sha256(hash_input_flattened);
    U256::from_bytes32(sha_digest).to_u128_limbs()
}
