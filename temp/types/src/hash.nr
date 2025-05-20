use crate::{
    abis::{
        contract_class_function_leaf_preimage::ContractClassFunctionLeafPreimage,
        contract_class_log::ContractClassLog,
        function_selector::FunctionSelector,
        note_hash::ScopedNoteHash,
        nullifier::ScopedNullifier,
        private_log::{PrivateLog, PrivateLogData},
        side_effect::{OrderedValue, scoped::Scoped},
    },
    address::{AztecAddress, EthAddress},
    constants::{
        CONTRACT_CLASS_LOG_SIZE_IN_FIELDS, FUNCTION_TREE_HEIGHT, GENERATOR_INDEX__NOTE_HASH_NONCE,
        GENERATOR_INDEX__OUTER_NULLIFIER, GENERATOR_INDEX__SILOED_NOTE_HASH,
        GENERATOR_INDEX__UNIQUE_NOTE_HASH, TWO_POW_64,
    },
    merkle_tree::root::root_from_sibling_path,
    messaging::l2_to_l1_message::{L2ToL1Message, ScopedL2ToL1Message},
    poseidon2::Poseidon2Sponge,
    traits::{FromField, Hash, ToField},
    utils::{arrays::array_concat, field::{field_from_bytes, field_from_bytes_32_trunc}},
};

pub fn sha256_to_field<let N: u32>(bytes_to_hash: [u8; N]) -> Field {
    let sha256_hashed = sha256::digest(bytes_to_hash);
    let hash_in_a_field = field_from_bytes_32_trunc(sha256_hashed);

    hash_in_a_field
}

pub fn private_functions_root_from_siblings(
    selector: FunctionSelector,
    vk_hash: Field,
    function_leaf_index: Field,
    function_leaf_sibling_path: [Field; FUNCTION_TREE_HEIGHT],
) -> Field {
    let function_leaf_preimage = ContractClassFunctionLeafPreimage { selector, vk_hash };
    let function_leaf = function_leaf_preimage.hash();
    root_from_sibling_path(
        function_leaf,
        function_leaf_index,
        function_leaf_sibling_path,
    )
}

pub fn compute_note_hash_nonce(first_nullifier_in_tx: Field, note_index_in_tx: u32) -> Field {
    // Hashing the first nullifier with note index in tx is guaranteed to be unique (because all nullifiers are also
    // unique).
    poseidon2_hash_with_separator(
        [first_nullifier_in_tx, note_index_in_tx as Field],
        GENERATOR_INDEX__NOTE_HASH_NONCE,
    )
}

pub fn compute_unique_note_hash(nonce: Field, siloed_note_hash: Field) -> Field {
    let inputs = [nonce, siloed_note_hash];
    poseidon2_hash_with_separator(inputs, GENERATOR_INDEX__UNIQUE_NOTE_HASH)
}

pub fn compute_siloed_note_hash(app: AztecAddress, note_hash: Field) -> Field {
    poseidon2_hash_with_separator(
        [app.to_field(), note_hash],
        GENERATOR_INDEX__SILOED_NOTE_HASH,
    )
}

/// Computes unique note hashes from siloed note hashes
pub fn compute_unique_siloed_note_hash(
    siloed_note_hash: Field,
    first_nullifier: Field,
    note_index_in_tx: u32,
) -> Field {
    if siloed_note_hash == 0 {
        0
    } else {
        let nonce = compute_note_hash_nonce(first_nullifier, note_index_in_tx);
        compute_unique_note_hash(nonce, siloed_note_hash)
    }
}

/// Siloing in the context of Aztec refers to the process of hashing a note hash with a contract address (this way
/// the note hash is scoped to a specific contract). This is used to prevent intermingling of notes between contracts.
pub fn silo_note_hash(note_hash: ScopedNoteHash) -> Field {
    if note_hash.contract_address.is_zero() {
        0
    } else {
        compute_siloed_note_hash(note_hash.contract_address, note_hash.value())
    }
}

pub fn compute_siloed_nullifier(app: AztecAddress, nullifier: Field) -> Field {
    poseidon2_hash_with_separator(
        [app.to_field(), nullifier],
        GENERATOR_INDEX__OUTER_NULLIFIER,
    )
}

pub fn silo_nullifier(nullifier: ScopedNullifier) -> Field {
    if nullifier.contract_address.is_zero() {
        nullifier.value() // Return value instead of 0 because the first nullifier's contract address is zero.
    } else {
        compute_siloed_nullifier(nullifier.contract_address, nullifier.value())
    }
}

