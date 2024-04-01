
#include "prover_instance.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_library.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/srs/factories/file_crs_factory.hpp"
#include <gtest/gtest.h>
using namespace bb;

template <class Flavor> class InstanceTests : public testing::Test {
    using FF = typename Flavor::FF;
    using Polynomial = bb::Polynomial<FF>;
    using Builder = typename Flavor::CircuitBuilder;

  public:
    /**
     * @brief Get a random polynomial
     *
     * @param size
     * @return Polynomial
     */
    static constexpr Polynomial get_random_polynomial(size_t size)
    {
        Polynomial random_polynomial{ size };
        for (auto& coeff : random_polynomial) {
            coeff = FF::random_element();
        }
        return random_polynomial;
    }

    static void populate_span(auto& polynomial_view, const auto& polynomial)
    {
        ASSERT(polynomial_view.size() <= polynomial.size());
        for (size_t idx = 0; idx < polynomial.size(); idx++) {
            polynomial_view[idx] = polynomial[idx];
        }
    };
    /**
     * @brief Check consistency of the computation of the sorted list accumulator
     * @details This test compares a simple, unoptimized, easily readable calculation of the sorted list accumulator
     * to the optimized implementation used by the prover. It's purpose is to provide confidence that some optimization
     * introduced into the calculation has not changed the result.
     * @note This test does confirm the correctness of the sorted list accumulator, only that the two implementations
     * yield an identical result.
     */
    static void test_sorted_list_accumulator_construction()
    {
        srs::init_crs_factory("../srs_db/ignition");

        // Construct a simple circuit of size n = 8 (i.e. the minimum circuit size)
        Builder builder;

        auto a = 2;
        builder.add_variable(a);

        builder.add_gates_to_ensure_all_polys_are_non_zero();
        builder.finalize_circuit();
        auto instance = ProverInstance_<Flavor>(builder);

        // Get random challenge eta
        auto eta = FF::random_element();
        auto eta_two = FF::random_element();
        auto eta_three = FF::random_element();

        auto sorted_list_polynomials = instance.proving_key.sorted_polynomials;

        // Method 1: computed sorted list accumulator polynomial using prover library method
        instance.proving_key.compute_sorted_list_accumulator(eta, eta_two, eta_three);
        auto sorted_list_accumulator = instance.proving_key.sorted_accum;

        // Compute s = s_1 + η*s_2 + η²*s_3 + η³*s_4
        Polynomial sorted_list_accumulator_expected{ sorted_list_polynomials[0] };
        for (size_t i = 0; i < instance.proving_key.circuit_size; ++i) {
            sorted_list_accumulator_expected[i] += sorted_list_polynomials[1][i] * eta +
                                                   sorted_list_polynomials[2][i] * eta_two +
                                                   sorted_list_polynomials[3][i] * eta_three;
        }

        EXPECT_EQ(sorted_list_accumulator, sorted_list_accumulator_expected);
    };
};

using FlavorTypes = testing::Types<UltraFlavor, GoblinUltraFlavor>;
TYPED_TEST_SUITE(InstanceTests, FlavorTypes);

TYPED_TEST(InstanceTests, SortedListAccumulator)
{
    TestFixture::test_sorted_list_accumulator_construction();
}
