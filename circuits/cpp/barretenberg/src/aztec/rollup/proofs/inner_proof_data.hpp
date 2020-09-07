#pragma once
#include <numeric/uint256/uint256.hpp>
#include <numeric/uint128/uint128.hpp>
#include <ecc/curves/bn254/fr.hpp>
#include <array>

namespace rollup {
namespace proofs {

namespace InnerProofFields {
enum {
    PROOF_ID = 0,
    PUBLIC_INPUT = 1,
    PUBLIC_OUTPUT = 2,
    NEW_NOTE1_X = 3,
    NEW_NOTE1_Y = 4,
    NEW_NOTE2_X = 5,
    NEW_NOTE2_Y = 6,
    NULLIFIER1 = 7,
    NULLIFIER2 = 8,
    INPUT_OWNER = 9,
    OUTPUT_OWNER = 10,
    MERKLE_ROOT = 11,
    ACCOUNT_NULLIFIER = 12,
};
const size_t NUM_PUBLISHED = 11;
} // namespace InnerProofFields

namespace InnerProofOffsets {
enum {
    PROOF_ID = InnerProofFields::PROOF_ID * 32,
    PUBLIC_INPUT = InnerProofFields::PUBLIC_INPUT * 32,
    PUBLIC_OUTPUT = InnerProofFields::PUBLIC_OUTPUT * 32,
    NEW_NOTE1_X = InnerProofFields::NEW_NOTE1_X * 32,
    NEW_NOTE1_Y = InnerProofFields::NEW_NOTE1_Y * 32,
    NEW_NOTE2_X = InnerProofFields::NEW_NOTE2_X * 32,
    NEW_NOTE2_Y = InnerProofFields::NEW_NOTE2_Y * 32,
    NULLIFIER1 = InnerProofFields::NULLIFIER1 * 32,
    NULLIFIER2 = InnerProofFields::NULLIFIER2 * 32,
    INPUT_OWNER = InnerProofFields::INPUT_OWNER * 32,
    OUTPUT_OWNER = InnerProofFields::OUTPUT_OWNER * 32,
    MERKLE_ROOT = InnerProofFields::MERKLE_ROOT * 32,
    ACCOUNT_NULLIFIER = InnerProofFields::ACCOUNT_NULLIFIER * 32,
};
}

struct inner_proof_data {
    uint256_t proof_id;
    uint256_t public_input;
    uint256_t public_output;
    std::array<uint8_t, 64> new_note1;
    std::array<uint8_t, 64> new_note2;
    uint128_t nullifier1;
    uint128_t nullifier2;
    barretenberg::fr input_owner;
    barretenberg::fr output_owner;

    barretenberg::fr merkle_root;
    uint128_t account_nullifier;

    inner_proof_data(std::vector<uint8_t> const& proof_data);
};

} // namespace proofs
} // namespace rollup