pub fn compute_siloed_private_log_field(contract_address: AztecAddress, field: Field) -> Field {
    poseidon2_hash([contract_address.to_field(), field])
}

pub fn silo_private_log(private_log: Scoped<PrivateLogData>) -> PrivateLog {
    if private_log.contract_address.is_zero() {
        private_log.inner.log
    } else {
        let mut fields = private_log.inner.log.fields;
        fields[0] = compute_siloed_private_log_field(private_log.contract_address, fields[0]);
        PrivateLog::new(fields, private_log.inner.log.length)
    }
}

pub fn compute_siloed_contract_class_log_field(
    contract_address: AztecAddress,
    first_field: Field,
) -> Field {
    poseidon2_hash([contract_address.to_field(), first_field])
}

pub fn silo_contract_class_log(contract_class_log: ContractClassLog) -> ContractClassLog {
    if contract_class_log.contract_address.is_zero() {
        contract_class_log
    } else {
        let mut log = contract_class_log;
        log.log.fields[0] = compute_siloed_contract_class_log_field(
            contract_class_log.contract_address,
            log.log.fields[0],
        );
        log
    }
}

pub fn compute_contract_class_log_hash(log: [Field; CONTRACT_CLASS_LOG_SIZE_IN_FIELDS]) -> Field {
    poseidon2_hash(log)
}

pub fn merkle_hash(left: Field, right: Field) -> Field {
    poseidon2_hash([left, right])
}

pub fn compute_l2_to_l1_hash(
    contract_address: AztecAddress,
    recipient: EthAddress,
    content: Field,
    rollup_version_id: Field,
    chain_id: Field,
) -> Field {
    let mut bytes: [u8; 160] = std::mem::zeroed();

    let inputs =
        [contract_address.to_field(), rollup_version_id, recipient.to_field(), chain_id, content];
    for i in 0..5 {
        // TODO are bytes be in fr.to_buffer() ?
        let item_bytes: [u8; 32] = inputs[i].to_be_bytes();
        for j in 0..32 {
            bytes[32 * i + j] = item_bytes[j];
        }
    }

    sha256_to_field(bytes)
}

pub fn silo_l2_to_l1_message(
    msg: ScopedL2ToL1Message,
    rollup_version_id: Field,
    chain_id: Field,
) -> Field {
    if msg.contract_address.is_zero() {
        0
    } else {
        compute_l2_to_l1_hash(
            msg.contract_address,
            msg.message.recipient,
            msg.message.content,
            rollup_version_id,
            chain_id,
        )
    }
}

// Computes sha256 hash of 2 input hashes.
//
// NB: This method now takes in two 31 byte fields - it assumes that any input
// is the result of a sha_to_field hash and => is truncated
//
// TODO(Jan and David): This is used for the encrypted_log hashes.
// Can we check to see if we can just use hash_to_field or pedersen_compress here?
//
pub fn accumulate_sha256(input: [Field; 2]) -> Field {
    // This is a note about the cpp code, since it takes an array of Fields
    // instead of a u128.
    // 4 Field elements when converted to bytes will usually
    // occupy 4 * 32 = 128 bytes.
    // However, this function is making the assumption that each Field
    // only occupies 128 bits.
    //
    // TODO(David): This does not seem to be getting guaranteed anywhere in the code?
    // Concatentate two fields into 32x2 = 64 bytes
    // accumulate_sha256 assumes that the inputs are pre-truncated 31 byte numbers
    let mut hash_input_flattened = [0; 64];
    for offset in 0..input.len() {
        let input_as_bytes: [u8; 32] = input[offset].to_be_bytes();
        for byte_index in 0..32 {
            hash_input_flattened[offset * 32 + byte_index] = input_as_bytes[byte_index];
        }
    }

    sha256_to_field(hash_input_flattened)
}

pub fn verification_key_hash<let N: u32>(key: [Field; N]) -> Field {
    crate::hash::poseidon2_hash(key)
}

#[inline_always]
pub fn pedersen_hash<let N: u32>(inputs: [Field; N], hash_index: u32) -> Field {
    std::hash::pedersen_hash_with_separator(inputs, hash_index)
}

pub fn poseidon2_hash<let N: u32>(inputs: [Field; N]) -> Field {
    poseidon::poseidon2::Poseidon2::hash(inputs, N)
}

#[no_predicates]
pub fn poseidon2_hash_with_separator<let N: u32, T>(inputs: [Field; N], separator: T) -> Field
where
    T: ToField,
{
    let inputs_with_separator = array_concat([separator.to_field()], inputs);
    poseidon2_hash(inputs_with_separator)
}

// Performs a fixed length hash with a subarray of the given input.
// Useful for SpongeBlob in which we aborb M things and want to check it vs a hash of M elts of an N-len array.
// Using stdlib poseidon, this will always absorb an extra 1 as a 'variable' hash, and not match spongeblob.squeeze()
// or any ts implementation. Also checks that any remaining elts not hashed are empty.
#[no_predicates]
pub fn poseidon2_hash_subarray<let N: u32>(input: [Field; N], in_len: u32) -> Field {
    let mut sponge = poseidon2_absorb_chunks(input, in_len, false);
    sponge.squeeze()
}

// NB the below is the same as poseidon::poseidon2::Poseidon2::hash(), but replacing a range check with a bit check,
// and absorbing in chunks of 3 below.
#[no_predicates]
pub fn poseidon2_cheaper_variable_hash<let N: u32>(input: [Field; N], in_len: u32) -> Field {
    let mut sponge = poseidon2_absorb_chunks(input, in_len, true);
    // In the case where the hash preimage is variable-length, we append `1` to the end of the input, to distinguish
    // from fixed-length hashes. (the combination of this additional field element + the hash IV ensures
    // fixed-length and variable-length hashes do not collide)
    if in_len != N {
        sponge.absorb(1);
    }
    sponge.squeeze()
}

// The below fn reduces gates of a conditional poseidon2 hash by approx 3x (thank you ~* Giant Brain Dev @IlyasRidhuan *~ for the idea)
// Why? Because when we call stdlib poseidon, we call absorb for each item. When absorbing is conditional, it seems the compiler does not know
// what cache_size will be when calling absorb, so it assigns the permutation gates for /each i/ rather than /every 3rd i/, which is actually required.
// The below code forces the compiler to:
//  - absorb normally up to 2 times to set cache_size to 1
//  - absorb in chunks of 3 to ensure perm. only happens every 3rd absorb
//  - absorb normally up to 2 times to add any remaining values to the hash
// In fixed len hashes, the compiler is able to tell that it will only need to perform the permutation every 3 absorbs.
// NB: it also replaces unnecessary range checks (i < thing) with a bit check (&= i != thing), which alone reduces the gates of a var. hash by half.

#[no_predicates]
fn poseidon2_absorb_chunks<let N: u32>(
    input: [Field; N],
    in_len: u32,
    variable: bool,
) -> Poseidon2Sponge {
    let iv: Field = (in_len as Field) * TWO_POW_64;
    let mut sponge = Poseidon2Sponge::new(iv);
    // Even though shift is always 1 here, if we input in_len = 0 we get an underflow
    // since we cannot isolate computation branches. The below is just to avoid that.
    let shift = if in_len == 0 { 0 } else { 1 };
    if in_len != 0 {
        // cache_size = 0, init absorb
        sponge.cache[0] = input[0];
        sponge.cache_size = 1;
        // shift = num elts already added to make cache_size 1 = 1 for a fresh sponge
        // M = max_chunks = (N - 1 - (N - 1) % 3) / 3: (must be written as a fn of N to compile)
        // max_remainder = (N - 1) % 3;
        // max_chunks = (N - 1 - max_remainder) / 3;
        sponge = poseidon2_absorb_chunks_loop::<N, (N - 1 - (N - 1) % 3) / 3>(
            sponge,
            input,
            in_len,
            variable,
            shift,
        );
    }
    sponge
}

// NB: If it's not required to check that the non-absorbed elts of 'input' are 0s, set skip_0_check=true
#[no_predicates]
pub fn poseidon2_absorb_chunks_existing_sponge<let N: u32>(
    in_sponge: Poseidon2Sponge,
    input: [Field; N],
    in_len: u32,
    skip_0_check: bool,
) -> Poseidon2Sponge {
    let mut sponge = in_sponge;
    // 'shift' is to account for already added inputs
    let mut shift = 0;
    // 'stop' is to avoid an underflow when inputting in_len = 0
    let mut stop = false;
    for i in 0..3 {
        if shift == in_len {
            stop = true;
        }
        if (sponge.cache_size != 1) & (!stop) {
            sponge.absorb(input[i]);
            shift += 1;
        }
    }
    sponge = if stop {
        sponge
    } else {
        // max_chunks = (N - (N % 3)) / 3;
        poseidon2_absorb_chunks_loop::<N, (N - (N % 3)) / 3>(
            sponge,
            input,
            in_len,
            skip_0_check,
            shift,
        )
    };
    sponge
}

