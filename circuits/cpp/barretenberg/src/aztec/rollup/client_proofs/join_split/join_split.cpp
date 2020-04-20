#include "join_split.hpp"
#include "../../pedersen_note/pedersen_note.hpp"
#include <stdlib/encryption/schnorr/schnorr.hpp>
#include <common/log.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <stdlib/merkle_tree/merkle_tree.hpp>
#include "note_pair.hpp"
#include "verify_signature.hpp"

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace plonk;
using namespace pedersen_note;

typedef std::pair<private_note, public_note> note_pair;

static std::shared_ptr<waffle::proving_key> proving_key;
static std::shared_ptr<waffle::verification_key> verification_key;

field_ct process_input_note(Composer& composer,
                            field_ct merkle_root,
                            merkle_tree::fr_hash_path hash_path,
                            field_ct index,
                            note_pair const& note,
                            bool_ct is_real)
{
    // Check that the input note data, follows the given hash paths, to the publically given merkle root.
    auto witness_hash_path = merkle_tree::create_witness_hash_path(composer, hash_path);

    byte_array_ct leaf(&composer);
    leaf.write(note.second.ciphertext.x).write(note.second.ciphertext.y);

    bool_ct exists =
        merkle_tree::check_membership(composer, merkle_root, witness_hash_path, leaf, byte_array_ct(index));
    composer.assert_equal(is_real.witness_index, exists.witness_index);

    // Compute input notes nullifier index. We mix in the index and notes secret as part of the value we hash into the
    // tree to ensure notes will always have unique entries.
    // [256 bits of encrypted note x coord][32 most sig bits of index][223 bits of note viewing key][1 bit is_real]
    byte_array_ct note_hash_data = byte_array_ct(&composer);
    note_hash_data.write(note.second.ciphertext.x)
        .write(byte_array_ct(index).slice(28, 4))
        .write(byte_array_ct(note.first.secret).slice(4, 28));
    note_hash_data.set_bit(511, is_real);

    // We have to convert the byte_array_ct into a field_ct to get the montgomery form. Can we avoid this?
    field_ct nullifier_index = stdlib::merkle_tree::hash_value(note_hash_data);

    return nullifier_index;
}

void join_split_circuit(Composer& composer, join_split_tx const& tx)
{
    uint32_ct public_input = public_witness_ct(&composer, tx.public_input);
    uint32_ct public_output = public_witness_ct(&composer, tx.public_output);
    uint32_ct num_input_notes = witness_ct(&composer, tx.num_input_notes);

    field_ct input_note1_index = witness_ct(&composer, tx.input_index[0]);
    field_ct input_note2_index = witness_ct(&composer, tx.input_index[1]);

    note_pair input_note1_data = create_note_pair(composer, tx.input_note[0]);
    note_pair input_note2_data = create_note_pair(composer, tx.input_note[1]);

    note_pair output_note1_data = create_note_pair(composer, tx.output_note[0]);
    note_pair output_note2_data = create_note_pair(composer, tx.output_note[1]);
    set_note_public(composer, output_note1_data.second);
    set_note_public(composer, output_note2_data.second);

    // Verify input and output notes balance. Use field_ct to prevent overflow.
    field_ct total_in_value =
        field_ct(input_note1_data.first.value) + field_ct(input_note2_data.first.value) + field_ct(public_input);
    field_ct total_out_value =
        field_ct(output_note1_data.first.value) + field_ct(output_note2_data.first.value) + field_ct(public_output);
    // total_in_value = total_in_value.normalize();
    // total_out_value = total_out_value.normalize();
    composer.assert_equal(total_in_value.witness_index, total_out_value.witness_index);

    // Verify input notes have the same owner.
    composer.assert_equal(input_note1_data.first.owner.x.witness_index, input_note2_data.first.owner.x.witness_index);
    composer.assert_equal(input_note1_data.first.owner.y.witness_index, input_note2_data.first.owner.y.witness_index);

    // Verify that the given signature was signed over all 4 notes using the input note owners private key.
    std::array<public_note, 4> notes = {
        input_note1_data.second, input_note2_data.second, output_note1_data.second, output_note2_data.second
    };
    verify_signature(composer, notes, tx.input_note[0].owner, tx.signature);

    field_ct merkle_root = public_witness_ct(&composer, tx.merkle_root);
    field_ct nullifier1 = process_input_note(
        composer, merkle_root, tx.input_path[0], input_note1_index, input_note1_data, num_input_notes >= 1);
    field_ct nullifier2 = process_input_note(
        composer, merkle_root, tx.input_path[1], input_note2_index, input_note2_data, num_input_notes >= 2);

    composer.set_public_input(nullifier1.witness_index);
    composer.set_public_input(nullifier2.witness_index);
}

void init_proving_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory)
{
    // Junk data required just to create proving key.
    join_split_tx tx;
    tx.input_path[0].resize(32);
    tx.input_path[1].resize(32);

    Composer composer(std::move(crs_factory));
    join_split_circuit(composer, tx);
    proving_key = composer.compute_proving_key();
}

void init_verification_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory)
{
    if (!proving_key) {
        std::abort();
    }
    // Patch the 'nothing' reference string fed to init_proving_key.
    proving_key->reference_string = crs_factory->get_prover_crs(proving_key->n);
    verification_key = waffle::turbo_composer::compute_verification_key(proving_key, crs_factory->get_verifier_crs());
}

Prover new_join_split_prover(join_split_tx const& tx)
{
    Composer composer(proving_key, nullptr);
    join_split_circuit(composer, tx);

    info("composer gates: ", composer.get_num_gates());
    info("public inputs: ", composer.public_inputs.size());

    Prover prover = composer.create_prover();

    return prover;
}

bool verify_proof(waffle::plonk_proof const& proof)
{
    Verifier verifier(verification_key, Composer::create_manifest(9));
    return verifier.verify_proof(proof);
}

} // namespace join_split
} // namespace client_proofs
} // namespace rollup