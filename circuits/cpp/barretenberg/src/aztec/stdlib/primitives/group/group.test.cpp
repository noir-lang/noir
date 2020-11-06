#include "../../types/turbo.hpp"
#include <common/test.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

TEST(stdlib_group, test_fixed_base_scalar_mul)
{
    auto scalar = uint256_t(123, 0, 0, 0);
    auto priv_key = grumpkin::fr(scalar);
    auto pub_key = crypto::pedersen::get_generator(0) * priv_key;

    Composer composer;
    auto priv_key_witness = field_ct(witness_ct(&composer, fr(scalar)));

    auto result = group_ct::fixed_base_scalar_mul<128>(priv_key_witness, 0);

    EXPECT_EQ(result.x.get_value(), pub_key.x);
    EXPECT_EQ(result.y.get_value(), pub_key.y);

    auto native_result = crypto::pedersen::fixed_base_scalar_mul<128>(barretenberg::fr(scalar), 0);
    EXPECT_EQ(native_result.x, pub_key.x);
    EXPECT_EQ(native_result.y, pub_key.y);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}
