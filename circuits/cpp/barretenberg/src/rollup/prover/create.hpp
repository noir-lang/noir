#pragma once
#include "note.hpp"

namespace rollup {

struct new_note_context {
    byte_array note_index;
    note_pair note_data;
    hash_path old_path;
    hash_path new_path;
    field_t old_root;
    field_t new_root;
    byte_array value;
};

new_note_context create_new_note_context(rollup_context& ctx, field_t const& index_field, note_pair const& note_data);

void create_note(rollup_context& ctx, new_note_context const& note_ctx);

bool create(rollup_context& ctx, tx_note const& note);

}