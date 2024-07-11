#include "barretenberg/ecc/scalar_multiplication/sorted_msm.hpp"
#include "barretenberg/common/mem.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/common/zip_view.hpp"
#include "barretenberg/ecc/scalar_multiplication/point_table.hpp"
#include "barretenberg/ecc/scalar_multiplication/scalar_multiplication.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/srs/factories/file_crs_factory.hpp"
#include "barretenberg/srs/io.hpp"

#include <cstddef>
#include <vector>

namespace bb {

namespace {
auto& engine = numeric::get_debug_randomness();
}

template <typename Curve> class SortedMsmTests : public ::testing::Test {

  public:
    using G1 = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;

    struct TestData {
        size_t num_points;
        std::vector<G1> points;
        std::vector<Fr> scalars;
        G1 msm_result;
    };

    /**
     * @brief Generate a set of random points and scalars based on an input sequence_counts
     * @details E.g. given sequence counts {7, 2, 9}, generate a set of random points and scalars with only 3 unique
     * scalar values repeated according to the sequence counts. Also compute the result of the corresponding MSM for
     * test comparisons.
     *
     * @param sequence_counts
     * @return TestData
     */
    TestData generate_test_data_from_sequence_counts(std::span<uint64_t> sequence_counts)
    {
        // Generate duplicate scalars corresponding to the sequence counts
        size_t num_points{ 0 };
        std::vector<Fr> scalars;
        for (auto& count : sequence_counts) {
            Fr repeated_scalar = Fr::random_element();
            for (size_t i = 0; i < count; ++i) {
                scalars.emplace_back(repeated_scalar);
                num_points++;
            }
        }

        // Randomly shuffle the scalars so duplicates are no longer grouped together
        std::random_device rd;
        std::shuffle(scalars.begin(), scalars.end(), std::default_random_engine(rd()));

        // Randomly generate as many points as scalars
        std::vector<G1> points;
        for (size_t i = 0; i < num_points; ++i) {
            points.emplace_back(G1::random_element());
        }

        // Compute the result of the MSM
        G1 msm_result = points[0] * scalars[0];
        for (size_t i = 1; i < num_points; ++i) {
            msm_result = msm_result + points[i] * scalars[i];
        }

        return { num_points, points, scalars, msm_result };
    }
};

using Curves = ::testing::Types<curve::BN254, curve::Grumpkin>;

TYPED_TEST_SUITE(SortedMsmTests, Curves);

// Test method for a single affine addition with provided slope denominator
TYPED_TEST(SortedMsmTests, AffineAddWithDenominator)
{
    using Curve = TypeParam;
    using G1 = typename Curve::AffineElement;
    using Fq = typename Curve::BaseField;
    using Sorter = MsmSorter<Curve>;

    Sorter msm_sorter;

    G1 point_1 = G1::random_element();
    G1 point_2 = G1::random_element();
    Fq denominator = (point_2.x - point_1.x).invert();

    G1 expected = point_1 + point_2;

    G1 result = msm_sorter.affine_add_with_denominator(point_1, point_2, denominator);

    EXPECT_EQ(result, expected);
}

// Test method for batch computing slope denominators for a set of point addition sequences
TYPED_TEST(SortedMsmTests, ComputePointAdditionDenominators)
{
    using Curve = TypeParam;
    using Fq = typename Curve::BaseField;
    using Sorter = MsmSorter<Curve>;
    using AdditionSequences = typename Sorter::AdditionSequences;

    // Generate random MSM inputs based on a set of sequence counts
    std::array<uint64_t, 2> sequence_counts{ 3, 2 };
    auto test_data = TestFixture::generate_test_data_from_sequence_counts(sequence_counts);
    size_t num_points = test_data.num_points;
    auto& points = test_data.points;

    AdditionSequences addition_sequences{ sequence_counts, test_data.points, {} };

    const size_t num_pairs = 2;
    std::array<Fq, num_pairs> denominators_expected;
    denominators_expected[0] = (points[1].x - points[0].x).invert();
    denominators_expected[1] = (points[4].x - points[3].x).invert();

    Sorter msm_sorter(num_points);
    msm_sorter.batch_compute_point_addition_slope_inverses(addition_sequences);

    for (size_t i = 0; i < num_pairs; ++i) {
        Fq result = msm_sorter.denominators[i];
        Fq expected = denominators_expected[i];
        EXPECT_EQ(result, expected);
    }
}

// Test method for batched addition of point addition sequences in place
TYPED_TEST(SortedMsmTests, BatchedAffineAddInPlace)
{
    using Curve = TypeParam;
    using G1 = typename Curve::AffineElement;
    using Sorter = MsmSorter<Curve>;
    using AdditionSequences = typename Sorter::AdditionSequences;

    // Generate random MSM inputs based on a set of sequence counts
    std::array<uint64_t, 3> sequence_counts{ 5, 2, 3 };
    auto [num_points, points, scalars, msm_result] =
        TestFixture::generate_test_data_from_sequence_counts(sequence_counts);

    AdditionSequences addition_sequences{ sequence_counts, points, {} };

    std::vector<G1> expected_points;
    size_t point_idx = 0;
    for (auto count : sequence_counts) {
        G1 sum = points[point_idx++];
        for (size_t i = 1; i < count; ++i) {
            sum = sum + points[point_idx++];
        }
        expected_points.emplace_back(sum);
    }

    Sorter msm_sorter(num_points);
    msm_sorter.batched_affine_add_in_place(addition_sequences);

    for (size_t idx = 0; idx < expected_points.size(); ++idx) {
        EXPECT_EQ(expected_points[idx], points[idx]);
    }
}

// Test generation of point addition sequences from an arbitrary set of points and scalars
TYPED_TEST(SortedMsmTests, GenerateAdditionSequences)
{
    using Curve = TypeParam;
    using G1 = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;
    using Sorter = MsmSorter<Curve>;
    using AdditionSequences = typename Sorter::AdditionSequences;

    // Generate random MSM inputs based on a set of sequence counts
    std::array<uint64_t, 3> sequence_counts{ 5, 2, 3 };
    auto [num_points, points, scalars, expected_msm_result] =
        TestFixture::generate_test_data_from_sequence_counts(sequence_counts);

    Sorter msm_sorter{ num_points };
    AdditionSequences result = msm_sorter.construct_addition_sequences(scalars, points);

    // The resulting sequence counts should match expectation but only as multisets
    std::multiset<Fr> expected_sequence_counts(sequence_counts.begin(), sequence_counts.end());
    std::multiset<Fr> result_sequence_counts(result.sequence_counts.begin(), result.sequence_counts.end());
    EXPECT_EQ(expected_sequence_counts, result_sequence_counts);

    // The result points will be sorted but should match the original as multisets
    std::multiset<G1> expected_points(points.begin(), points.end());
    std::multiset<G1> result_points(result.points.begin(), result.points.end());
    EXPECT_EQ(expected_points, result_points);

    G1 msm_result;
    msm_result.self_set_infinity();
    size_t scalar_idx = 0;
    size_t point_idx = 0;
    for (auto count : result.sequence_counts) {
        for (size_t i = 0; i < count; ++i) {
            msm_result = msm_result + result.points[point_idx] * msm_sorter.unique_scalars[scalar_idx];
            point_idx++;
        }
        scalar_idx++;
    }

    EXPECT_EQ(msm_result, expected_msm_result);
}

// Test that the method reduce_msm_inputs can reduce a set of {points, scalars} with duplicate scalars to a reduced set
// of inputs {points', scalars'} such that all scalars in scalars' are unique and that perfoming the MSM on the reduced
// inputs yields the same result as with the original inputs
TYPED_TEST(SortedMsmTests, ReduceMsmInputsSimple)
{
    using Curve = TypeParam;
    using G1 = typename Curve::AffineElement;
    using Sorter = MsmSorter<Curve>;

    // Generate random MSM inputs based on a set of sequence counts
    std::array<uint64_t, 3> sequence_counts{ 5, 2, 3 };
    auto [num_points, points, scalars, expected_msm_result] =
        TestFixture::generate_test_data_from_sequence_counts(sequence_counts);

    Sorter msm_sorter{ num_points };
    auto [result_scalars, result_points] = msm_sorter.reduce_msm_inputs(scalars, points);

    G1 msm_result = result_points[0] * result_scalars[0];
    for (size_t i = 1; i < result_points.size(); ++i) {
        msm_result = msm_result + result_points[i] * result_scalars[i];
    }

    EXPECT_EQ(msm_result, expected_msm_result);
}

// Test that the method reduce_msm_inputs can reduce a set of {points, scalars} with duplicate scalars to a reduced set
// of inputs {points', scalars'} such that all scalars in scalars' are unique and that perfoming the MSM on the reduced
// inputs yields the same result as with the original inputs
TYPED_TEST(SortedMsmTests, ReduceMsmInputs)
{
    using Curve = TypeParam;
    using G1 = typename Curve::AffineElement;
    using Sorter = MsmSorter<Curve>;

    // Generate random MSM inputs based on a set of sequence counts
    const size_t num_unique_scalars = 5;
    std::array<uint64_t, num_unique_scalars> sequence_counts{ 75, 1, 28, 382, 3 };
    auto [num_points, points, scalars, expected_msm_result] =
        TestFixture::generate_test_data_from_sequence_counts(sequence_counts);

    Sorter msm_sorter{ num_points };
    auto [result_scalars, result_points] = msm_sorter.reduce_msm_inputs(scalars, points);

    // Points and scalars should both be reduced to the number of unique scalars
    EXPECT_EQ(result_scalars.size(), num_unique_scalars);
    EXPECT_EQ(result_points.size(), num_unique_scalars);

    // Performing the MSM over the reduced inputs should yield the same result as the original
    G1 msm_result = result_points[0] * result_scalars[0];
    for (size_t i = 1; i < result_points.size(); ++i) {
        msm_result = msm_result + result_points[i] * result_scalars[i];
    }
    EXPECT_EQ(msm_result, expected_msm_result);
}
} // namespace bb