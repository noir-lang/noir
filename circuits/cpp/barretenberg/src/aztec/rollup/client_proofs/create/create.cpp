#include "create.hpp"
#include <stdlib/encryption/schnorr/schnorr.hpp>
#include "../../pedersen_note/pedersen_note.hpp"

namespace rollup {
namespace client_proofs {
namespace create {

using namespace plonk;
using namespace rollup::tx;
using namespace plonk::stdlib::types::turbo;
using namespace pedersen_note;

typedef std::pair<private_note, public_note> note_pair;

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

waffle::plonk_proof create_note_proof(Composer& composer, tx_note const& note, crypto::schnorr::signature const& sig)
{
    note_pair note_data = create_note_pair(composer, note);

    verify_signature(composer, note_data.second, note.owner, sig);

#ifndef __wasm__
    std::cout << "gates: " << composer.get_num_gates() << std::endl;
#endif

    Prover prover = composer.create_prover();
    waffle::plonk_proof proof = prover.construct_proof();

    return proof;
}

std::vector<uint8_t> create_note_proof(tx_note const& note,
                                       crypto::schnorr::signature const& sig,
                                       std::unique_ptr<waffle::MemReferenceStringFactory>&& crs_factory)
{
    Composer composer(std::move(crs_factory));
    return create_note_proof(composer, note, sig).proof_data;
}

} // namespace create
} // namespace client_proofs
} // namespace rollup