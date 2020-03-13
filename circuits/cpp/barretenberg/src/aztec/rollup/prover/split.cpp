#include "split.hpp"
#include "create.hpp"
#include "destroy.hpp"
#include "note.hpp"
#include "rollup_context.hpp"

namespace rollup {
namespace prover {

bool split(
    rollup_context& ctx, uint32_t in_index, tx_note const& in_note, tx_note const& out_note1, tx_note const& out_note2)
{
    note_pair in_note_data = create_note_pair(ctx.composer, in_note);
    note_pair out_note1_data = create_note_pair(ctx.composer, out_note1);
    note_pair out_note2_data = create_note_pair(ctx.composer, out_note2);
    field_ct in_index_field = witness_ct(&ctx.composer, in_index);
    field_ct total_output = field_ct(out_note1_data.first.value) + field_ct(out_note2_data.first.value);

    ctx.composer.assert_equal(in_note_data.first.value.get_witness_index(), total_output.witness_index);

    auto note1 = create_new_note_context(ctx, ctx.data_size, out_note1_data);
    set_note_public(ctx.composer, note1.note_data.second);
    create_note(ctx, note1);

    auto note2 = create_new_note_context(ctx, ctx.data_size, out_note2_data);
    set_note_public(ctx.composer, note2.note_data.second);
    create_note(ctx, note2);

    auto create_note_ctx =
        create_destroy_note_context(ctx, in_index_field, in_note_data, witness_ct(&ctx.composer, true));
    destroy_note(ctx, create_note_ctx);

    ctx.composer.set_public_input(ctx.data_root.witness_index);
    ctx.composer.set_public_input(ctx.nullifier_root.witness_index);

    return true;
}

} // namespace prover
} // namespace rollup