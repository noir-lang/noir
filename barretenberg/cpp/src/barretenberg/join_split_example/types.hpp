#pragma once

#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"

#include "barretenberg/plonk/proof_system/prover/prover.hpp"
#include "barretenberg/stdlib/commitment/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/encryption/schnorr/schnorr.hpp"
#include "barretenberg/stdlib/merkle_tree/hash_path.hpp"
#include "barretenberg/stdlib/primitives/bool/bool.hpp"
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib/primitives/group/cycle_group.hpp"
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"

namespace join_split_example {

using Builder = bb::UltraCircuitBuilder;
using Composer = plonk::UltraComposer;

using Prover = std::conditional_t<std::same_as<Composer, plonk::UltraComposer>, plonk::UltraProver, plonk::Prover>;

using Verifier =
    std::conditional_t<std::same_as<Composer, plonk::UltraComposer>, plonk::UltraVerifier, plonk::Verifier>;

using witness_ct = bb::stdlib::witness_t<Builder>;
using public_witness_ct = bb::stdlib::public_witness_t<Builder>;
using bool_ct = bb::stdlib::bool_t<Builder>;
using byte_array_ct = bb::stdlib::byte_array<Builder>;
using field_ct = bb::stdlib::field_t<Builder>;
using suint_ct = bb::stdlib::safe_uint_t<Builder>;
using uint32_ct = bb::stdlib::uint32<Builder>;
using group_ct = bb::stdlib::cycle_group<Builder>;
using pedersen_commitment = bb::stdlib::pedersen_commitment<Builder>;
using pedersen_hash = bb::stdlib::pedersen_hash<Builder>;
using bn254 = bb::stdlib::bn254<Builder>;

using hash_path_ct = bb::stdlib::merkle_tree::hash_path<Builder>;

namespace schnorr {
using signature_bits = bb::stdlib::schnorr::signature_bits<Builder>;
} // namespace schnorr

} // namespace join_split_example