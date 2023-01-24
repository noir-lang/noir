#pragma once
#include "../notes/native/asset_id.hpp"
#include "../notes/native/value/complete_partial_commitment.hpp"
#include "../notes/native/claim/claim_note.hpp"
#include "../notes/native/claim/compute_nullifier.hpp"
#include "../notes/native/defi_interaction/note.hpp"
#include "../notes/native/defi_interaction/compute_nullifier.hpp"
#include <stdlib/merkle_tree/hash_path.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace claim {

using namespace plonk::stdlib::types::turbo;

struct claim_tx {
    fr data_root;
    fr defi_root;

    uint32_t claim_note_index;
    merkle_tree::fr_hash_path claim_note_path;
    notes::native::claim::claim_note claim_note;

    uint32_t defi_note_index;
    merkle_tree::fr_hash_path defi_interaction_note_path;
    notes::native::defi_interaction::note defi_interaction_note;

    fr output_value_a;
    fr output_value_b;

    bool operator==(claim_tx const&) const = default;

    std::array<fr, 2> get_output_notes()
    {
        const auto virtual_flag = static_cast<uint32_t>(1 << (MAX_NUM_ASSETS_BIT_LENGTH - 1));
        const auto bridge_call_data = notes::native::bridge_call_data::from_uint256_t(claim_note.bridge_call_data);

        const bool& success = defi_interaction_note.interaction_result;

        const bool first_output_virtual = notes::native::get_asset_id_flag(bridge_call_data.output_asset_id_a);
        const bool second_output_virtual = notes::native::get_asset_id_flag(bridge_call_data.output_asset_id_b);

        const auto asset_id_a_good = first_output_virtual ? virtual_flag + defi_interaction_note.interaction_nonce
                                                          : bridge_call_data.output_asset_id_a;
        const auto asset_id_b_good = second_output_virtual ? virtual_flag + defi_interaction_note.interaction_nonce
                                                           : bridge_call_data.output_asset_id_b;

        const auto& asset_id_a_bad = bridge_call_data.input_asset_id_a;
        const auto& asset_id_b_bad = bridge_call_data.input_asset_id_b;

        const auto asset_id_a = success ? asset_id_a_good : asset_id_a_bad;
        const auto asset_id_b = success ? asset_id_b_good : asset_id_b_bad;

        auto output_note_a = notes::native::value::complete_partial_commitment(
            claim_note.value_note_partial_commitment,
            success ? output_value_a : fr(claim_note.deposit_value),
            asset_id_a,
            notes::native::claim::compute_nullifier(claim_note.commit()));

        auto output_note_b = notes::native::value::complete_partial_commitment(
            claim_note.value_note_partial_commitment,
            success ? output_value_b : fr(claim_note.deposit_value),
            asset_id_b,
            notes::native::defi_interaction::compute_nullifier(defi_interaction_note.commit(), claim_note.commit()));

        bool has_output_two = (success && bridge_call_data.config.second_output_in_use) ||
                              (!success && bridge_call_data.config.second_input_in_use);
        return { output_note_a, has_output_two ? output_note_b : 0 };
    }
};

template <typename B> inline void read(B& buf, claim_tx& tx)
{
    using serialize::read;
    read(buf, tx.data_root);
    read(buf, tx.defi_root);
    read(buf, tx.claim_note_index);
    read(buf, tx.claim_note_path);
    read(buf, tx.claim_note);
    read(buf, tx.defi_note_index);
    read(buf, tx.defi_interaction_note_path);
    read(buf, tx.defi_interaction_note);
    read(buf, tx.output_value_a);
    read(buf, tx.output_value_b);
}

template <typename B> inline void write(B& buf, claim_tx const& tx)
{
    using serialize::write;
    write(buf, tx.data_root);
    write(buf, tx.defi_root);
    write(buf, tx.claim_note_index);
    write(buf, tx.claim_note_path);
    write(buf, tx.claim_note);
    write(buf, tx.defi_note_index);
    write(buf, tx.defi_interaction_note_path);
    write(buf, tx.defi_interaction_note);
    write(buf, tx.output_value_a);
    write(buf, tx.output_value_b);
}

inline std::ostream& operator<<(std::ostream& os, claim_tx const& tx)
{
    return os << "data_root: " << tx.data_root << "\n"
              << "defi_root: " << tx.defi_root << "\n"
              << "claim_note_index: " << tx.claim_note_index << "\n"
              << "claim_note_path: " << tx.claim_note_path << "\n"
              << "defi_note_index: " << tx.defi_note_index << "\n"
              << "interaction_note_path: " << tx.defi_interaction_note_path << "\n"
              << "output_value_a: " << tx.output_value_a << "\n"
              << "output_value_b: " << tx.output_value_b << "\n";
}

} // namespace claim
} // namespace proofs
} // namespace rollup
