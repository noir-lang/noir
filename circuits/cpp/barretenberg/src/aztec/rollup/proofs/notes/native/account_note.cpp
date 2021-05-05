#include "account_note.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include "../constants.hpp"

using namespace barretenberg;

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

grumpkin::g1::affine_element encrypt_account_note(account_note const& note)
{
    std::vector<barretenberg::fr> hash_elements{ note.account_alias_id, note.owner_key.x, note.signing_key.x };
    return crypto::pedersen::encrypt_native(hash_elements, GeneratorIndex::ACCOUNT_NOTE_HASH_INPUTS);
}

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup