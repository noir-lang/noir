#pragma once
#include "barretenberg/common/std_array.hpp"
#include "barretenberg/eccvm/eccvm_flavor.hpp"
#include "barretenberg/eccvm_recursion/verifier_commitment_key.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/relations/ecc_vm/ecc_lookup_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_msm_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_point_table_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_set_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_transcript_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_wnaf_relation.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/stdlib/honk_recursion/transcript/transcript.hpp"

// NOLINTBEGIN(cppcoreguidelines-avoid-const-or-ref-data-members) ?

namespace bb {

template <typename BuilderType> class ECCVMRecursiveFlavor_ {
  public:
    using CircuitBuilder = BuilderType; // determines the arithmetisation of recursive verifier
    using FF = stdlib::bigfield<CircuitBuilder, bb::Bn254FqParams>;
    using BF = stdlib::field_t<CircuitBuilder>;
    using RelationSeparator = FF;
    using NativeFlavor = ECCVMFlavor;
    using NativeVerificationKey = NativeFlavor::VerificationKey;

    static constexpr size_t NUM_WIRES = ECCVMFlavor::NUM_WIRES;
    // The number of multivariate polynomials on which a sumcheck prover sumcheck operates (including shifts). We often
    // need containers of this size to hold related data, so we choose a name more agnostic than `NUM_POLYNOMIALS`.
    // Note: this number does not include the individual sorted list polynomials.
    static constexpr size_t NUM_ALL_ENTITIES = ECCVMFlavor::NUM_ALL_ENTITIES;
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = ECCVMFlavor::NUM_PRECOMPUTED_ENTITIES;
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES = ECCVMFlavor::NUM_WITNESS_ENTITIES;

    // define the tuple of Relations that comprise the Sumcheck relation
    // Reuse the Relations from ECCVM
    using Relations = ECCVMFlavor::Relations_<FF>;

    // think these two are not needed for recursive verifier land
    // using GrandProductRelations = std::tuple<ECCVMSetRelation<FF>>;
    // using LookupRelation = ECCVMLookupRelation<FF>;
    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_partial_relation_length<Relations>();

    // BATCHED_RELATION_PARTIAL_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta`
    // random polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation
    // length = 3
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = MAX_PARTIAL_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size<Relations>::value;

    // Instantiate the BarycentricData needed to extend each Relation Univariate

    // define the containers for storing the contributions from each relation in Sumcheck
    using TupleOfArraysOfValues = decltype(create_tuple_of_arrays_of_values<Relations>());

  public:
    /**
     * @brief A field element for each entity of the flavor.  These entities represent the prover polynomials
     * evaluated at one point.
     */
    class AllValues : public ECCVMFlavor::AllEntities<FF> {
      public:
        using Base = ECCVMFlavor::AllEntities<FF>;
        using Base::Base;
    };

}; // NOLINTEND(cppcoreguidelines-avoid-const-or-ref-data-members)

} // namespace bb
