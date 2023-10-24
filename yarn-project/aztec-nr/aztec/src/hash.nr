use dep::std::hash::{pedersen_with_separator, sha256};
use crate::constants_gen::{
  GENERATOR_INDEX__SIGNATURE_PAYLOAD,
  GENERATOR_INDEX__L1_TO_L2_MESSAGE_SECRET,
};

pub fn sha256_to_field<N>(bytes_to_hash: [u8; N]) -> Field {
  let sha256_hashed = sha256(bytes_to_hash);

  // Convert it to a field element
  let mut v = 1;
  let mut high = 0 as Field;
  let mut low = 0 as Field;

  for i in 0..16 {
      high = high + (sha256_hashed[15 - i] as Field) * v;
      low = low + (sha256_hashed[16 + 15 - i] as Field) * v;
      v = v * 256;
  }

  // Abuse that a % p + b % p = (a + b) % p and that low < p
  let hash_in_a_field = low + high * v;

  hash_in_a_field
}

pub fn compute_secret_hash(secret: Field) -> Field {
  // TODO(#1205) This is probably not the right index to use
  pedersen_hash([secret], GENERATOR_INDEX__L1_TO_L2_MESSAGE_SECRET)
}

#[oracle(pedersenHash)]
fn pedersen_hash_oracle<N>(_inputs: [Field; N], _hash_index: u32) -> Field {}

unconstrained fn pedersen_hash_internal<N>(inputs: [Field; N], hash_index: u32) -> Field {
    pedersen_hash_oracle(inputs, hash_index)
}

// TODO(Kev): We can move this into the noir std-lib 
// as an acir opcode, once we have the pedersen PR merged.
pub fn pedersen_hash<N>(inputs: [Field; N], hash_index: u32) -> Field {
  // Temporary hack. This is not right. Kev forced me to do this. Blame him.
  pedersen_hash_internal(inputs, hash_index)
  // let input_size = inputs.len();
  // let mut modified_inputs = [];
  // modified_inputs = modified_inputs.push_back(input_size as Field);
  // for i in 0..N {
  //   modified_inputs = modified_inputs.push_back(inputs[i]);
  // }
  // pedersen_with_separator(modified_inputs, hash_index)[0]
}