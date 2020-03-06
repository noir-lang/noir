#pragma once
#include "note.hpp"

namespace rollup {

struct destroy_note_context {
    note_pair note_data;
    byte_array data_index;
    field_t data_root;
    hash_path data_path;
    byte_array data_value;
    field_t nullifier_index;
    hash_path nullifier_old_path;
    hash_path nullifier_new_path;
    field_t nullifier_old_root;
    field_t nullifier_new_root;
    byte_array nullifier_value;
    bool_t is_real;
};

destroy_note_context create_destroy_note_context(rollup_context& ctx,
                                                 field_t const& index_field,
                                                 note_pair const& note_data,
                                                 bool_t is_real);

void destroy_note(rollup_context& ctx, destroy_note_context const& destroy_ctx);

bool destroy(rollup_context& ctx, uint32_t const index, tx_note const& note);

}