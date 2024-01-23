#pragma once
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp"
#include "barretenberg/plonk/proof_system/prover/prover.hpp"
#include "barretenberg/plonk/proof_system/types/prover_settings.hpp"
#include "barretenberg/stdlib/commitment/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/encryption/schnorr/schnorr.hpp"
#include "barretenberg/stdlib/merkle_tree/hash_path.hpp"
#include "barretenberg/stdlib/primitives/bigfield/bigfield.hpp"
#include "barretenberg/stdlib/primitives/biggroup/biggroup.hpp"
#include "barretenberg/stdlib/primitives/bit_array/bit_array.hpp"
#include "barretenberg/stdlib/primitives/bool/bool.hpp"
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib/primitives/curves/secp256k1.hpp"
#include "barretenberg/stdlib/primitives/memory/rom_table.hpp"
#include "barretenberg/stdlib/primitives/packed_byte_array/packed_byte_array.hpp"
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
#include "barretenberg/stdlib/recursion/verifier/program_settings.hpp"

namespace bb::stdlib::types {

using namespace bb;

using Builder = bb::UltraCircuitBuilder;
using Composer = plonk::UltraComposer;

// TODO(Cody): These might be wrong depending on desired F-S hash.
using Prover = plonk::UltraProver;
using Verifier = plonk::UltraVerifier;

using settings = plonk::ultra_settings;

using kate_commitment_scheme = plonk::KateCommitmentScheme<settings>;
using witness_ct = stdlib::witness_t<Builder>;
using public_witness_ct = stdlib::public_witness_t<Builder>;
using bool_ct = stdlib::bool_t<Builder>;
using byte_array_ct = stdlib::byte_array<Builder>;
using packed_byte_array_ct = stdlib::packed_byte_array<Builder>;
using field_ct = stdlib::field_t<Builder>;
using suint_ct = stdlib::safe_uint_t<Builder>;
using uint8_ct = stdlib::uint8<Builder>;
using uint16_ct = stdlib::uint16<Builder>;
using uint32_ct = stdlib::uint32<Builder>;
using uint64_ct = stdlib::uint64<Builder>;
using bit_array_ct = stdlib::bit_array<Builder>;
using fq_ct = stdlib::bigfield<Builder, bb::Bn254FqParams>;
using biggroup_ct = stdlib::element<Builder, fq_ct, field_ct, bb::g1>;
using pedersen_commitment = stdlib::pedersen_commitment<Builder>;
using cycle_group_ct = stdlib::cycle_group<Builder>;
using bn254 = stdlib::bn254<Builder>;
using secp256k1_ct = stdlib::secp256k1<Builder>;

namespace merkle_tree {
using namespace stdlib::merkle_tree;
using hash_path = stdlib::merkle_tree::hash_path<Builder>;
} // namespace merkle_tree

using schnorr_signature_bits = stdlib::schnorr_signature_bits<Builder>;

// Ultra-composer specific types
using rom_table_ct = stdlib::rom_table<plonk::UltraComposer>;

using recursive_inner_verifier_settings = recursion::recursive_ultra_verifier_settings<bn254>;

} // namespace bb::stdlib::types
