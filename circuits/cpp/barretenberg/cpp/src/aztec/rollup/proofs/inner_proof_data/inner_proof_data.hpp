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
} // namespace InnerProofFields

namespace InnerProofOffsets {
enum {
    PROOF_ID = InnerProofFields::PROOF_ID * 32,
    NOTE_COMMITMENT1 = InnerProofFields::NOTE_COMMITMENT1 * 32,
    NOTE_COMMITMENT2 = InnerProofFields::NOTE_COMMITMENT2 * 32,
    NULLIFIER1 = InnerProofFields::NULLIFIER1 * 32,
    NULLIFIER2 = InnerProofFields::NULLIFIER2 * 32,
    PUBLIC_VALUE = InnerProofFields::PUBLIC_VALUE * 32,
    PUBLIC_OWNER = InnerProofFields::PUBLIC_OWNER * 32,
    PUBLIC_ASSET_ID = InnerProofFields::PUBLIC_ASSET_ID * 32,
    MERKLE_ROOT = InnerProofFields::MERKLE_ROOT * 32,
    TX_FEE = InnerProofFields::TX_FEE * 32,
    TX_FEE_ASSET_ID = InnerProofFields::TX_FEE_ASSET_ID * 32,
    BRIDGE_CALL_DATA = InnerProofFields::BRIDGE_CALL_DATA * 32,
    DEFI_DEPOSIT_VALUE = InnerProofFields::DEFI_DEPOSIT_VALUE * 32,
    DEFI_ROOT = InnerProofFields::DEFI_ROOT * 32,
    BACKWARD_LINK = InnerProofFields::BACKWARD_LINK * 32,
    ALLOW_CHAIN = InnerProofFields::ALLOW_CHAIN * 32,
};
}

struct inner_proof_data {
    uint256_t proof_id;
    barretenberg::fr note_commitment1;
    barretenberg::fr note_commitment2;
    uint256_t nullifier1;
    uint256_t nullifier2;
    uint256_t public_value;
    barretenberg::fr public_owner;
    uint256_t asset_id;

    barretenberg::fr merkle_root;
    uint256_t tx_fee;
    uint256_t tx_fee_asset_id;
    uint256_t bridge_call_data;
    uint256_t defi_deposit_value;
    barretenberg::fr defi_root;

    barretenberg::fr backward_link;
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

} // namespace proofs
} // namespace rollup
