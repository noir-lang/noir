#include "sign_join_split_tx.hpp"
#include "barretenberg/crypto/schnorr/schnorr.hpp"
#include "compute_signing_data.hpp"

namespace join_split_example {
namespace proofs {
namespace join_split {

using namespace crypto::schnorr;

signature sign_join_split_tx(join_split_tx const& tx, key_pair<grumpkin::fr, grumpkin::g1> const& keys)
{
    fr compressed = compute_signing_data(tx);

    std::vector<uint8_t> message(sizeof(fr));
    fr::serialize_to_buffer(compressed, &message[0]);

    crypto::schnorr::signature signature =
        crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
            std::string(message.begin(), message.end()), keys);
    return signature;
}

} // namespace join_split
} // namespace proofs
} // namespace join_split_example
