#pragma once
#include "note.hpp"
#include "rollup_context.hpp"

namespace rollup {
namespace prover {

using namespace rollup::tx;

struct destroy_note_context {
    note_pair note_data;
    byte_array_ct data_index;
    field_ct data_root;
    hash_path data_path;
    byte_array_ct data_value;
    field_ct nullifier_index;
    hash_path nullifier_old_path;
    hash_path nullifier_new_path;
    field_ct nullifier_old_root;
    field_ct nullifier_new_root;
    byte_array_ct nullifier_value;
    bool_ct is_real;
};

destroy_note_context create_destroy_note_context(rollup_context& ctx,
                                                 field_ct const& index_field,
                                                 note_pair const& note_data,
                                                 bool_ct is_real);

void destroy_note(rollup_context& ctx, destroy_note_context const& destroy_ctx);

bool destroy(rollup_context& ctx, uint32_t const index, tx_note const& note);

} // namespace prover
} // namespace rollup