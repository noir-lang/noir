#include "create.hpp"
#include "destroy.hpp"
#include "note.hpp"
#include "split.hpp"

namespace rollup {

bool join(rollup_context& ctx,
          uint32_t in_index1,
          uint32_t in_index2,
          tx_note const& in_note1,
          tx_note const& in_note2,
          tx_note const& out_note)
{
    field_t in_index1_field = witness_t(&ctx.composer, in_index1);
    field_t in_index2_field = witness_t(&ctx.composer, in_index2);
    note_pair in_note1_data = create_note_pair(ctx.composer, in_note1);
    note_pair in_note2_data = create_note_pair(ctx.composer, in_note2);
    note_pair out_note_data = create_note_pair(ctx.composer, out_note);
    field_t total_input = field_t(in_note1_data.first.value) + field_t(in_note2_data.first.value);

    ctx.composer.assert_equal(out_note_data.first.value.get_witness_index(), total_input.witness_index);

    auto new_note = create_new_note_context(ctx, ctx.data_size, out_note_data);
    create_note(ctx, new_note);

    auto note1 = create_destroy_note_context(ctx, in_index1_field, in_note1_data, witness_t(&ctx.composer, true));
    set_note_public(ctx.composer, note1.note_data.second);
    destroy_note(ctx, note1);

    auto note2 = create_destroy_note_context(ctx, in_index2_field, in_note2_data, witness_t(&ctx.composer, true));
    set_note_public(ctx.composer, note2.note_data.second);
    destroy_note(ctx, note2);

    ctx.composer.set_public_input(ctx.data_root.witness_index);
    ctx.composer.set_public_input(ctx.nullifier_root.witness_index);

    return true;
}

} // namespace rollup