#include "barretenberg/common/thread.hpp"
#include "barretenberg/vm/avm/generated/circuit_builder.hpp"
#include "barretenberg/vm/avm/generated/flavor.hpp"
#include "barretenberg/vm/avm/generated/full_row.hpp"
#include "barretenberg/vm/avm/trace/fixed_powers.hpp"

#include "barretenberg/vm/avm/trace/gadgets/range_check.hpp"

#include <cstdint>
#include <gtest/gtest.h>
#include <memory>
#include <vector>

namespace tests_avm {

using namespace bb;
using namespace bb::Avm_vm;

TEST(AvmRangeCheck, shouldRangeCheck)
{
    using FF = AvmFlavor::FF;
    constexpr size_t TRACE_SIZE = 1 << 16;

    std::vector<AvmFullRow<FF>> trace(TRACE_SIZE);

    bb::avm_trace::AvmRangeCheckBuilder range_check_builder;
    std::cerr << "Generating trace of size " << TRACE_SIZE << "..." << std::endl;

    // Do a bunch of range checks (clk does not matter here so we just have it as 0)
    range_check_builder.assert_range(0, 0, EventEmitter::ALU, 0);
    range_check_builder.assert_range(0, 1, EventEmitter::ALU, 0);
    range_check_builder.assert_range(0, 16, EventEmitter::ALU, 0);
    range_check_builder.assert_range(2, 2, EventEmitter::ALU, 0);
    range_check_builder.assert_range(255, 8, EventEmitter::ALU, 0);
    range_check_builder.assert_range(1 << 16, 17, EventEmitter::ALU, 0);
    range_check_builder.assert_range(1 << 18, 32, EventEmitter::ALU, 0);
    range_check_builder.assert_range(uint128_t(1) << 66, 67, EventEmitter::ALU, 0);
    range_check_builder.assert_range(1024, 109, EventEmitter::ALU, 0);
    range_check_builder.assert_range(1, 128, EventEmitter::ALU, 0);

    auto finalised_builder = range_check_builder.finalize();
    for (size_t i = 0; i < finalised_builder.size(); i++) {
        range_check_builder.merge_into(trace[i], finalised_builder[i]);
    }

    for (size_t i = 0; i < TRACE_SIZE; i++) {
        // Standard clk, range_check and powers_of_2
        trace[i].main_clk = i;
        if (i <= UINT8_MAX) {
            trace[i].main_sel_rng_8 = FF(1);
            merge_into(trace[i], bb::avm_trace::FixedPowersTable::get().at(i));
        }
        trace[i].main_sel_rng_16 = FF(1);

        // Put counts in the right place
        trace[i].lookup_rng_chk_0_counts = range_check_builder.u16_range_chk_counters[0][uint16_t(i)];
        trace[i].lookup_rng_chk_1_counts = range_check_builder.u16_range_chk_counters[1][uint16_t(i)];
        trace[i].lookup_rng_chk_2_counts = range_check_builder.u16_range_chk_counters[2][uint16_t(i)];
        trace[i].lookup_rng_chk_3_counts = range_check_builder.u16_range_chk_counters[3][uint16_t(i)];
        trace[i].lookup_rng_chk_4_counts = range_check_builder.u16_range_chk_counters[4][uint16_t(i)];
        trace[i].lookup_rng_chk_5_counts = range_check_builder.u16_range_chk_counters[5][uint16_t(i)];
        trace[i].lookup_rng_chk_6_counts = range_check_builder.u16_range_chk_counters[6][uint16_t(i)];
        trace[i].lookup_rng_chk_7_counts = range_check_builder.u16_range_chk_counters[7][uint16_t(i)];
        trace[i].lookup_rng_chk_diff_counts = range_check_builder.dyn_diff_counts[uint16_t(i)];
        trace[i].lookup_rng_chk_pow_2_counts = range_check_builder.powers_of_2_counts[uint8_t(i)];
    }
    std::cerr << "Done generating trace..." << std::endl;

    // We build the polynomials needed to run "sumcheck".
    AvmCircuitBuilder cb;
    cb.set_trace(std::move(trace));
    auto polys = cb.compute_polynomials();
    const size_t num_rows = polys.get_polynomial_size();
    std::cerr << "Done computing polynomials..." << std::endl;

    std::cerr << "Accumulating relations..." << std::endl;
    using Relation = Avm_vm::range_check<FF>;

    typename Relation::SumcheckArrayOfValuesOverSubrelations result;
    for (auto& r : result) {
        r = 0;
    }

    // We set the conditions up there.
    for (size_t r = 0; r < num_rows; ++r) {
        Relation::accumulate(result, polys.get_row(r), {}, 1);
    }

    for (size_t j = 0; j < result.size(); ++j) {
        if (result[j] != 0) {
            EXPECT_EQ(result[j], 0) << "Relation " << Relation::NAME << " subrelation "
                                    << Relation::get_subrelation_label(j) << " was expected to be zero.";
        }
    }
    std::cerr << "Accumulating lookup relations..." << std::endl;

    // Let's be explicit about the lookups we are checking
    using AllLookupRelations = std::tuple<
        // Lookups
        lookup_rng_chk_0_relation<FF>,
        lookup_rng_chk_1_relation<FF>,
        lookup_rng_chk_2_relation<FF>,
        lookup_rng_chk_3_relation<FF>,
        lookup_rng_chk_4_relation<FF>,
        lookup_rng_chk_5_relation<FF>,
        lookup_rng_chk_6_relation<FF>,
        lookup_rng_chk_7_relation<FF>,
        lookup_rng_chk_pow_2_relation<FF>,
        lookup_rng_chk_diff_relation<FF>>;

    const FF gamma = FF::random_element();
    const FF beta = FF::random_element();
    bb::RelationParameters<typename AvmFlavor::FF> params{
        .beta = beta,
        .gamma = gamma,
    };
    bb::constexpr_for<0, std::tuple_size_v<AllLookupRelations>, 1>([&]<size_t i>() {
        using LookupRelations = std::tuple_element_t<i, AllLookupRelations>;

        // Check the logderivative relation
        bb::compute_logderivative_inverse<AvmFlavor, LookupRelations>(polys, params, num_rows);

        typename LookupRelations::SumcheckArrayOfValuesOverSubrelations lookup_result;

        for (auto& r : lookup_result) {
            r = 0;
        }
        for (size_t r = 0; r < num_rows; ++r) {
            LookupRelations::accumulate(lookup_result, polys.get_row(r), params, 1);
        }
        for (const auto& j : lookup_result) {
            if (j != 0) {
                EXPECT_EQ(j, 0) << "Lookup Relation " << LookupRelations::NAME << " subrelation ";
            }
        }
    });
    std::cerr << "Relations accumulated..." << std::endl;
}

} // namespace tests_avm
