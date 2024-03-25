#pragma once

#include "barretenberg/crypto/merkle_tree/hash_path.hpp"
#include "barretenberg/stdlib/commitment/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/encryption/schnorr/schnorr.hpp"
#include "barretenberg/stdlib/primitives/bool/bool.hpp"
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib/primitives/group/cycle_group.hpp"
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"

namespace bb::join_split_example {

using Builder = UltraCircuitBuilder;

using witness_ct = stdlib::witness_t<Builder>;
using public_witness_ct = stdlib::public_witness_t<Builder>;
using bool_ct = stdlib::bool_t<Builder>;
using byte_array_ct = stdlib::byte_array<Builder>;
using field_ct = stdlib::field_t<Builder>;
using suint_ct = stdlib::safe_uint_t<Builder>;
using uint32_ct = stdlib::uint32<Builder>;
using group_ct = stdlib::cycle_group<Builder>;
using pedersen_commitment = stdlib::pedersen_commitment<Builder>;
using pedersen_hash = stdlib::pedersen_hash<Builder>;
using bn254 = stdlib::bn254<Builder>;
using hash_path_ct = crypto::merkle_tree::hash_path<Builder>;
using schnorr_signature_bits = stdlib::schnorr_signature_bits<Builder>;

} // namespace bb::join_split_example
