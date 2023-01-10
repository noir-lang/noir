
#include "kzg.hpp"

#include "../commitment_key.test.hpp"

#include <ecc/curves/bn254/g1.hpp>

#include <gtest/gtest.h>

namespace honk::pcs::kzg {

template <class Params> class BilinearAccumulationTest : public CommitmentTest<Params> {
  public:
    using Fr = typename Params::Fr;

    using Commitment = typename Params::Commitment;
    using Polynomial = barretenberg::Polynomial<Fr>;

    using Accumulator = BilinearAccumulator<Params>;

    void verify_accumulators(const Accumulator& prover_acc, const Accumulator& verifier_acc)
    {
        EXPECT_EQ(prover_acc, verifier_acc) << "BilinearAccumulation: accumulator mismatch";
        EXPECT_TRUE(prover_acc.verify(this->vk())) << "BilinearAccumulation: pairing check failed";
    }
};

TYPED_TEST_SUITE(BilinearAccumulationTest, CommitmentSchemeParams);

TYPED_TEST(BilinearAccumulationTest, single)
{
    const size_t n = 16;

    using OpeningScheme = UnivariateOpeningScheme<TypeParam>;

    auto [claim, witness] = this->random_claim(n);

    auto [acc, proof] = OpeningScheme::reduce_prove(this->ck(), claim, witness);
    auto result_acc = OpeningScheme::reduce_verify(claim, proof);

    this->verify_accumulators(acc, result_acc);
}

} // namespace honk::pcs::kzg