// The below is the loop to absorb elts into a poseidon sponge in chunks of 3
// shift - the num of elts already absorbed to ensure the sponge's cache_size = 1
// M - the max number of chunks required to absorb N things (must be comptime to compile)
// NB: The 0 checks ('Found non-zero field...') are messy, but having a separate loop over N to check
// for 0s costs 3N gates. Current approach is approx 2N gates.
#[no_predicates]
fn poseidon2_absorb_chunks_loop<let N: u32, let M: u32>(
    in_sponge: Poseidon2Sponge,
    input: [Field; N],
    in_len: u32,
    variable: bool,
    shift: u32,
) -> Poseidon2Sponge {
    assert(in_len <= N, "Given in_len to absorb is larger than the input array len");
    // When we have an existing sponge, we may have a shift of 0, and the final 'k+2' below = N
    // The below avoids an overflow
    let skip_last = 3 * M == N;
    // Writing in_sponge: &mut does not compile
    let mut sponge = in_sponge;
    let mut should_add = true;
    // The num of things left over after absorbing in 3s
    let remainder = (in_len - shift) % 3;
    // The num of chunks of 3 to absorb (maximum M)
    let chunks = (in_len - shift - remainder) / 3;
    for i in 0..M {
        // Now we loop through cache size = 1 -> 3
        should_add &= i != chunks;
        // This is the index at the start of the chunk (for readability)
        let k = 3 * i + shift;
        if should_add {
            // cache_size = 1, 2 => just assign
            sponge.cache[1] = input[k];
            sponge.cache[2] = input[k + 1];
            // cache_size = 3 => duplex + perm
            for j in 0..3 {
                sponge.state[j] += sponge.cache[j];
            }
            sponge.state = std::hash::poseidon2_permutation(sponge.state, 4);
            sponge.cache[0] = input[k + 2];
            // cache_size is now 1 again, repeat loop
        } else if (!variable) & (i != chunks) {
            // if we are hashing a fixed len array which is a subarray, we check the remaining elts are 0
            // NB: we don't check at i == chunks, because that chunk contains elts to be absorbed or checked below
            let last_0 = if (i == M - 1) & (skip_last) {
                0
            } else {
                input[k + 2]
            };
            let all_0 = (input[k] == 0) & (input[k + 1] == 0) & (last_0 == 0);
            assert(all_0, "Found non-zero field after breakpoint");
        }
    }
    // we have 'remainder' num of items left to absorb
    should_add = true;
    // below is to avoid overflows (i.e. if inlen is close to N)
    let mut should_check = !variable;
    for i in 0..3 {
        should_add &= i != remainder;
        should_check &= in_len - remainder + i != N;
        if should_add {
            // we want to absorb the final 'remainder' items
            sponge.absorb(input[in_len - remainder + i]);
        } else if should_check {
            assert(input[in_len - remainder + i] == 0, "Found non-zero field after breakpoint");
        }
    }
    sponge
}

pub fn poseidon2_hash_with_separator_slice<T>(inputs: [Field], separator: T) -> Field
where
    T: ToField,
{
    let in_len = inputs.len() + 1;
    let iv: Field = (in_len as Field) * TWO_POW_64;
    let mut sponge = Poseidon2Sponge::new(iv);
    sponge.absorb(separator.to_field());

    for i in 0..inputs.len() {
        sponge.absorb(inputs[i]);
    }

    sponge.squeeze()
}

#[no_predicates]
pub fn poseidon2_hash_bytes<let N: u32>(inputs: [u8; N]) -> Field {
    let mut fields = [0; (N + 30) / 31];
    let mut field_index = 0;
    let mut current_field = [0; 31];
    for i in 0..inputs.len() {
        let index = i % 31;
        current_field[index] = inputs[i];
        if index == 30 {
            fields[field_index] = field_from_bytes(current_field, false);
            current_field = [0; 31];
            field_index += 1;
        }
    }
    if field_index != fields.len() {
        fields[field_index] = field_from_bytes(current_field, false);
    }
    poseidon2_hash(fields)
}

