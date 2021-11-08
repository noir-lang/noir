#include "account_note.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace account {

grumpkin::fq generate_account_commitment(const barretenberg::fr& account_alias_id,
                                         const barretenberg::fr& owner_x,
                                         const barretenberg::fr& signing_x)
{
    return crypto::pedersen::compress_native({ account_alias_id, owner_x, signing_x },
                                             GeneratorIndex::ACCOUNT_NOTE_COMMITMENT);
}

grumpkin::fq account_note::commit() const
{
    return generate_account_commitment(account_alias_id, owner_key.x, signing_key.x);
}

} // namespace account
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup