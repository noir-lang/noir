#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/srs/factories/file_crs_factory.hpp"

#include <gtest/gtest.h>

namespace bb {

template <typename Curve> class CommitmentKeyTest : public ::testing::Test {
    using CK = CommitmentKey<Curve>;

    using Fr = typename Curve::ScalarField;
    using Commitment = typename Curve::AffineElement;
    using Polynomial = bb::Polynomial<Fr>;

  public:
    template <class CK> inline std::shared_ptr<CK> create_commitment_key(size_t num_points);
};

template <>
template <>
std::shared_ptr<CommitmentKey<curve::BN254>> CommitmentKeyTest<curve::BN254>::create_commitment_key<
    CommitmentKey<curve::BN254>>(const size_t num_points)
{
    srs::init_crs_factory("../srs_db/ignition");
    return std::make_shared<CommitmentKey<curve::BN254>>(num_points);
}

template <>
template <>
std::shared_ptr<CommitmentKey<curve::Grumpkin>> CommitmentKeyTest<curve::Grumpkin>::create_commitment_key<
    CommitmentKey<curve::Grumpkin>>(const size_t num_points)
{
    srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
    return std::make_shared<CommitmentKey<curve::Grumpkin>>(num_points);
}

using Curves = ::testing::Types<curve::BN254, curve::Grumpkin>;

TYPED_TEST_SUITE(CommitmentKeyTest, Curves);

// Check that commit and commit_sparse return the same result for a random sparse polynomial
TYPED_TEST(CommitmentKeyTest, CommitSparse)
{
    using Curve = TypeParam;
    using CK = CommitmentKey<Curve>;
    using G1 = Curve::AffineElement;
    using Fr = Curve::ScalarField;
    using Polynomial = bb::Polynomial<Fr>;

    const size_t num_points = 1 << 12; // large enough to ensure normal pippenger logic is used
    const size_t num_nonzero = 7;

    // Construct a sparse random polynomial
    Polynomial poly{ num_points };
    for (size_t i = 0; i < num_nonzero; ++i) {
        size_t idx = (i + 1) * (i + 1) % num_points;
        poly[idx] = Fr::random_element();
    }

    // Commit to the polynomial using both the conventional commit method and the sparse commitment method
    auto key = TestFixture::template create_commitment_key<CK>(num_points);
    G1 commit_result = key->commit(poly);
    G1 sparse_commit_result = key->commit_sparse(poly);

    EXPECT_EQ(sparse_commit_result, commit_result);
}

} // namespace bb
