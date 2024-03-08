#pragma once
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/examples/join_split/constants.hpp"

namespace bb::join_split_example::proofs::notes::native::account {

grumpkin::fq generate_account_commitment(const bb::fr& alias_hash, const bb::fr& owner_x, const bb::fr& signing_x);

struct account_note {
    bb::fr alias_hash;
    grumpkin::g1::affine_element owner_key;
    grumpkin::g1::affine_element signing_key;

    grumpkin::fq commit() const;
};

} // namespace bb::join_split_example::proofs::notes::native::account
