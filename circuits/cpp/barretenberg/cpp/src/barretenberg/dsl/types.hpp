#pragma once
#include "barretenberg/plonk/composer/ultra_composer.hpp"

#include "barretenberg/plonk/proof_system/prover/prover.hpp"
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
#include "barretenberg/stdlib/primitives/memory/ram_table.hpp"
namespace acir_format {

using Composer = plonk::UltraComposer;

using Prover = std::conditional_t<
    std::same_as<Composer, plonk::UltraComposer>,
    plonk::UltraWithKeccakProver,
    std::conditional_t<std::same_as<Composer, plonk::TurboComposer>, plonk::TurboProver, plonk::Prover>>;

using Verifier = std::conditional_t<
    std::same_as<Composer, plonk::UltraComposer>,
    plonk::UltraWithKeccakVerifier,
    std::conditional_t<std::same_as<Composer, plonk::TurboComposer>, plonk::TurboVerifier, plonk::Verifier>>;

using witness_ct = proof_system::plonk::stdlib::witness_t<Composer>;
using public_witness_ct = proof_system::plonk::stdlib::public_witness_t<Composer>;
using bool_ct = proof_system::plonk::stdlib::bool_t<Composer>;
using byte_array_ct = proof_system::plonk::stdlib::byte_array<Composer>;
using packed_byte_array_ct = proof_system::plonk::stdlib::packed_byte_array<Composer>;
using field_ct = proof_system::plonk::stdlib::field_t<Composer>;
using suint_ct = proof_system::plonk::stdlib::safe_uint_t<Composer>;
using uint8_ct = proof_system::plonk::stdlib::uint8<Composer>;
using uint16_ct = proof_system::plonk::stdlib::uint16<Composer>;
using uint32_ct = proof_system::plonk::stdlib::uint32<Composer>;
using uint64_ct = proof_system::plonk::stdlib::uint64<Composer>;
using bit_array_ct = proof_system::plonk::stdlib::bit_array<Composer>;
using fq_ct = proof_system::plonk::stdlib::bigfield<Composer, barretenberg::Bn254FqParams>;
using biggroup_ct = proof_system::plonk::stdlib::element<Composer, fq_ct, field_ct, barretenberg::g1>;
using point_ct = proof_system::plonk::stdlib::point<Composer>;
using pedersen_commitment = proof_system::plonk::stdlib::pedersen_commitment<Composer>;
using group_ct = proof_system::plonk::stdlib::group<Composer>;
using bn254 = proof_system::plonk::stdlib::bn254<Composer>;
using secp256k1_ct = proof_system::plonk::stdlib::secp256k1<Composer>;

using hash_path_ct = proof_system::plonk::stdlib::merkle_tree::hash_path<Composer>;

using schnorr_signature_bits_ct = proof_system::plonk::stdlib::schnorr::signature_bits<Composer>;

// Ultra-composer specific typesv
using rom_table_ct = proof_system::plonk::stdlib::rom_table<plonk::UltraComposer>;
using ram_table_ct = proof_system::plonk::stdlib::ram_table<plonk::UltraComposer>;

} // namespace acir_format
