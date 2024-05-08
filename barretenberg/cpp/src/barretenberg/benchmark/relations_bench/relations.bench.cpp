#include "barretenberg/eccvm/eccvm_flavor.hpp"
#include "barretenberg/protogalaxy/protogalaxy_prover.hpp"
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"
#include "barretenberg/translator_vm/goblin_translator_flavor.hpp"
#include <benchmark/benchmark.h>

namespace {
auto& engine = bb::numeric::get_debug_randomness();
}

namespace bb::benchmark::relations {

using Fr = bb::fr;
using Fq = grumpkin::fr;

// Generic helper for executing Relation::accumulate for the template specified input type
template <typename Flavor, typename Relation, typename Input, typename Accumulator>
void execute_relation(::benchmark::State& state)
{
    using FF = typename Flavor::FF;

    auto params = bb::RelationParameters<FF>::get_random();

    // Instantiate zero-initialized inputs and accumulator
    Input input{};
    Accumulator accumulator;

    for (auto _ : state) {
        Relation::accumulate(accumulator, input, params, 1);
    }
}

// Single execution of relation on values (FF), e.g. Sumcheck verifier / PG perturbator work
template <typename Flavor, typename Relation> void execute_relation_for_values(::benchmark::State& state)
{
    using Input = typename Flavor::AllValues;
    using Accumulator = typename Relation::SumcheckArrayOfValuesOverSubrelations;

    execute_relation<Flavor, Relation, Input, Accumulator>(state);
}

// Single execution of relation on Sumcheck univariates, i.e. Sumcheck/Decider prover work
template <typename Flavor, typename Relation> void execute_relation_for_univariates(::benchmark::State& state)
{
    using Input = typename Flavor::ExtendedEdges;
    using Accumulator = typename Relation::SumcheckTupleOfUnivariatesOverSubrelations;

    execute_relation<Flavor, Relation, Input, Accumulator>(state);
}

// Single execution of relation on PG univariates, i.e. PG combiner work
template <typename Flavor, typename Relation> void execute_relation_for_pg_univariates(::benchmark::State& state)
{
    using ProverInstances = ProverInstances_<Flavor>;
    using ProtoGalaxyProver = ProtoGalaxyProver_<ProverInstances>;
    using Input = ProtoGalaxyProver::ExtendedUnivariates;
    using Accumulator = typename Relation::template ProtogalaxyTupleOfUnivariatesOverSubrelations<ProverInstances::NUM>;

    execute_relation<Flavor, Relation, Input, Accumulator>(state);
}

// Ultra relations (PG prover combiner work)
BENCHMARK(execute_relation_for_pg_univariates<UltraFlavor, UltraArithmeticRelation<Fr>>);
BENCHMARK(execute_relation_for_pg_univariates<UltraFlavor, DeltaRangeConstraintRelation<Fr>>);
BENCHMARK(execute_relation_for_pg_univariates<UltraFlavor, EllipticRelation<Fr>>);
BENCHMARK(execute_relation_for_pg_univariates<UltraFlavor, AuxiliaryRelation<Fr>>);
BENCHMARK(execute_relation_for_pg_univariates<UltraFlavor, LookupRelation<Fr>>);
BENCHMARK(execute_relation_for_pg_univariates<UltraFlavor, UltraPermutationRelation<Fr>>);

// Goblin-Ultra only relations (PG prover combiner work)
BENCHMARK(execute_relation_for_pg_univariates<GoblinUltraFlavor, EccOpQueueRelation<Fr>>);
BENCHMARK(execute_relation_for_pg_univariates<GoblinUltraFlavor, DatabusLookupRelation<Fr>>);
BENCHMARK(execute_relation_for_pg_univariates<GoblinUltraFlavor, Poseidon2ExternalRelation<Fr>>);
BENCHMARK(execute_relation_for_pg_univariates<GoblinUltraFlavor, Poseidon2InternalRelation<Fr>>);

// Ultra relations (Sumcheck prover work)
BENCHMARK(execute_relation_for_univariates<UltraFlavor, UltraArithmeticRelation<Fr>>);
BENCHMARK(execute_relation_for_univariates<UltraFlavor, DeltaRangeConstraintRelation<Fr>>);
BENCHMARK(execute_relation_for_univariates<UltraFlavor, EllipticRelation<Fr>>);
BENCHMARK(execute_relation_for_univariates<UltraFlavor, AuxiliaryRelation<Fr>>);
BENCHMARK(execute_relation_for_univariates<UltraFlavor, LookupRelation<Fr>>);
BENCHMARK(execute_relation_for_univariates<UltraFlavor, UltraPermutationRelation<Fr>>);

// Goblin-Ultra only relations (Sumcheck prover work)
BENCHMARK(execute_relation_for_univariates<GoblinUltraFlavor, EccOpQueueRelation<Fr>>);
BENCHMARK(execute_relation_for_univariates<GoblinUltraFlavor, DatabusLookupRelation<Fr>>);
BENCHMARK(execute_relation_for_univariates<GoblinUltraFlavor, Poseidon2ExternalRelation<Fr>>);
BENCHMARK(execute_relation_for_univariates<GoblinUltraFlavor, Poseidon2InternalRelation<Fr>>);

// Ultra relations (verifier work)
BENCHMARK(execute_relation_for_values<UltraFlavor, UltraArithmeticRelation<Fr>>);
BENCHMARK(execute_relation_for_values<UltraFlavor, DeltaRangeConstraintRelation<Fr>>);
BENCHMARK(execute_relation_for_values<UltraFlavor, EllipticRelation<Fr>>);
BENCHMARK(execute_relation_for_values<UltraFlavor, AuxiliaryRelation<Fr>>);
BENCHMARK(execute_relation_for_values<UltraFlavor, LookupRelation<Fr>>);
BENCHMARK(execute_relation_for_values<UltraFlavor, UltraPermutationRelation<Fr>>);

// Goblin-Ultra only relations (verifier work)
BENCHMARK(execute_relation_for_values<GoblinUltraFlavor, EccOpQueueRelation<Fr>>);
BENCHMARK(execute_relation_for_values<GoblinUltraFlavor, DatabusLookupRelation<Fr>>);
BENCHMARK(execute_relation_for_values<GoblinUltraFlavor, Poseidon2ExternalRelation<Fr>>);
BENCHMARK(execute_relation_for_values<GoblinUltraFlavor, Poseidon2InternalRelation<Fr>>);

// Translator VM
BENCHMARK(execute_relation_for_values<GoblinTranslatorFlavor, GoblinTranslatorDecompositionRelation<Fr>>);
BENCHMARK(execute_relation_for_values<GoblinTranslatorFlavor, GoblinTranslatorOpcodeConstraintRelation<Fr>>);
BENCHMARK(execute_relation_for_values<GoblinTranslatorFlavor, GoblinTranslatorAccumulatorTransferRelation<Fr>>);
BENCHMARK(execute_relation_for_values<GoblinTranslatorFlavor, GoblinTranslatorDeltaRangeConstraintRelation<Fr>>);
BENCHMARK(execute_relation_for_values<GoblinTranslatorFlavor, GoblinTranslatorNonNativeFieldRelation<Fr>>);
BENCHMARK(execute_relation_for_values<GoblinTranslatorFlavor, GoblinTranslatorPermutationRelation<Fr>>);

// ECCVM
BENCHMARK(execute_relation_for_values<ECCVMFlavor, ECCVMLookupRelation<Fq>>);
BENCHMARK(execute_relation_for_values<ECCVMFlavor, ECCVMMSMRelation<Fq>>);
BENCHMARK(execute_relation_for_values<ECCVMFlavor, ECCVMPointTableRelation<Fq>>);
BENCHMARK(execute_relation_for_values<ECCVMFlavor, ECCVMSetRelation<Fq>>);
BENCHMARK(execute_relation_for_values<ECCVMFlavor, ECCVMTranscriptRelation<Fq>>);
BENCHMARK(execute_relation_for_values<ECCVMFlavor, ECCVMWnafRelation<Fq>>);

} // namespace bb::benchmark::relations

BENCHMARK_MAIN();