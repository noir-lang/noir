#pragma once
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include <array>

namespace bb::join_split_example::proofs {

namespace inner_proof_fields {
enum {
    PROOF_ID,
    NOTE_COMMITMENT1,
    NOTE_COMMITMENT2,
    NULLIFIER1,
    NULLIFIER2,
    PUBLIC_VALUE,
    PUBLIC_OWNER,
    PUBLIC_ASSET_ID,
    MERKLE_ROOT,
    TX_FEE,
    TX_FEE_ASSET_ID,
    BRIDGE_CALL_DATA,
    DEFI_DEPOSIT_VALUE,
    DEFI_ROOT,
    BACKWARD_LINK,
    ALLOW_CHAIN,
    NUM_FIELDS
};
} // namespace inner_proof_fields

namespace inner_proof_offsets {
enum {
    PROOF_ID = inner_proof_fields::PROOF_ID * 32,
    NOTE_COMMITMENT1 = inner_proof_fields::NOTE_COMMITMENT1 * 32,
    NOTE_COMMITMENT2 = inner_proof_fields::NOTE_COMMITMENT2 * 32,
    NULLIFIER1 = inner_proof_fields::NULLIFIER1 * 32,
    NULLIFIER2 = inner_proof_fields::NULLIFIER2 * 32,
    PUBLIC_VALUE = inner_proof_fields::PUBLIC_VALUE * 32,
    PUBLIC_OWNER = inner_proof_fields::PUBLIC_OWNER * 32,
    PUBLIC_ASSET_ID = inner_proof_fields::PUBLIC_ASSET_ID * 32,
    MERKLE_ROOT = inner_proof_fields::MERKLE_ROOT * 32,
    TX_FEE = inner_proof_fields::TX_FEE * 32,
    TX_FEE_ASSET_ID = inner_proof_fields::TX_FEE_ASSET_ID * 32,
    BRIDGE_CALL_DATA = inner_proof_fields::BRIDGE_CALL_DATA * 32,
    DEFI_DEPOSIT_VALUE = inner_proof_fields::DEFI_DEPOSIT_VALUE * 32,
    DEFI_ROOT = inner_proof_fields::DEFI_ROOT * 32,
    BACKWARD_LINK = inner_proof_fields::BACKWARD_LINK * 32,
    ALLOW_CHAIN = inner_proof_fields::ALLOW_CHAIN * 32,
};
}

struct inner_proof_data {
    uint256_t proof_id;
    bb::fr note_commitment1;
    bb::fr note_commitment2;
    uint256_t nullifier1;
    uint256_t nullifier2;
    uint256_t public_value;
    bb::fr public_owner;
    uint256_t asset_id;

    bb::fr merkle_root;
    uint256_t tx_fee;
    uint256_t tx_fee_asset_id;
    uint256_t bridge_call_data;
    uint256_t defi_deposit_value;
    bb::fr defi_root;

    bb::fr backward_link;
    uint256_t allow_chain;

    inner_proof_data(std::vector<uint8_t> const& proof_data);
};

inline std::ostream& operator<<(std::ostream& os, inner_proof_data const& data)
{
    // clang-format off
    return os << "{\n"
        << "  proof_id: " << data.proof_id << "\n"
        << "  note_commitment1: " << data.note_commitment1 << "\n"
        << "  note_commitment2: " << data.note_commitment2 << "\n"
        << "  nullifier1: " << data.nullifier1 << "\n"
        << "  nullifier2: " << data.nullifier2 << "\n"
        << "  public_value: " << data.public_value << "\n"
        << "  public_owner: " << data.public_owner << "\n"
        << "  asset_id: " << data.asset_id << "\n"
        << "  merkle_root: " << data.merkle_root << "\n"
        << "  tx_fee: " << data.tx_fee << "\n"
        << "  tx_fee_asset_id: " << data.tx_fee_asset_id << "\n"
        << "  bridge_call_data: " << data.bridge_call_data << "\n"
        << "  defi_deposit_value: " << data.defi_deposit_value << "\n"
        << "  defi_root: " << data.defi_root << "\n"
        << "  backward_link: " << data.backward_link << "\n"
        << "  allow_chain: " << data.allow_chain << "\n"
        << "}";
    // clang-format on
}

} // namespace bb::join_split_example::proofs
