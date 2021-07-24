#pragma once
#include <numeric/uint256/uint256.hpp>
#include <numeric/uint128/uint128.hpp>
#include <ecc/curves/bn254/fr.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <array>

namespace rollup {
namespace proofs {

using namespace barretenberg;

namespace InnerProofFields {
enum {
    PROOF_ID,
    PUBLIC_INPUT,
    PUBLIC_OUTPUT,
    ASSET_ID,
    NOTE_COMMITMENT1,
    NOTE_COMMITMENT2,
    NULLIFIER1,
    NULLIFIER2,
    INPUT_OWNER,
    OUTPUT_OWNER,
    MERKLE_ROOT,
    TX_FEE,
    NUM_FIELDS
};
} // namespace InnerProofFields

namespace InnerProofOffsets {
enum {
    PROOF_ID = InnerProofFields::PROOF_ID * 32,
    PUBLIC_INPUT = InnerProofFields::PUBLIC_INPUT * 32,
    PUBLIC_OUTPUT = InnerProofFields::PUBLIC_OUTPUT * 32,
    ASSET_ID = InnerProofFields::ASSET_ID * 32,
    NOTE_COMMITMENT1 = InnerProofFields::NOTE_COMMITMENT1 * 32,
    NOTE_COMMITMENT2 = InnerProofFields::NOTE_COMMITMENT2 * 32,
    NULLIFIER1 = InnerProofFields::NULLIFIER1 * 32,
    NULLIFIER2 = InnerProofFields::NULLIFIER2 * 32,
    INPUT_OWNER = InnerProofFields::INPUT_OWNER * 32,
    OUTPUT_OWNER = InnerProofFields::OUTPUT_OWNER * 32,
    MERKLE_ROOT = InnerProofFields::MERKLE_ROOT * 32,
    TX_FEE = InnerProofFields::TX_FEE * 32,
};
}

struct inner_proof_data {
    uint256_t proof_id;
    uint256_t public_input;
    uint256_t public_output;
    uint256_t asset_id;
    barretenberg::fr note_commitment1;
    barretenberg::fr note_commitment2;
    uint256_t nullifier1;
    uint256_t nullifier2;
    barretenberg::fr input_owner;
    barretenberg::fr output_owner;

    barretenberg::fr merkle_root;
    uint256_t tx_fee;

    inner_proof_data(std::vector<uint8_t> const& proof_data);
};

inline std::ostream& operator<<(std::ostream& os, inner_proof_data const& data)
{
    // clang-format off
    return os << "{\n"
        << "  proof_id: " << data.proof_id << "\n"
        << "  public_input: " << data.public_input << "\n"
        << "  public_output: " << data.public_output << "\n"
        << "  asset_id: " << data.asset_id << "\n"
        << "  note_commitment1: " << data.note_commitment1 << "\n"
        << "  note_commitment2: " << data.note_commitment2 << "\n"
        << "  nullifier1: " << data.nullifier1 << "\n"
        << "  nullifier2: " << data.nullifier2 << "\n"
        << "  input_owner: " << data.input_owner << "\n"
        << "  output_owner: " << data.output_owner << "\n"
        << "  merkle_root: " << data.merkle_root << "\n"
        << "  tx_fee: " << data.tx_fee << "\n"
        << "}";
    // clang-format on
}

} // namespace proofs
} // namespace rollup
