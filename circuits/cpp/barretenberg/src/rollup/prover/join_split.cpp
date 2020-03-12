#include "join_split.hpp"
#include "create.hpp"
#include "destroy.hpp"
#include "note.hpp"
#include <stdlib/encryption/schnorr/schnorr.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>

namespace rollup {
namespace prover {

using namespace rollup::pedersen_note;

void verify_signature(rollup_context& ctx,
                      std::array<public_note, 4> const& notes,
                      grumpkin::g1::affine_element const& pub_key,
                      crypto::schnorr::signature const& sig)
{
    point owner_pub_key = { witness_ct(&ctx.composer, pub_key.x), witness_ct(&ctx.composer, pub_key.y) };
    stdlib::schnorr::signature_bits signature = stdlib::schnorr::convert_signature(&ctx.composer, sig);
    std::array<field_ct, 8> to_compress;
    for (size_t i = 0; i < 4; ++i) {
        to_compress[i * 2] = notes[i].ciphertext.x;
        to_compress[i * 2 + 1] = notes[i].ciphertext.y;
    }
    byte_array_ct message = plonk::stdlib::pedersen::compress_eight(to_compress);
    byte_array_ct message2(&ctx.composer, message.bits().rbegin(), message.bits().rend());
    stdlib::schnorr::verify_signature(message2, owner_pub_key, signature);
}

bool join_split(rollup_context& ctx, join_split_tx const& tx)
{
    uint32_ct public_input = public_witness_ct(&ctx.composer, tx.public_input);
    uint32_ct public_output = public_witness_ct(&ctx.composer, tx.public_output);
    uint32_ct num_input_notes = witness_ct(&ctx.composer, tx.num_input_notes);

    field_ct input_note1_index = witness_ct(&ctx.composer, tx.input_note_index[0]);
    field_ct input_note2_index = witness_ct(&ctx.composer, tx.input_note_index[1]);

    note_pair input_note1_data = create_note_pair(ctx.composer, tx.input_note[0]);
    note_pair input_note2_data = create_note_pair(ctx.composer, tx.input_note[1]);

    note_pair output_note1_data = create_note_pair(ctx.composer, tx.output_note[0]);
    note_pair output_note2_data = create_note_pair(ctx.composer, tx.output_note[1]);
    set_note_public(ctx.composer, output_note1_data.second);
    set_note_public(ctx.composer, output_note2_data.second);

    // Verify input and output notes balance. Use field_ct to prevent overflow.
    field_ct total_in_value =
        field_ct(input_note1_data.first.value) + field_ct(input_note2_data.first.value) + field_ct(public_input);
    field_ct total_out_value =
        field_ct(output_note1_data.first.value) + field_ct(output_note2_data.first.value) + field_ct(public_output);
    // total_in_value = total_in_value.normalize();
    // total_out_value = total_out_value.normalize();
    ctx.composer.assert_equal(total_in_value.witness_index, total_out_value.witness_index);

    // Verify input notes are owned by whoever signed the signature.
    std::array<public_note, 4> notes = {
        input_note1_data.second, input_note2_data.second, output_note1_data.second, output_note2_data.second
    };
    verify_signature(ctx, notes, tx.owner_pub_key, tx.signature);

    auto note1_create_ctx = create_new_note_context(ctx, ctx.data_size, output_note1_data);
    create_note(ctx, note1_create_ctx);

    auto note2_create_ctx = create_new_note_context(ctx, ctx.data_size, output_note2_data);
    create_note(ctx, note2_create_ctx);

    auto note1_destroy_ctx =
        create_destroy_note_context(ctx, input_note1_index, input_note1_data, num_input_notes >= 1);
    destroy_note(ctx, note1_destroy_ctx);

    auto note2_destroy_ctx =
        create_destroy_note_context(ctx, input_note2_index, input_note2_data, num_input_notes >= 2);
    destroy_note(ctx, note2_destroy_ctx);

    ctx.composer.set_public_input(ctx.data_root.witness_index);
    ctx.composer.set_public_input(ctx.nullifier_root.witness_index);

    return true;
}

} // namespace rollup
}
