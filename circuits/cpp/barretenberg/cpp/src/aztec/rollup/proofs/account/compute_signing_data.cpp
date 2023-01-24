#include "compute_signing_data.hpp"

namespace rollup {
namespace proofs {
namespace account {

using namespace crypto::pedersen;

barretenberg::fr compute_signing_data(account_tx const& tx)
{
    auto nullifier_1 = tx.compute_account_alias_hash_nullifier();
    auto nullifier_2 = tx.compute_account_public_key_nullifier();

    std::vector<grumpkin::fq> to_compress = {
        tx.alias_hash,
        tx.account_public_key.x,
        tx.new_account_public_key.x,
        tx.new_signing_pub_key_1.x,
        tx.new_signing_pub_key_2.x,
        nullifier_1,
        nullifier_2,
    };
    return compress_native(to_compress);
}

} // namespace account
} // namespace proofs
} // namespace rollup
