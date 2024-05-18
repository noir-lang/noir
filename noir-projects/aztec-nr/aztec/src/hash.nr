use dep::protocol_types::{
    address::{AztecAddress, EthAddress},
    constants::{
    GENERATOR_INDEX__SECRET_HASH, GENERATOR_INDEX__MESSAGE_NULLIFIER, ARGS_HASH_CHUNK_COUNT,
    GENERATOR_INDEX__FUNCTION_ARGS, ARGS_HASH_CHUNK_LENGTH
},
    traits::Hash, hash::{pedersen_hash, silo_nullifier, sha256_to_field}
};
use crate::oracle::logs_traits::{LensForEncryptedLog, ToBytesForUnencryptedLog};

pub fn compute_secret_hash(secret: Field) -> Field {
    pedersen_hash([secret], GENERATOR_INDEX__SECRET_HASH)
}

pub fn compute_unencrypted_log_hash<T, N, M>(
    contract_address: AztecAddress,
    event_selector: Field,
    log: T
) -> Field where T: ToBytesForUnencryptedLog<N, M> {
    let message_bytes: [u8; N] = log.to_be_bytes_arr();
    // can't use N - not in scope error
    let n = message_bytes.len();
    let mut hash_bytes = [0; M];
    // Address is converted to 32 bytes in ts
    let address_bytes = contract_address.to_be_bytes_arr();
    for i in 0..32 {
        hash_bytes[i] = address_bytes[i];
    }
    let event_bytes = event_selector.to_be_bytes(4);
    for i in 0..4 {
        hash_bytes[32 + i] = event_bytes[i];
    }
    let len_bytes = (n as Field).to_be_bytes(4);
    for i in 0..4 {
        hash_bytes[36 + i] = len_bytes[i];
    }
    for i in 0..n {
        hash_bytes[40 + i] = message_bytes[i];
    }

    sha256_to_field(hash_bytes)
}

pub fn compute_message_hash(
    sender: EthAddress,
    chain_id: Field,
    recipient: AztecAddress,
    version: Field,
    content: Field,
    secret_hash: Field
) -> Field {
    let mut hash_bytes = [0 as u8; 192];
    let sender_bytes = sender.to_field().to_be_bytes(32);
    let chain_id_bytes = chain_id.to_be_bytes(32);
    let recipient_bytes = recipient.to_field().to_be_bytes(32);
    let version_bytes = version.to_be_bytes(32);
    let content_bytes = content.to_be_bytes(32);
    let secret_hash_bytes = secret_hash.to_be_bytes(32);

    for i in 0..32 {
        hash_bytes[i] = sender_bytes[i];
        hash_bytes[i + 32] = chain_id_bytes[i];
        hash_bytes[i + 64] = recipient_bytes[i];
        hash_bytes[i + 96] = version_bytes[i];
        hash_bytes[i + 128] = content_bytes[i];
        hash_bytes[i + 160] = secret_hash_bytes[i];
    }

    sha256_to_field(hash_bytes)
}

// The nullifier of a l1 to l2 message is the hash of the message salted with the secret and index of the message hash
// in the L1 to L2 message tree
pub fn compute_message_nullifier(message_hash: Field, secret: Field, leaf_index: Field) -> Field {
    pedersen_hash(
        [message_hash, secret, leaf_index],
        GENERATOR_INDEX__MESSAGE_NULLIFIER
    )
}

pub fn compute_siloed_nullifier(address: AztecAddress, nullifier: Field) -> Field {
    silo_nullifier(address, nullifier)
}

struct ArgsHasher {
    fields: [Field],
}

impl Hash for ArgsHasher {
    fn hash(self) -> Field {
        hash_args(self.fields)
    }
}

impl ArgsHasher {
    pub fn new() -> Self {
        Self { fields: [] }
    }

    pub fn add(&mut self, field: Field) {
        self.fields = self.fields.push_back(field);
    }

    pub fn add_multiple<N>(&mut self, fields: [Field; N]) {
        for i in 0..N {
            self.fields = self.fields.push_back(fields[i]);
        }
    }
}

pub fn hash_args_array<N>(args: [Field; N]) -> Field {
    hash_args(args.as_slice())
}

