#pragma once
#include "../notes/native/claim/claim_note_tx_data.hpp"
#include "../notes/native/value/value_note.hpp"
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/hash_path.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace plonk::stdlib::types::turbo;

struct join_split_tx {
    uint256_t public_input;
    uint256_t public_output;
    barretenberg::fr public_owner;
    uint32_t asset_id;
    uint32_t num_input_notes;
    std::array<uint32_t, 2> input_index;
    barretenberg::fr old_data_root;
    std::array<merkle_tree::fr_hash_path, 2> input_path;
    std::array<notes::native::value::value_note, 2> input_note;
    std::array<notes::native::value::value_note, 2> output_note;

    notes::native::claim::claim_note_tx_data claim_note;

    grumpkin::fr account_private_key;
    barretenberg::fr alias_hash;
    uint32_t nonce;
    uint32_t account_index;
    merkle_tree::fr_hash_path account_path;
    grumpkin::g1::affine_element signing_pub_key;

    uint32_t propagated_input_index; // 0: none, 1: input_note1, 2: input_note2
    barretenberg::fr backward_link;  // 0: no link, otherwise: any commitment.
    uint32_t allow_chain;            // 0: none, 1: output_note1, 2: output_note2

    crypto::schnorr::signature signature;

    uint32_t proof_id() const
    {
        if (claim_note.deposit_value > 0) {
            return ProofIds::DEFI_DEPOSIT;
        }
        if (public_input > 0) {
            return ProofIds::DEPOSIT;
        }
        if (public_output > 0) {
            return ProofIds::WITHDRAW;
        }
        return ProofIds::SEND;
    }

    uint256_t public_value() const { return public_input > 0 ? public_input : public_output; }

    bool operator==(join_split_tx const&) const = default;
};

void read(uint8_t const*& it, join_split_tx& tx);
void write(std::vector<uint8_t>& buf, join_split_tx const& tx);

std::ostream& operator<<(std::ostream& os, join_split_tx const& tx);

} // namespace join_split
} // namespace proofs
} // namespace rollup
