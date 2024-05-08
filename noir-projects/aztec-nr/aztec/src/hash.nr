use dep::protocol_types::{
    address::{AztecAddress, EthAddress},
    constants::{
    GENERATOR_INDEX__SECRET_HASH, GENERATOR_INDEX__MESSAGE_NULLIFIER, ARGS_HASH_CHUNK_COUNT,
    GENERATOR_INDEX__FUNCTION_ARGS, ARGS_HASH_CHUNK_LENGTH
},
    traits::Hash, hash::{pedersen_hash, poseidon2_hash, silo_nullifier, sha256_to_field}
};
use crate::oracle::logs_traits::{LensForEncryptedLog, ToBytesForUnencryptedLog};

pub fn compute_secret_hash(secret: Field) -> Field {
    pedersen_hash([secret], GENERATOR_INDEX__SECRET_HASH)
}

pub fn compute_encrypted_log_hash<N, M, L>(encrypted_log: [Field; M]) -> Field where [Field; N]: LensForEncryptedLog<N, M, L> {
    let mut bytes = [0; L];
    // Note that bytes.append(encrypted_log[i].to_be_bytes(31)) results in bound error
    for i in 0..M - 1 {
        let to_add = encrypted_log[i].to_be_bytes(31);
        for j in 0..31 {
            bytes[i*31 + j] = to_add[j];
        }
    }
    // can't assign as L - not in scope error for: L-31*(M-1)
    let num_bytes = bytes.len() as u32 - 31 * (M - 1);
    let to_add_final = encrypted_log[M - 1].to_be_bytes(num_bytes);
    for j in 0..num_bytes {
        bytes[(M-1)*31 + j] = to_add_final[j];
    }
    sha256_to_field(bytes)
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
fn compute_enc_log_hash_304() {
    let input = [
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0021a0d4aa9989656b592187cf6da1965df53ab2ff2277421e663465cf20d3e9,
        0x00c3969cc350f3474f8187a33ac1317181961f5f94043b07ce888d85a5d20cb5,
        0x0058198041ed1547b056955b5141a5a8a1551b0c8d094255ec9daaf3604d9348,
        0x00247ad96df2e4d984cf795ed7316234743a681f824a45c46253de8bfde48850,
        0x007fc251f4ce44f4e9aba3dbf6567228be28fac85660156f2825ddb0b0577457,
        0x009315851323c6bc2aaa42e23fe5f3be97208f2d8167eafdfc5742d94f2f4dd4,
        0x00b938289e563b0fe01982cd9b8d9e33e3069046768ad01c0fb05e429e7b7909,
        0x00fbcc257a3211f705b471eee763b0f43876a2b2178fab6d2b09bd2b7e086584,
        0x000000000000008c3289b5793b7448f4d45ecde039d004b6f037cad10b5c2336
    ];
    let hash = compute_encrypted_log_hash(input);
    assert(hash == 0x001e3c013994947fe28957a876bf1b2c3a69ac69cc92909efd4f2ae9b972f893);
}

#[test]
fn compute_enc_log_hash_368() {
    let input = [
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x002190697d2a50e229a7a077e0951073f7d51e46679f10466153c308b63b1ea9,
        0x00543e346facc6799b94514c9d461bcc836c04b083b13c2e4544a39130473c1e,
        0x000df76d59526f8f953bcc7d9f77cdaefd36435931f0d7348f794bc275b42ded,
        0x00a6d390ee1723af7f7ac1ae4fc81a266b2370fe07040a36d06dbe242e02413e,
        0x00acbce15b6af1fbe94bd0f7b70f11768265dff77bfe63398f2a053efdfdf26d,
        0x00b8b131b9f42c689beb095ba4f4a836d4d15c9068d0422e9add6ca82b786329,
        0x00661a6a654b38f0f97d404ef5553e0efea9ed670561ae86685b31bbb2824fac,
        0x00113a6b58edfaec0065b365f66ba8d8aa68254b8690035e8d671a17a843f0a1,
        0x0023f2d2eae8c4449bac8f268a3e62a3faace1fe1401f0efdc8b0ccfbc8fb271,
        0x00cf6603f8c61993dd2f662c719671c61727a2f4e925fb988b23d31feccd77d9,
        0x0000000000a402a84b7294671799c38dd805f6a827a3a12633fdf91a57debe1f
    ];
    let hash = compute_encrypted_log_hash(input);
    assert(hash == 0x00a0d651ac0cbc01b72430fa6a05d91738595af6e0229347b4c9968223387aeb);
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
