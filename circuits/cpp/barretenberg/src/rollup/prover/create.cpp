#include "create.hpp"

namespace rollup {

new_note_context create_new_note_context(rollup_context& ctx, field_t const& index_field, note_pair const& note_data)
{
    uint128_t index_to_create = field_to_uint128(index_field.get_value());

    std::string new_element = create_note_db_element(note_data.second);

    fr_hash_path old_path = ctx.data_db.get_hash_path(index_to_create);
    fr_hash_path new_path = stdlib::merkle_tree::get_new_hash_path(old_path, index_to_create, new_element);

    byte_array new_value_byte_array(&ctx.composer);
    new_value_byte_array.write(note_data.second.ciphertext.x).write(note_data.second.ciphertext.y);

    field_t old_root = ctx.data_root;
    field_t new_root = witness_t(&ctx.composer, stdlib::merkle_tree::get_hash_path_root(new_path));

    new_note_context note_ctx = {
        index_field,
        note_data,
        stdlib::merkle_tree::create_witness_hash_path(ctx.composer, old_path),
        stdlib::merkle_tree::create_witness_hash_path(ctx.composer, new_path),
        old_root,
        new_root,
        new_value_byte_array,
    };

    return note_ctx;
}

void create_note(rollup_context& ctx, new_note_context const& note_ctx)
{
    stdlib::merkle_tree::update_membership(ctx.composer,
                                           note_ctx.new_root,
                                           note_ctx.new_path,
                                           note_ctx.value,
                                           note_ctx.old_root,
                                           note_ctx.old_path,
                                           byte_array(&ctx.composer, 64),
                                           note_ctx.note_index);

    ctx.data_db.update_element(ctx.data_db.size(), note_ctx.value.get_value());

    ctx.data_size = (ctx.data_size + 1).normalize();
    ctx.data_root = note_ctx.new_root;
}

bool create(rollup_context& ctx, tx_note const& note)
{
    note_pair note_data = create_note_pair(ctx.composer, note);
    new_note_context note_ctx = create_new_note_context(ctx, ctx.data_size, note_data);
    set_note_public(ctx.composer, note_ctx.note_data.second);
    create_note(ctx, note_ctx);
    ctx.composer.set_public_input(ctx.data_root.witness_index);
    return true;
}

} // namespace rollup