#include "account_note.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace account {

using namespace barretenberg;

inline grumpkin::g1::affine_element encrypt(account_note const& note)
{
    std::vector<barretenberg::fr> hash_elements{ note.account_alias_id, note.owner_key.x, note.signing_key.x };
    return crypto::pedersen::encrypt_native(hash_elements, GeneratorIndex::ACCOUNT_NOTE_HASH_INPUTS);
}

} // namespace account
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup