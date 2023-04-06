#pragma once
#include "barretenberg/plonk/proof_system/constants.hpp"
#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/plonk/composer/turbo_composer.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/stdlib/primitives/bigfield/bigfield.hpp"
#include "barretenberg/stdlib/primitives/biggroup/biggroup.hpp"
#include "barretenberg/stdlib/primitives/bit_array/bit_array.hpp"
#include "barretenberg/stdlib/primitives/bool/bool.hpp"
#include "barretenberg/stdlib/primitives/packed_byte_array/packed_byte_array.hpp"
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
#include "barretenberg/stdlib/primitives/bigfield/bigfield.hpp"
#include "barretenberg/stdlib/primitives/biggroup/biggroup.hpp"
#include "barretenberg/stdlib/commitment/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/commitment/pedersen/pedersen_plookup.hpp"
#include "barretenberg/stdlib/merkle_tree/hash_path.hpp"
#include "barretenberg/stdlib/encryption/schnorr/schnorr.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib/primitives/curves/secp256k1.hpp"
#include "barretenberg/stdlib/primitives/memory/rom_table.hpp"
#include "barretenberg/stdlib/recursion/verifier/program_settings.hpp"
#include "barretenberg/stdlib/primitives/memory/ram_table.hpp"
#include "barretenberg/stdlib/primitives/memory/rom_table.hpp"
#include "barretenberg/stdlib/primitives/memory/dynamic_array.hpp"

namespace proof_system::plonk::stdlib::types {

using namespace proof_system::plonk;
static constexpr size_t SYSTEM_COMPOSER = proof_system::plonk::SYSTEM_COMPOSER;

typedef std::conditional_t<
    SYSTEM_COMPOSER == proof_system::STANDARD,
    plonk::StandardComposer,
    std::conditional_t<SYSTEM_COMPOSER == proof_system::TURBO, plonk::TurboComposer, plonk::UltraComposer>>
    Composer;

typedef std::conditional_t<
    SYSTEM_COMPOSER == proof_system::STANDARD,
    plonk::Prover,
    std::conditional_t<SYSTEM_COMPOSER == proof_system::TURBO, plonk::TurboProver, plonk::UltraProver>>
    Prover;

typedef std::conditional_t<
    SYSTEM_COMPOSER == proof_system::STANDARD,
    plonk::Verifier,
    std::conditional_t<SYSTEM_COMPOSER == proof_system::TURBO, plonk::TurboVerifier, plonk::UltraVerifier>>
    Verifier;

typedef std::conditional_t<
    SYSTEM_COMPOSER == proof_system::STANDARD,
    plonk::Prover,
    std::conditional_t<SYSTEM_COMPOSER == proof_system::TURBO, plonk::TurboProver, plonk::UltraProver>>
    Prover;

typedef std::conditional_t<
    SYSTEM_COMPOSER == proof_system::STANDARD,
    plonk::Verifier,
    std::conditional_t<SYSTEM_COMPOSER == proof_system::TURBO, plonk::TurboVerifier, plonk::UltraVerifier>>
    Verifier;

typedef stdlib::witness_t<Composer> witness_ct;
typedef stdlib::public_witness_t<Composer> public_witness_ct;
typedef stdlib::bool_t<Composer> bool_ct;
typedef stdlib::byte_array<Composer> byte_array_ct;
typedef stdlib::packed_byte_array<Composer> packed_byte_array_ct;
typedef stdlib::field_t<Composer> field_ct;
typedef stdlib::safe_uint_t<Composer> suint_ct;
typedef stdlib::uint8<Composer> uint8_ct;
typedef stdlib::uint16<Composer> uint16_ct;
typedef stdlib::uint32<Composer> uint32_ct;
typedef stdlib::uint64<Composer> uint64_ct;
typedef stdlib::bit_array<Composer> bit_array_ct;
typedef stdlib::bigfield<Composer, barretenberg::Bn254FqParams> fq_ct;
typedef stdlib::element<Composer, fq_ct, field_ct, barretenberg::g1> biggroup_ct;
typedef stdlib::point<Composer> point_ct;
typedef stdlib::pedersen_commitment<Composer> pedersen_commitment;
typedef stdlib::group<Composer> group_ct;
typedef stdlib::bn254<Composer> bn254;
typedef stdlib::secp256k1<Composer> secp256k1_ct;

namespace merkle_tree {
using namespace stdlib::merkle_tree;
typedef stdlib::merkle_tree::hash_path<Composer> hash_path;
} // namespace merkle_tree

namespace schnorr {
typedef stdlib::schnorr::signature_bits<Composer> signature_bits;
} // namespace schnorr

// Ultra-composer specific types
typedef stdlib::rom_table<plonk::UltraComposer> rom_table_ct;

typedef std::conditional_t<SYSTEM_COMPOSER == proof_system::TURBO,
                           recursion::recursive_turbo_verifier_settings<bn254>,
                           recursion::recursive_ultra_verifier_settings<bn254>>
    recursive_inner_verifier_settings;

} // namespace proof_system::plonk::stdlib::types