#[test]
fn poseidon_chunks_matches_fixed() {
    let in_len = 501;
    let mut input: [Field; 4096] = [0; 4096];
    let mut fixed_input = [3; 501];
    assert(in_len == fixed_input.len()); // sanity check
    for i in 0..in_len {
        input[i] = 3;
    }
    let sub_chunk_hash = poseidon2_hash_subarray(input, in_len);
    let fixed_len_hash = poseidon::poseidon2::Poseidon2::hash(fixed_input, fixed_input.len());
    assert(sub_chunk_hash == fixed_len_hash);
}

#[test]
fn poseidon_chunks_matches_variable() {
    let in_len = 501;
    let mut input: [Field; 4096] = [0; 4096];
    for i in 0..in_len {
        input[i] = 3;
    }
    let variable_chunk_hash = poseidon2_cheaper_variable_hash(input, in_len);
    let variable_len_hash = poseidon::poseidon2::Poseidon2::hash(input, in_len);
    assert(variable_chunk_hash == variable_len_hash);
}

#[test]
fn existing_sponge_poseidon_chunks_matches_fixed() {
    let in_len = 501;
    let mut input: [Field; 4096] = [0; 4096];
    let mut fixed_input = [3; 501];
    assert(in_len == fixed_input.len()); // sanity check
    for i in 0..in_len {
        input[i] = 3;
    }
    // absorb 250 of the 501 things
    let empty_sponge = Poseidon2Sponge::new((in_len as Field) * TWO_POW_64);
    let first_sponge = poseidon2_absorb_chunks_existing_sponge(empty_sponge, input, 250, true);
    // now absorb the final 251 (since they are all 3s, im being lazy and not making a new array)
    let mut final_sponge = poseidon2_absorb_chunks_existing_sponge(first_sponge, input, 251, true);
    let fixed_len_hash = Poseidon2Sponge::hash(fixed_input, fixed_input.len());
    assert(final_sponge.squeeze() == fixed_len_hash);
}

#[test]
fn poseidon_chunks_empty_inputs() {
    let in_len = 0;
    let mut input: [Field; 4096] = [0; 4096];
    let mut constructed_empty_sponge = poseidon2_absorb_chunks(input, in_len, true);
    let mut first_sponge =
        poseidon2_absorb_chunks_existing_sponge(constructed_empty_sponge, input, in_len, true);
    assert(first_sponge.squeeze() == constructed_empty_sponge.squeeze());
}

#[test]
fn smoke_sha256_to_field() {
    let full_buffer = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
        48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
        71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93,
        94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112,
        113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130,
        131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148,
        149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159,
    ];
    let result = sha256_to_field(full_buffer);

    assert(result == 0x448ebbc9e1a31220a2f3830c18eef61b9bd070e5084b7fa2a359fe729184c7);

    // to show correctness of the current ver (truncate one byte) vs old ver (mod full bytes):
    let result_bytes = sha256::digest(full_buffer);
    let truncated_field = crate::utils::field::field_from_bytes_32_trunc(result_bytes);
    assert(truncated_field == result);
    let mod_res = result + (result_bytes[31] as Field);
    assert(mod_res == 0x448ebbc9e1a31220a2f3830c18eef61b9bd070e5084b7fa2a359fe729184e0);
}

#[test]
fn compute_l2_l1_hash() {
    // All zeroes
    let hash_result =
        compute_l2_to_l1_hash(AztecAddress::from_field(0), EthAddress::zero(), 0, 0, 0);
    assert(hash_result == 0xb393978842a0fa3d3e1470196f098f473f9678e72463cb65ec4ab5581856c2);

    // Non-zero case
    let hash_result = compute_l2_to_l1_hash(
        AztecAddress::from_field(1),
        EthAddress::from_field(3),
        5,
        2,
        4,
    );
    assert(hash_result == 0x3f88c1044a05e5340ed20466276500f6d45ca5603913b9091e957161734e16);
}

#[test]
fn silo_l2_to_l1_message_matches_typescript() {
    let version = 4;
    let chainId = 5;

    let hash = silo_l2_to_l1_message(
        ScopedL2ToL1Message {
            message: L2ToL1Message { recipient: EthAddress::from_field(1), content: 2, counter: 0 },
            contract_address: AztecAddress::from_field(3),
        },
        version,
        chainId,
    );

    // The following value was generated by `l2_to_l1_message.test.ts`
    let hash_from_typescript = 0x00c6155d69febb9d5039b374dd4f77bf57b7c881709aa524a18acaa0bd57476a;

    assert_eq(hash, hash_from_typescript);
}
