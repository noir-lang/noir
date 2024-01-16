#include "compute_nullifier.hpp"
#include "../../constants.hpp"
#include "barretenberg/crypto/blake2s/blake2s.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"

namespace join_split_example::proofs::notes::native {

using namespace bb;

/**
 * Computes a nullifier for a _value_ note
 */
fr compute_nullifier(grumpkin::fq const& note_commitment,
                     grumpkin::fr const& account_private_key,
                     const bool is_note_in_use)
{
    auto hashed_pk = crypto::pedersen_commitment::commit_native(
        { fr(account_private_key) }, GeneratorIndex::JOIN_SPLIT_NULLIFIER_ACCOUNT_PRIVATE_KEY);

    std::vector<bb::fr> buf{
        note_commitment,
        hashed_pk.x,
        hashed_pk.y,
        static_cast<int>(is_note_in_use),
    };
    auto hashed_inputs = crypto::pedersen_hash::hash(buf, GeneratorIndex::JOIN_SPLIT_NULLIFIER);

    auto blake_result = blake2::blake2s(to_buffer(hashed_inputs));

    return from_buffer<fr>(blake_result);
}

} // namespace join_split_example::proofs::notes::native
