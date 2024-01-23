#include "barretenberg/flavor/ecc_vm.hpp"
#include "barretenberg/flavor/goblin_translator.hpp"
#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include <benchmark/benchmark.h>

namespace {
auto& engine = bb::numeric::get_debug_randomness();
}

using namespace bb::honk::sumcheck;

namespace bb::benchmark::relations {

using Fr = bb::fr;
using Fq = grumpkin::fr;

template <typename Flavor, typename Relation> void execute_relation(::benchmark::State& state)
{
    using FF = typename Flavor::FF;
    using AllValues = typename Flavor::AllValues;
    using SumcheckArrayOfValuesOverSubrelations = typename Relation::SumcheckArrayOfValuesOverSubrelations;

    auto params = bb::RelationParameters<FF>::get_random();

    // Extract an array containing all the polynomial evaluations at a given row i
    AllValues new_value{};
    // Define the appropriate SumcheckArrayOfValuesOverSubrelations type for this relation and initialize to zero
    SumcheckArrayOfValuesOverSubrelations accumulator;
    // Evaluate each constraint in the relation and check that each is satisfied

    for (auto _ : state) {
        Relation::accumulate(accumulator, new_value, params, 1);
    }
}
BENCHMARK(execute_relation<honk::flavor::Ultra, UltraArithmeticRelation<Fr>>);
BENCHMARK(execute_relation<honk::flavor::Ultra, GenPermSortRelation<Fr>>);
BENCHMARK(execute_relation<honk::flavor::Ultra, EllipticRelation<Fr>>);
BENCHMARK(execute_relation<honk::flavor::Ultra, AuxiliaryRelation<Fr>>);
BENCHMARK(execute_relation<honk::flavor::Ultra, LookupRelation<Fr>>);
BENCHMARK(execute_relation<honk::flavor::Ultra, UltraPermutationRelation<Fr>>);

BENCHMARK(execute_relation<honk::flavor::GoblinUltra, EccOpQueueRelation<Fr>>);

BENCHMARK(execute_relation<honk::flavor::GoblinTranslator, GoblinTranslatorDecompositionRelation<Fr>>);
BENCHMARK(execute_relation<honk::flavor::GoblinTranslator, GoblinTranslatorOpcodeConstraintRelation<Fr>>);
BENCHMARK(execute_relation<honk::flavor::GoblinTranslator, GoblinTranslatorAccumulatorTransferRelation<Fr>>);
BENCHMARK(execute_relation<honk::flavor::GoblinTranslator, GoblinTranslatorGenPermSortRelation<Fr>>);
BENCHMARK(execute_relation<honk::flavor::GoblinTranslator, GoblinTranslatorNonNativeFieldRelation<Fr>>);
BENCHMARK(execute_relation<honk::flavor::GoblinTranslator, GoblinTranslatorPermutationRelation<Fr>>);

BENCHMARK(execute_relation<honk::flavor::ECCVM, ECCVMLookupRelation<Fq>>);
BENCHMARK(execute_relation<honk::flavor::ECCVM, ECCVMMSMRelation<Fq>>);
BENCHMARK(execute_relation<honk::flavor::ECCVM, ECCVMPointTableRelation<Fq>>);
BENCHMARK(execute_relation<honk::flavor::ECCVM, ECCVMSetRelation<Fq>>);
BENCHMARK(execute_relation<honk::flavor::ECCVM, ECCVMTranscriptRelation<Fq>>);
BENCHMARK(execute_relation<honk::flavor::ECCVM, ECCVMWnafRelation<Fq>>);

} // namespace bb::benchmark::relations
