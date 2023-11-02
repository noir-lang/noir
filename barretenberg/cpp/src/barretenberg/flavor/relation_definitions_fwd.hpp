#pragma once

#include "barretenberg/proof_system/relations/relation_types.hpp"

#define ExtendedEdge(Flavor) Flavor::ExtendedEdges
#define EvaluationEdge(Flavor) Flavor::AllValues
#define EntityEdge(Flavor) Flavor::AllEntities<Flavor::FF, Flavor::FF>

#define ACCUMULATE(...) _ACCUMULATE(__VA_ARGS__)
#define _ACCUMULATE(Preface, RelationBase, Flavor, AccumulatorType, EdgeType)                                          \
    Preface template void                                                                                              \
    RelationBase<Flavor::FF>::accumulate<proof_system::Relation<RelationBase<Flavor::FF>>::AccumulatorType,            \
                                         EdgeType(Flavor)>(                                                            \
        proof_system::Relation<RelationBase<Flavor::FF>>::AccumulatorType&,                                            \
        EdgeType(Flavor) const&,                                                                                       \
        RelationParameters<Flavor::FF> const&,                                                                         \
        Flavor::FF const&);

#define PERMUTATION_METHOD(...) _PERMUTATION_METHOD(__VA_ARGS__)
#define _PERMUTATION_METHOD(Preface, MethodName, RelationBase, Flavor, AccumulatorType, EdgeType)                      \
    Preface template typename proof_system::Relation<RelationBase<Flavor::FF>>::AccumulatorType                        \
    RelationBase<Flavor::FF>::MethodName<proof_system::Relation<RelationBase<Flavor::FF>>::AccumulatorType,            \
                                         EdgeType(Flavor)>(EdgeType(Flavor) const&,                                    \
                                                           RelationParameters<Flavor::FF> const&);

#define SUMCHECK_RELATION_CLASS(...) _SUMCHECK_RELATION_CLASS(__VA_ARGS__)
#define _SUMCHECK_RELATION_CLASS(Preface, RelationBase, Flavor)                                                        \
    ACCUMULATE(Preface, RelationBase, Flavor, SumcheckTupleOfUnivariatesOverSubrelations, ExtendedEdge)                \
    ACCUMULATE(Preface, RelationBase, Flavor, SumcheckArrayOfValuesOverSubrelations, EvaluationEdge)                   \
    ACCUMULATE(Preface, RelationBase, Flavor, SumcheckArrayOfValuesOverSubrelations, EntityEdge)

#define DECLARE_SUMCHECK_RELATION_CLASS(RelationBase, Flavor) SUMCHECK_RELATION_CLASS(extern, RelationBase, Flavor)
#define DEFINE_SUMCHECK_RELATION_CLASS(RelationBase, Flavor) SUMCHECK_RELATION_CLASS(, RelationBase, Flavor)

#define SUMCHECK_PERMUTATION_CLASS(...) _SUMCHECK_PERMUTATION_CLASS(__VA_ARGS__)
#define _SUMCHECK_PERMUTATION_CLASS(Preface, RelationBase, Flavor)                                                     \
    PERMUTATION_METHOD(                                                                                                \
        Preface, compute_permutation_numerator, RelationBase, Flavor, UnivariateAccumulator0, ExtendedEdge)            \
    PERMUTATION_METHOD(                                                                                                \
        Preface, compute_permutation_numerator, RelationBase, Flavor, ValueAccumulator0, EvaluationEdge)               \
    PERMUTATION_METHOD(Preface, compute_permutation_numerator, RelationBase, Flavor, ValueAccumulator0, EntityEdge)    \
    PERMUTATION_METHOD(                                                                                                \
        Preface, compute_permutation_denominator, RelationBase, Flavor, UnivariateAccumulator0, ExtendedEdge)          \
    PERMUTATION_METHOD(                                                                                                \
        Preface, compute_permutation_denominator, RelationBase, Flavor, ValueAccumulator0, EvaluationEdge)             \
    PERMUTATION_METHOD(Preface, compute_permutation_denominator, RelationBase, Flavor, ValueAccumulator0, EntityEdge)

#define DECLARE_SUMCHECK_PERMUTATION_CLASS(RelationBase, Flavor)                                                       \
    SUMCHECK_PERMUTATION_CLASS(extern, RelationBase, Flavor)
#define DEFINE_SUMCHECK_PERMUTATION_CLASS(RelationBase, Flavor) SUMCHECK_PERMUTATION_CLASS(, RelationBase, Flavor)
