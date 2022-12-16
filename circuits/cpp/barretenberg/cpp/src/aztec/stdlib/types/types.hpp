#pragma once
#include <plonk/proof_system/constants.hpp>
#include <plonk/composer/standard_composer.hpp>
#include <plonk/composer/turbo_composer.hpp>
#include <plonk/composer/ultra_composer.hpp>
#include <stdlib/primitives/bigfield/bigfield.hpp>
#include <stdlib/primitives/biggroup/biggroup.hpp>
#include <stdlib/primitives/bit_array/bit_array.hpp>
#include <stdlib/primitives/bool/bool.hpp>
#include <stdlib/primitives/packed_byte_array/packed_byte_array.hpp>
#include <stdlib/primitives/byte_array/byte_array.hpp>
#include <stdlib/primitives/uint/uint.hpp>
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/primitives/bigfield/bigfield.hpp>
#include <stdlib/primitives/biggroup/biggroup.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/merkle_tree/hash_path.hpp>
#include <stdlib/encryption/schnorr/schnorr.hpp>
#include <stdlib/primitives/curves/bn254.hpp>
#include <stdlib/primitives/curves/secp256k1.hpp>
#include <stdlib/primitives/memory/rom_table.hpp>
#include <stdlib/recursion/verifier/program_settings.hpp>

namespace plonk::stdlib::types {

using namespace plonk;
static constexpr size_t SYSTEM_COMPOSER = waffle::SYSTEM_COMPOSER;

typedef std::conditional_t<
    SYSTEM_COMPOSER == waffle::STANDARD,
    waffle::StandardComposer,
    std::conditional_t<SYSTEM_COMPOSER == waffle::TURBO, waffle::TurboComposer, waffle::UltraComposer>>
    Composer;

typedef std::conditional_t<
    SYSTEM_COMPOSER == waffle::STANDARD,
    waffle::Prover,
    std::conditional_t<SYSTEM_COMPOSER == waffle::TURBO, waffle::TurboProver, waffle::UltraProver>>
    Prover;

typedef std::conditional_t<
    SYSTEM_COMPOSER == waffle::STANDARD,
    waffle::Verifier,
    std::conditional_t<SYSTEM_COMPOSER == waffle::TURBO, waffle::TurboVerifier, waffle::UltraVerifier>>
    Verifier;

typedef std::conditional_t<
    SYSTEM_COMPOSER == waffle::STANDARD,
    waffle::UnrolledProver,
    std::conditional_t<SYSTEM_COMPOSER == waffle::TURBO, waffle::UnrolledTurboProver, waffle::UnrolledUltraProver>>
    UnrolledProver;

typedef std::conditional_t<
    SYSTEM_COMPOSER == waffle::STANDARD,
    waffle::UnrolledVerifier,
    std::conditional_t<SYSTEM_COMPOSER == waffle::TURBO, waffle::UnrolledTurboVerifier, waffle::UnrolledUltraVerifier>>
    UnrolledVerifier;

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
typedef stdlib::pedersen<Composer> pedersen;
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
typedef stdlib::rom_table<waffle::UltraComposer> rom_table_ct;

typedef std::conditional_t<SYSTEM_COMPOSER == waffle::TURBO,
                           recursion::recursive_turbo_verifier_settings<bn254>,
                           recursion::recursive_ultra_verifier_settings<bn254>>
    recursive_inner_verifier_settings;

} // namespace plonk::stdlib::types