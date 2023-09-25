#pragma once

#include "barretenberg/honk/composer/ultra_composer.hpp"
#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"

#include "barretenberg/plonk/proof_system/prover/prover.hpp"
#include "barretenberg/stdlib/commitment/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/commitment/pedersen/pedersen_plookup.hpp"
#include "barretenberg/stdlib/encryption/schnorr/schnorr.hpp"
#include "barretenberg/stdlib/merkle_tree/hash_path.hpp"
#include "barretenberg/stdlib/primitives/bool/bool.hpp"
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"

namespace join_split_example {

using Builder = proof_system::UltraCircuitBuilder;
using Composer = plonk::UltraComposer;

using Prover = std::conditional_t<std::same_as<Composer, plonk::UltraComposer>, plonk::UltraProver, plonk::Prover>;

using Verifier =
    std::conditional_t<std::same_as<Composer, plonk::UltraComposer>, plonk::UltraVerifier, plonk::Verifier>;

using witness_ct = proof_system::plonk::stdlib::witness_t<Builder>;
using public_witness_ct = proof_system::plonk::stdlib::public_witness_t<Builder>;
using bool_ct = proof_system::plonk::stdlib::bool_t<Builder>;
using byte_array_ct = proof_system::plonk::stdlib::byte_array<Builder>;
using field_ct = proof_system::plonk::stdlib::field_t<Builder>;
using suint_ct = proof_system::plonk::stdlib::safe_uint_t<Builder>;
using uint32_ct = proof_system::plonk::stdlib::uint32<Builder>;
using point_ct = proof_system::plonk::stdlib::point<Builder>;
using pedersen_commitment = proof_system::plonk::stdlib::pedersen_commitment<Builder>;
using group_ct = proof_system::plonk::stdlib::group<Builder>;
using bn254 = proof_system::plonk::stdlib::bn254<Builder>;

using hash_path_ct = proof_system::plonk::stdlib::merkle_tree::hash_path<Builder>;

namespace schnorr {
using signature_bits = proof_system::plonk::stdlib::schnorr::signature_bits<Builder>;
} // namespace schnorr

} // namespace join_split_example