pub fn hash_args(args: [Field]) -> Field {
    if args.len() == 0 {
        0
    } else {
        assert(args.len() < ARGS_HASH_CHUNK_COUNT * ARGS_HASH_CHUNK_LENGTH);
        let mut chunks_hashes = [0; ARGS_HASH_CHUNK_COUNT];
        let mut current_chunk_values = [0; ARGS_HASH_CHUNK_LENGTH];

        let mut current_chunk_index = 0;
        let mut index_inside_current_chunk = 0;
        for i in 0..args.len() {
            current_chunk_values[index_inside_current_chunk] = args[i];
            index_inside_current_chunk+=1;
            if index_inside_current_chunk == ARGS_HASH_CHUNK_LENGTH {
                chunks_hashes[current_chunk_index] = pedersen_hash(current_chunk_values, GENERATOR_INDEX__FUNCTION_ARGS);
                current_chunk_values = [0; ARGS_HASH_CHUNK_LENGTH];
                current_chunk_index+=1;
                index_inside_current_chunk = 0;
            }
        }
        if index_inside_current_chunk > 0 {
            chunks_hashes[current_chunk_index] = pedersen_hash(current_chunk_values, GENERATOR_INDEX__FUNCTION_ARGS);
        }
        pedersen_hash(chunks_hashes, GENERATOR_INDEX__FUNCTION_ARGS)
    }
}

#[test]
fn compute_var_args_hash() {
    let mut input = ArgsHasher::new();
    for i in 0..800 {
        input.add(i as Field);
    }
    let hash = input.hash();
    assert(hash == 0x05a1023fef839ac88731f49ae983e172c1b600a3c8f3393ad0ac25d819ac0f0f);
}

#[test]
fn compute_unenc_log_hash_array() {
    let contract_address = AztecAddress::from_field(0x233a3e0df23b2b15b324194cb4a151f26c0b7333250781d34cc269d85dc334c6);
    let event_selector = 5;
    let log = [
        0x20660de09f35f876e3e69d227b2a35166ad05f09d82d06366ec9b6f65a51fec2,
        0x1b52bfe3b8689761916f76dc3d38aa8810860db325cd39ca611eed980091f01c,
        0x2e559c4045c378a56ad13b9edb1e8de4e7ad3b3aa35cc7ba9ec77f7a68fa43a4,
        0x25d0f689c4a4178a29d59306f2675824d19be6d25e44fa03b03f49c263053dd2,
        0x2d513a722d6f352dc0961f156afdc5e31495b9f0e35cb069261a8e55e2df67fd
    ];
    let hash = compute_unencrypted_log_hash(contract_address, event_selector, log);
    assert(hash == 0x00846d6969c8c2f61d39cd2762efcb0abb14f88d59c2675910251ef2bcffe9a7);
}

#[test]
fn compute_unenc_log_hash_addr() {
    let contract_address = AztecAddress::from_field(0x233a3e0df23b2b15b324194cb4a151f26c0b7333250781d34cc269d85dc334c6);
    let event_selector = 5;
    let log = AztecAddress::from_field(0x26aa302d4715fd8a687453cb26d616b0768027bd54bcae56b09d908ecd9f8303);
    let hash = compute_unencrypted_log_hash(contract_address, event_selector, log);
    assert(hash == 0x00880a801230ea08c98a802a11b4786cba474513875f0fc69a615e81c5f9f21c);
}

#[test]
fn compute_unenc_log_hash_str() {
    let contract_address = AztecAddress::from_field(0x1b401e1146c5c507962287065c81f0ef7590adae3802c533d7549d6bf0a41bd8);
    let event_selector = 5;
    let log = "dummy";
    let hash = compute_unencrypted_log_hash(contract_address, event_selector, log);
    assert(hash == 0x00a78b5347813624ecfd26e5b8bc6146f418b0cfcc8296b5112d09b8ebba9496);
}

#[test]
fn compute_unenc_log_hash_longer_str() {
    let contract_address = AztecAddress::from_field(0x1b401e1146c5c507962287065c81f0ef7590adae3802c533d7549d6bf0a41bd8);
    let event_selector = 5;
    let log = "Hello this is a string";
    let hash = compute_unencrypted_log_hash(contract_address, event_selector, log);
    assert(hash == 0x001f3390ea242afee7ce46dafdbdc4bd4f1cf20cd63850d12d60ff9956712c4f);
}
