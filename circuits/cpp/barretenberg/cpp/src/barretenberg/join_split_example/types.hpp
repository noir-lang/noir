#pragma once

#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/plonk/composer/turbo_composer.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"

#include "barretenberg/plonk/proof_system/prover/prover.hpp"
#include "barretenberg/stdlib/primitives/bool/bool.hpp"
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
#include "barretenberg/stdlib/commitment/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/commitment/pedersen/pedersen_plookup.hpp"
#include "barretenberg/stdlib/merkle_tree/hash_path.hpp"
#include "barretenberg/stdlib/encryption/schnorr/schnorr.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"

namespace join_split_example {

using Composer = plonk::UltraComposer;

using Prover = std::conditional_t<
    std::same_as<Composer, plonk::UltraComposer>,
    plonk::UltraProver,
    std::conditional_t<std::same_as<Composer, plonk::TurboComposer>, plonk::TurboProver, plonk::Prover>>;

using Verifier = std::conditional_t<
    std::same_as<Composer, plonk::UltraComposer>,
    plonk::UltraVerifier,
    std::conditional_t<std::same_as<Composer, plonk::TurboComposer>, plonk::TurboVerifier, plonk::Verifier>>;

using witness_ct = proof_system::plonk::stdlib::witness_t<Composer>;
using public_witness_ct = proof_system::plonk::stdlib::public_witness_t<Composer>;
using bool_ct = proof_system::plonk::stdlib::bool_t<Composer>;
using byte_array_ct = proof_system::plonk::stdlib::byte_array<Composer>;
using field_ct = proof_system::plonk::stdlib::field_t<Composer>;
using suint_ct = proof_system::plonk::stdlib::safe_uint_t<Composer>;
using uint32_ct = proof_system::plonk::stdlib::uint32<Composer>;
using point_ct = proof_system::plonk::stdlib::point<Composer>;
using pedersen_commitment = proof_system::plonk::stdlib::pedersen_commitment<Composer>;
using group_ct = proof_system::plonk::stdlib::group<Composer>;
using bn254 = proof_system::plonk::stdlib::bn254<Composer>;

using hash_path_ct = proof_system::plonk::stdlib::merkle_tree::hash_path<Composer>;

namespace schnorr {
using signature_bits = proof_system::plonk::stdlib::schnorr::signature_bits<Composer>;
} // namespace schnorr

} // namespace join_split_example