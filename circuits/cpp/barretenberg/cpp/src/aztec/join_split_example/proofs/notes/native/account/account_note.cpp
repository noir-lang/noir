#include "account_note.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include "../../constants.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace native {
namespace account {

grumpkin::fq generate_account_commitment(const barretenberg::fr& alias_hash,
                                         const barretenberg::fr& owner_x,
                                         const barretenberg::fr& signing_x)
{
    return crypto::pedersen::compress_native({ alias_hash, owner_x, signing_x },
                                             GeneratorIndex::ACCOUNT_NOTE_COMMITMENT);
}

grumpkin::fq account_note::commit() const
{
    return generate_account_commitment(alias_hash, owner_key.x, signing_key.x);
}

} // namespace account
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace join_split_example