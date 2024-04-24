use dep::protocol_types::{
    address::{AztecAddress, EthAddress},
    constants::{
    GENERATOR_INDEX__SECRET_HASH, GENERATOR_INDEX__MESSAGE_NULLIFIER, ARGS_HASH_CHUNK_COUNT,
    GENERATOR_INDEX__FUNCTION_ARGS, ARGS_HASH_CHUNK_LENGTH
},
    traits::Hash, hash::{pedersen_hash, poseidon2_hash, silo_nullifier, sha256_to_field}
};

pub fn compute_secret_hash(secret: Field) -> Field {
    pedersen_hash([secret], GENERATOR_INDEX__SECRET_HASH)
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
