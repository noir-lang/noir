#include "sign_notes.hpp"
#include "encrypt_note.hpp"
#include "compute_nullifier.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <crypto/schnorr/schnorr.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

using namespace crypto::schnorr;
using namespace crypto::pedersen;

signature sign_notes(proofs::join_split::join_split_tx const& tx,
                     key_pair<grumpkin::fr, grumpkin::g1> const& keys,
                     numeric::random::Engine* engine)
{

    const grumpkin::g1::affine_element input_note_1 = encrypt_note(tx.input_note[0]);
    const grumpkin::g1::affine_element input_note_2 = encrypt_note(tx.input_note[1]);
    const grumpkin::g1::affine_element output_note_1 = encrypt_note(tx.output_note[0]);
    const grumpkin::g1::affine_element output_note_2 = encrypt_note(tx.output_note[1]);

    const auto nullifier1 = compute_nullifier(input_note_1,
                                                tx.input_index[0],
                                                tx.account_private_key,
                                                tx.num_input_notes >= 1);
    const auto nullifier2 = compute_nullifier(input_note_2,
                                                tx.input_index[1],
                                                tx.account_private_key,
                                                tx.num_input_notes >= 2);

    std::vector<grumpkin::fq> to_compress;

    to_compress.push_back(tx.public_input);
    to_compress.push_back(tx.public_output);
    to_compress.push_back(grumpkin::fq(uint64_t(tx.asset_id)));
    to_compress.push_back(output_note_1.x);
    to_compress.push_back(output_note_1.y);
    to_compress.push_back(output_note_2.x);
    to_compress.push_back(output_note_2.y);
    to_compress.push_back(nullifier1);
    to_compress.push_back(nullifier2);
    to_compress.push_back(tx.input_owner);
    to_compress.push_back(tx.output_owner);

    fr compressed = compress_native(to_compress);

    std::vector<uint8_t> message(sizeof(fr));
    fr::serialize_to_buffer(compressed, &message[0]);

    crypto::schnorr::signature signature =
        crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
            std::string(message.begin(), message.end()), keys, engine);
    return signature;
}

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup