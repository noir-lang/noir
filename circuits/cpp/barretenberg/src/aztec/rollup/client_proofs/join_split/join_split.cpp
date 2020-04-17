#include "join_split.hpp"
#include "../../pedersen_note/pedersen_note.hpp"
#include <stdlib/encryption/schnorr/schnorr.hpp>
#include <common/log.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>

namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace plonk;
using namespace pedersen_note;

typedef std::pair<private_note, public_note> note_pair;

static std::shared_ptr<waffle::proving_key> proving_key;
static std::shared_ptr<waffle::verification_key> verification_key;

note_pair create_note_pair(Composer& composer, tx_note const& note)
{
    note_pair result;

    field_ct view_key = witness_ct(&composer, note.secret);
    field_ct note_owner_x = public_witness_ct(&composer, note.owner.x);
    field_ct note_owner_y = public_witness_ct(&composer, note.owner.y);
    uint32_ct witness_value = public_witness_ct(&composer, note.value);
    result.first = { { note_owner_x, note_owner_y }, witness_value, view_key };
    result.second = encrypt_note(result.first);
    return result;
}

void verify_signature(Composer& composer,
                      public_note const& note,
                      grumpkin::g1::affine_element const& pub_key,
                      crypto::schnorr::signature const& sig)
{
    point owner_pub_key = { witness_ct(&composer, pub_key.x), witness_ct(&composer, pub_key.y) };
    stdlib::schnorr::signature_bits signature = stdlib::schnorr::convert_signature(&composer, sig);
    byte_array_ct message = note.ciphertext.x;
    byte_array_ct message2(&composer, message.bits().rbegin(), message.bits().rend());
    stdlib::schnorr::verify_signature(message2, owner_pub_key, signature);
}

void join_split_circuit(Composer& composer, tx_note const& note, crypto::schnorr::signature const& sig)
{
    note_pair note_data = create_note_pair(composer, note);
    verify_signature(composer, note_data.second, note.owner, sig);
}

void init_proving_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory)
{
    // Junk data required just to create proving key.
    tx_note note;
    crypto::schnorr::signature sig;

    Composer composer(std::move(crs_factory));
    join_split_circuit(composer, note, sig);
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

Prover new_join_split_prover(join_split_tx const& )
{
    Composer composer(proving_key, nullptr);
    // join_split_circuit(composer, note, sig);

    info("composer gates: ", composer.get_num_gates());

    Prover prover = composer.create_prover();

    return prover;
}

bool verify_proof(waffle::plonk_proof const& proof)
{
    Verifier verifier(verification_key, Composer::create_manifest(7));
    return verifier.verify_proof(proof);
}

} // namespace create
} // namespace client_proofs
} // namespace rollup