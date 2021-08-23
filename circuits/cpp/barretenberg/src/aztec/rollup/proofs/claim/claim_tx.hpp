#pragma once
#include "../notes/native/claim/claim_note.hpp"
#include "../notes/native/defi_interaction/note.hpp"
#include <stdlib/merkle_tree/hash_path.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace claim {

using namespace plonk::stdlib::types::turbo;

struct claim_tx {
    barretenberg::fr data_root;
    barretenberg::fr defi_root;

    uint32_t claim_note_index;
    merkle_tree::fr_hash_path claim_note_path;
    notes::native::claim::claim_note claim_note;

    merkle_tree::fr_hash_path defi_interaction_note_path;
    notes::native::defi_interaction::note defi_interaction_note;

    fr output_value_a;
    fr output_value_b;

    bool operator==(claim_tx const&) const = default;
};

template <typename B> inline void read(B& buf, claim_tx& tx)
{
    using serialize::read;
    read(buf, tx.data_root);
    read(buf, tx.defi_root);
    read(buf, tx.claim_note_index);
    read(buf, tx.claim_note_path);
    read(buf, tx.claim_note);
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
              << "interaction_note_path: " << tx.defi_interaction_note_path << "\n"
              << "output_value_a: " << tx.output_value_a << "\n"
              << "output_value_b: " << tx.output_value_b << "\n";
}

} // namespace claim
} // namespace proofs
} // namespace rollup
