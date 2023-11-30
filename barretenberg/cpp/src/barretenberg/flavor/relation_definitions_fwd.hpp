#pragma once

#include "barretenberg/relations/relation_types.hpp"

#define ExtendedEdge(Flavor) Flavor::ExtendedEdges
#define EvaluationEdge(Flavor) Flavor::AllValues
#define EntityEdge(Flavor) Flavor::AllEntities<Flavor::FF>

#define ACCUMULATE(...) _ACCUMULATE(__VA_ARGS__)
#define _ACCUMULATE(Preface, RelationImpl, Flavor, AccumulatorType, EdgeType)                                          \
    Preface template void                                                                                              \
    RelationImpl<Flavor::FF>::accumulate<proof_system::Relation<RelationImpl<Flavor::FF>>::AccumulatorType,            \
                                         EdgeType(Flavor)>(                                                            \
        proof_system::Relation<RelationImpl<Flavor::FF>>::AccumulatorType&,                                            \
        EdgeType(Flavor) const&,                                                                                       \
        RelationParameters<Flavor::FF> const&,                                                                         \
        Flavor::FF const&);

#define PERMUTATION_METHOD(...) _PERMUTATION_METHOD(__VA_ARGS__)
#define _PERMUTATION_METHOD(Preface, MethodName, RelationImpl, Flavor, AccumulatorType, EdgeType)                      \
    Preface template typename proof_system::Relation<RelationImpl<Flavor::FF>>::AccumulatorType                        \
    RelationImpl<Flavor::FF>::MethodName<proof_system::Relation<RelationImpl<Flavor::FF>>::AccumulatorType,            \
                                         EdgeType(Flavor)>(EdgeType(Flavor) const&,                                    \
                                                           RelationParameters<Flavor::FF> const&);

#define SUMCHECK_RELATION_CLASS(...) _SUMCHECK_RELATION_CLASS(__VA_ARGS__)
#define _SUMCHECK_RELATION_CLASS(Preface, RelationImpl, Flavor)                                                        \
    ACCUMULATE(Preface, RelationImpl, Flavor, SumcheckTupleOfUnivariatesOverSubrelations, ExtendedEdge)                \
    ACCUMULATE(Preface, RelationImpl, Flavor, SumcheckArrayOfValuesOverSubrelations, EvaluationEdge)                   \
    ACCUMULATE(Preface, RelationImpl, Flavor, SumcheckArrayOfValuesOverSubrelations, EntityEdge)

#define DECLARE_SUMCHECK_RELATION_CLASS(RelationImpl, Flavor) SUMCHECK_RELATION_CLASS(extern, RelationImpl, Flavor)
#define DEFINE_SUMCHECK_RELATION_CLASS(RelationImpl, Flavor) SUMCHECK_RELATION_CLASS(, RelationImpl, Flavor)

#define SUMCHECK_PERMUTATION_CLASS(...) _SUMCHECK_PERMUTATION_CLASS(__VA_ARGS__)
#define _SUMCHECK_PERMUTATION_CLASS(Preface, RelationImpl, Flavor)                                                     \
    PERMUTATION_METHOD(                                                                                                \
        Preface, compute_permutation_numerator, RelationImpl, Flavor, UnivariateAccumulator0, ExtendedEdge)            \
    PERMUTATION_METHOD(                                                                                                \
        Preface, compute_permutation_numerator, RelationImpl, Flavor, ValueAccumulator0, EvaluationEdge)               \
    PERMUTATION_METHOD(Preface, compute_permutation_numerator, RelationImpl, Flavor, ValueAccumulator0, EntityEdge)    \
    PERMUTATION_METHOD(                                                                                                \
        Preface, compute_permutation_denominator, RelationImpl, Flavor, UnivariateAccumulator0, ExtendedEdge)          \
    PERMUTATION_METHOD(                                                                                                \
        Preface, compute_permutation_denominator, RelationImpl, Flavor, ValueAccumulator0, EvaluationEdge)             \
    PERMUTATION_METHOD(Preface, compute_permutation_denominator, RelationImpl, Flavor, ValueAccumulator0, EntityEdge)

#define DECLARE_SUMCHECK_PERMUTATION_CLASS(RelationImpl, Flavor)                                                       \
    SUMCHECK_PERMUTATION_CLASS(extern, RelationImpl, Flavor)
#define DEFINE_SUMCHECK_PERMUTATION_CLASS(RelationImpl, Flavor) SUMCHECK_PERMUTATION_CLASS(, RelationImpl, Flavor)
