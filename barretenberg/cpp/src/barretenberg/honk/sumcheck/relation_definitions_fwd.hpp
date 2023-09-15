#pragma once

#include "barretenberg/proof_system/relations/relation_types.hpp"

#define ExtendedEdge(Flavor) Flavor::ExtendedEdges<Flavor::MAX_RELATION_LENGTH>
#define EvaluationEdge(Flavor) Flavor::ClaimedEvaluations
#define EntityEdge(Flavor) Flavor::AllEntities<Flavor::FF, Flavor::FF>

#define ADD_EDGE_CONTRIBUTION(...) _ADD_EDGE_CONTRIBUTION(__VA_ARGS__)
#define _ADD_EDGE_CONTRIBUTION(Preface, RelationBase, Flavor, AccumulatorType, EdgeType)                               \
    Preface template void                                                                                              \
    RelationBase<Flavor::FF>::accumulate<proof_system::Relation<RelationBase<Flavor::FF>>::AccumulatorType,            \
                                         EdgeType(Flavor)>(                                                            \
        proof_system::Relation<RelationBase<Flavor::FF>>::AccumulatorType::Accumulators&,                              \
        EdgeType(Flavor) const&,                                                                                       \
        RelationParameters<Flavor::FF> const&,                                                                         \
        Flavor::FF const&);

#define PERMUTATION_METHOD(...) _PERMUTATION_METHOD(__VA_ARGS__)
#define _PERMUTATION_METHOD(Preface, MethodName, RelationBase, Flavor, AccumulatorType, EdgeType)                      \
    Preface template RelationBase<Flavor::FF>::template Accumulator<                                                   \
        proof_system::Relation<RelationBase<Flavor::FF>>::AccumulatorType>                                             \
    RelationBase<Flavor::FF>::MethodName<proof_system::Relation<RelationBase<Flavor::FF>>::AccumulatorType,            \
                                         EdgeType(Flavor)>(                                                            \
        EdgeType(Flavor) const&, RelationParameters<Flavor::FF> const&, size_t const);

#define SUMCHECK_RELATION_CLASS(...) _SUMCHECK_RELATION_CLASS(__VA_ARGS__)
#define _SUMCHECK_RELATION_CLASS(Preface, RelationBase, Flavor)                                                        \
    ADD_EDGE_CONTRIBUTION(Preface, RelationBase, Flavor, UnivariateAccumulatorsAndViews, ExtendedEdge)                 \
    ADD_EDGE_CONTRIBUTION(Preface, RelationBase, Flavor, ValueAccumulatorsAndViews, EvaluationEdge)                    \
    ADD_EDGE_CONTRIBUTION(Preface, RelationBase, Flavor, ValueAccumulatorsAndViews, EntityEdge)

#define DECLARE_SUMCHECK_RELATION_CLASS(RelationBase, Flavor) SUMCHECK_RELATION_CLASS(extern, RelationBase, Flavor)
#define DEFINE_SUMCHECK_RELATION_CLASS(RelationBase, Flavor) SUMCHECK_RELATION_CLASS(, RelationBase, Flavor)

#define SUMCHECK_PERMUTATION_CLASS(...) _SUMCHECK_PERMUTATION_CLASS(__VA_ARGS__)
#define _SUMCHECK_PERMUTATION_CLASS(Preface, RelationBase, Flavor)                                                     \
    PERMUTATION_METHOD(                                                                                                \
        Preface, compute_permutation_numerator, RelationBase, Flavor, UnivariateAccumulatorsAndViews, ExtendedEdge)    \
    PERMUTATION_METHOD(                                                                                                \
        Preface, compute_permutation_numerator, RelationBase, Flavor, ValueAccumulatorsAndViews, EvaluationEdge)       \
    PERMUTATION_METHOD(                                                                                                \
        Preface, compute_permutation_numerator, RelationBase, Flavor, ValueAccumulatorsAndViews, EntityEdge)           \
    PERMUTATION_METHOD(                                                                                                \
        Preface, compute_permutation_denominator, RelationBase, Flavor, UnivariateAccumulatorsAndViews, ExtendedEdge)  \
    PERMUTATION_METHOD(                                                                                                \
        Preface, compute_permutation_denominator, RelationBase, Flavor, ValueAccumulatorsAndViews, EvaluationEdge)     \
    PERMUTATION_METHOD(                                                                                                \
        Preface, compute_permutation_denominator, RelationBase, Flavor, ValueAccumulatorsAndViews, EntityEdge)

#define DECLARE_SUMCHECK_PERMUTATION_CLASS(RelationBase, Flavor)                                                       \
    SUMCHECK_PERMUTATION_CLASS(extern, RelationBase, Flavor)
#define DEFINE_SUMCHECK_PERMUTATION_CLASS(RelationBase, Flavor) SUMCHECK_PERMUTATION_CLASS(, RelationBase, Flavor)
