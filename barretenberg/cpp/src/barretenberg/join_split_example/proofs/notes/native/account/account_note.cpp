#include "account_note.hpp"
#include "../../constants.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"

namespace join_split_example::proofs::notes::native::account {

grumpkin::fq generate_account_commitment(const bb::fr& alias_hash, const bb::fr& owner_x, const bb::fr& signing_x)
{
    return crypto::pedersen_hash::hash({ alias_hash, owner_x, signing_x }, GeneratorIndex::ACCOUNT_NOTE_COMMITMENT);
}

grumpkin::fq account_note::commit() const
{
    return generate_account_commitment(alias_hash, owner_key.x, signing_key.x);
}

} // namespace join_split_example::proofs::notes::native::account
