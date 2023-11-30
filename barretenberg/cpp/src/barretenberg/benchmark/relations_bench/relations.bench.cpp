#include "barretenberg/flavor/ecc_vm.hpp"
#include "barretenberg/flavor/goblin_translator.hpp"
#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include <benchmark/benchmark.h>

namespace {
auto& engine = numeric::random::get_debug_engine();
}

using namespace proof_system::honk::sumcheck;

namespace proof_system::benchmark::relations {

using Fr = barretenberg::fr;
using Fq = grumpkin::fr;

template <typename Flavor, typename Relation> void execute_relation(::benchmark::State& state)
{
    using FF = typename Flavor::FF;
    using AllValues = typename Flavor::AllValues;
    using SumcheckArrayOfValuesOverSubrelations = typename Relation::SumcheckArrayOfValuesOverSubrelations;

    auto params = proof_system::RelationParameters<FF>::get_random();

    // Extract an array containing all the polynomial evaluations at a given row i
    AllValues new_value{};
    // Define the appropriate SumcheckArrayOfValuesOverSubrelations type for this relation and initialize to zero
    SumcheckArrayOfValuesOverSubrelations accumulator;
    // Evaluate each constraint in the relation and check that each is satisfied

    for (auto _ : state) {
        Relation::accumulate(accumulator, new_value, params, 1);
    }
}

void ultra_auxiliary_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::Ultra, AuxiliaryRelation<Fr>>(state);
}
BENCHMARK(ultra_auxiliary_relation);

void ultra_elliptic_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::Ultra, EllipticRelation<Fr>>(state);
}
BENCHMARK(ultra_elliptic_relation);

void ultra_ecc_op_queue_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::GoblinUltra, EccOpQueueRelation<Fr>>(state);
}
BENCHMARK(ultra_ecc_op_queue_relation);

void ultra_gen_perm_sort_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::Ultra, GenPermSortRelation<Fr>>(state);
}
BENCHMARK(ultra_gen_perm_sort_relation);

void ultralookup_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::Ultra, LookupRelation<Fr>>(state);
}
BENCHMARK(ultralookup_relation);

void ultra_permutation_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::Ultra, UltraPermutationRelation<Fr>>(state);
}
BENCHMARK(ultra_permutation_relation);

void ultra_arithmetic_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::Ultra, UltraArithmeticRelation<Fr>>(state);
}
BENCHMARK(ultra_arithmetic_relation);

void translator_decomposition_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::GoblinTranslator, GoblinTranslatorDecompositionRelation<Fr>>(state);
}
BENCHMARK(translator_decomposition_relation);

void translator_opcode_constraint_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::GoblinTranslator, GoblinTranslatorOpcodeConstraintRelation<Fr>>(state);
}
BENCHMARK(translator_opcode_constraint_relation);

void translator_accumulator_transfer_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::GoblinTranslator, GoblinTranslatorAccumulatorTransferRelation<Fr>>(state);
}
BENCHMARK(translator_accumulator_transfer_relation);

void translator_gen_perm_sort_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::GoblinTranslator, GoblinTranslatorGenPermSortRelation<Fr>>(state);
}
BENCHMARK(translator_gen_perm_sort_relation);

void translator_non_native_field_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::GoblinTranslator, GoblinTranslatorNonNativeFieldRelation<Fr>>(state);
}
BENCHMARK(translator_non_native_field_relation);

void translator_permutation_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::GoblinTranslator, GoblinTranslatorPermutationRelation<Fr>>(state);
}
BENCHMARK(translator_permutation_relation);

void eccvm_lookup_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::ECCVM, ECCVMLookupRelation<Fq>>(state);
}
BENCHMARK(eccvm_lookup_relation);

void eccvm_msm_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::ECCVM, ECCVMMSMRelation<Fq>>(state);
}
BENCHMARK(eccvm_msm_relation);

void eccvm_point_table_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::ECCVM, ECCVMPointTableRelation<Fq>>(state);
}
BENCHMARK(eccvm_point_table_relation);

void eccvm_set_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::ECCVM, ECCVMSetRelation<Fq>>(state);
}
BENCHMARK(eccvm_set_relation);

void eccvm_transcript_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::ECCVM, ECCVMTranscriptRelation<Fq>>(state);
}
BENCHMARK(eccvm_transcript_relation);

void eccvm_wnaf_relation(::benchmark::State& state) noexcept
{
    execute_relation<honk::flavor::ECCVM, ECCVMWnafRelation<Fq>>(state);
}
BENCHMARK(eccvm_wnaf_relation);

} // namespace proof_system::benchmark::relations
