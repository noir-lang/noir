#include "destroy.hpp"

namespace rollup {

destroy_note_context create_destroy_note_context(rollup_context& ctx,
                                                 field_t const& index_field,
                                                 note_pair const& note_data,
                                                 bool_t is_real)
{
    uint128_t index_to_destroy = field_to_uint128(index_field.get_value());
    field_t data_root = ctx.data_root;
    fr_hash_path data_path = ctx.data_db.get_hash_path(index_to_destroy);
    byte_array data_value = create_note_leaf(ctx.composer, note_data.second);

    // We mix in the index and notes secret as part of the value we hash into the tree to ensure notes will always have
    // unique entries.
    byte_array note_hash_data = byte_array(&ctx.composer);
    note_hash_data.write(note_data.second.ciphertext.x)
        .write(byte_array(index_field).slice(28, 4))
        .write(byte_array(note_data.first.secret).slice(4, 28));
    note_hash_data.set_bit(511, is_real);

    // We have to convert the byte_array into a field_t to get the montgomery form. Can we avoid this?
    field_t nullifier_index = stdlib::merkle_tree::hash_value(note_hash_data);
    uint128_t nullifier_index_raw = field_to_uint128(nullifier_index.get_value());

    byte_array nullifier_value(&ctx.composer);
    nullifier_value.write(field_t(1ULL)).write(field_t(uint64_t(0)));

    fr_hash_path nullifier_old_path = ctx.nullifier_db.get_hash_path(nullifier_index_raw);
    fr_hash_path nullifier_new_path =
        stdlib::merkle_tree::get_new_hash_path(nullifier_old_path, nullifier_index_raw, nullifier_value.get_value());

    field_t nullifier_old_root = ctx.nullifier_root;
    field_t nullifier_new_root = witness_t(&ctx.composer, stdlib::merkle_tree::get_hash_path_root(nullifier_new_path));

    destroy_note_context note_ctx = {
        note_data,
        index_field,
        data_root,
        stdlib::merkle_tree::create_witness_hash_path(ctx.composer, data_path),
        data_value,
        nullifier_index,
        stdlib::merkle_tree::create_witness_hash_path(ctx.composer, nullifier_old_path),
        stdlib::merkle_tree::create_witness_hash_path(ctx.composer, nullifier_new_path),
        nullifier_old_root,
        nullifier_new_root,
        nullifier_value,
        is_real,
    };

    return note_ctx;
}

void destroy_note(rollup_context& ctx, destroy_note_context const& destroy_ctx)
{
    // Check that the note we want to destroy exists.
    bool_t exists = stdlib::merkle_tree::check_membership(
        ctx.composer, destroy_ctx.data_root, destroy_ctx.data_path, destroy_ctx.data_value, destroy_ctx.data_index);
    ctx.composer.assert_equal(destroy_ctx.is_real.witness_index, exists.witness_index);

    stdlib::merkle_tree::update_membership(ctx.composer,
                                           destroy_ctx.nullifier_new_root,
                                           destroy_ctx.nullifier_new_path,
                                           destroy_ctx.nullifier_value,
                                           destroy_ctx.nullifier_old_root,
                                           destroy_ctx.nullifier_old_path,
                                           byte_array(&ctx.composer, 64),
                                           static_cast<byte_array>(destroy_ctx.nullifier_index));

    ctx.nullifier_db.update_element(field_to_uint128(destroy_ctx.nullifier_index.get_value()),
                                    destroy_ctx.nullifier_value.get_value());

    ctx.nullifier_root = destroy_ctx.nullifier_new_root;
}

bool destroy(rollup_context& ctx, uint32_t const index, tx_note const& note)
{
    field_t index_to_destroy_field = witness_t(&ctx.composer, index);
    note_pair note_data = create_note_pair(ctx.composer, note);
    destroy_note_context destroy_ctx =
        create_destroy_note_context(ctx, index_to_destroy_field, note_data, witness_t(&ctx.composer, true));
    set_note_public(ctx.composer, destroy_ctx.note_data.second);

    destroy_note(ctx, destroy_ctx);

    ctx.composer.set_public_input(ctx.nullifier_root.witness_index);

    return true;
}

} // namespace rollup