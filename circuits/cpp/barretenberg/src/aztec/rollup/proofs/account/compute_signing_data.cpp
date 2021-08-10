#include "compute_signing_data.hpp"

namespace rollup {
namespace proofs {
namespace account {

using namespace crypto::pedersen;

barretenberg::fr compute_signing_data(account_tx const& tx)
{
    std::vector<grumpkin::fq> to_compress = {
        tx.account_alias_id(),      tx.account_public_key.x,    tx.new_account_public_key.x,
        tx.new_signing_pub_key_1.x, tx.new_signing_pub_key_2.x,
    };
    return compress_native(to_compress);
}

} // namespace account
} // namespace proofs
} // namespace rollup
