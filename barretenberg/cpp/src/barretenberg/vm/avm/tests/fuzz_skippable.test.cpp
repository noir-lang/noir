#include "barretenberg/common/thread.hpp"
#include "barretenberg/vm/avm/generated/circuit_builder.hpp"
#include "barretenberg/vm/avm/generated/flavor.hpp"
#include "barretenberg/vm/avm/generated/full_row.hpp"

#include <gtest/gtest.h>
#include <memory>
#include <vector>

namespace tests_avm {

using namespace bb;
using namespace bb::Avm_vm;

TEST(AvmSkippableTests, shouldSkipCorrectly)
{
    using FF = AvmFlavor::FF;
    constexpr size_t TRACE_SIZE = 1 << 15;

    std::vector<AvmFullRow<FF>> trace(TRACE_SIZE);
    std::cerr << "Generating trace of size " << TRACE_SIZE << "..." << std::endl;
    // This is the most time consuming part of this test!
    // In particular, the generation of random fields.
    bb::parallel_for(trace.size(), [&](size_t i) {
        // The first row needs to be zeroes otherwise shifting doesn't work.
        if (i == 0) {
            return;
        }
        AvmFullRow<FF>& row = trace[i];

        // Fill the row with random values.
        auto as_vector = row.as_vector();
        const auto as_vector_size = as_vector.size();
        for (size_t j = 0; j < as_vector_size; j++) {
            // FF::random_element(); is so slow! Using std::rand() instead.
            const_cast<FF&>(as_vector[j]) = FF(std::rand());
        }

        // Set the conditions for skippable to return true.
        row.poseidon2_sel_poseidon_perm = 0;
    });
    std::cerr << "Done generating trace..." << std::endl;

    // We build the polynomials needed to run "sumcheck".
    AvmCircuitBuilder cb;
    cb.set_trace(std::move(trace));
    auto polys = cb.compute_polynomials();
    std::cerr << "Done computing polynomials..." << std::endl;

    // For each skippable relation we will check:
    // 1. That Relation::skippable returns true (i.e., we correctly set the conditions)
    // 2. That the sumcheck result is zero (i.e., it was ok to skip the relation)
    for (size_t ri = 1; ri < TRACE_SIZE; ++ri) {
        auto row = polys.get_row(ri);

        bb::constexpr_for<0, std::tuple_size_v<AvmFlavor::Relations>, 1>([&]<size_t i>() {
            using Relation = std::tuple_element_t<i, AvmFlavor::Relations>;

            // We only want to test skippable relations.
            if constexpr (isSkippable<Relation, AvmFullRow<FF>>) {
                typename Relation::SumcheckArrayOfValuesOverSubrelations result;
                for (auto& r : result) {
                    r = 0;
                }

                // We set the conditions up there.
                auto skip = Relation::skip(row);
                EXPECT_TRUE(skip) << "Relation " << Relation::NAME << " was expected to be skippable at row " << ri
                                  << ".";

                Relation::accumulate(result, row, {}, 1);

                // If the relation is skippable, the result should be zero.
                for (size_t j = 0; j < result.size(); ++j) {
                    if (result[j] != 0) {
                        EXPECT_EQ(result[j], 0)
                            << "Relation " << Relation::NAME << " subrelation " << j << " was expected to be zero.";
                        GTEST_SKIP();
                    }
                }
            }
        });
    }
}

} // namespace tests_avm