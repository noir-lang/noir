/* eslint-disable camelcase */
export const acvmInfoJson = {
  language: {
    name: 'PLONK-CSAT',
    width: 3,
  },
  opcodes_supported: ['arithmetic', 'directive', 'brillig', 'memory_init', 'memory_op'],
  black_box_functions_supported: [
    'and',
    'xor',
    'range',
    'sha256',
    'blake2s',
    'keccak256',
    'schnorr_verify',
    'pedersen',
    'pedersen_hash',
    'ecdsa_secp256k1',
    'ecdsa_secp256r1',
    'fixed_base_scalar_mul',
    'recursive_aggregation',
  ],
};
