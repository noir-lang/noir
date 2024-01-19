#pragma once
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/polynomials/barycentric.hpp"
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include <array>
#include <concepts>
#include <span>
#include <string>
#include <type_traits>
#include <vector>

#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/recursion/honk/transcript/transcript.hpp"

namespace bb::honk::flavor {

/**
 * @brief The recursive counterpart to the "native" Goblin Ultra flavor.
 * @details This flavor can be used to instantiate a recursive Ultra Honk verifier for a proof created using the
 * GoblinUltra flavor. It is similar in structure to its native counterpart with two main differences: 1) the
 * curve types are stdlib types (e.g. field_t instead of field) and 2) it does not specify any Prover related types
 * (e.g. Polynomial, ExtendedEdges, etc.) since we do not emulate prover computation in circuits, i.e. it only makes
 * sense to instantiate a Verifier with this flavor.
 *
 * @note Unlike conventional flavors, "recursive" flavors are templated by a builder (much like native vs stdlib types).
 * This is because the flavor itself determines the details of the underlying verifier algorithm (i.e. the set of
 * relations), while the Builder determines the arithmetization of that algorithm into a circuit.
 *
 * @tparam BuilderType Determines the arithmetization of the verifier circuit defined based on this flavor.
 */
template <typename BuilderType> class GoblinUltraRecursive_ {
  public:
    using CircuitBuilder = BuilderType; // Determines arithmetization of circuit instantiated with this flavor
    using Curve = plonk::stdlib::bn254<CircuitBuilder>;
    using GroupElement = typename Curve::Element;
    using FF = typename Curve::ScalarField;
    using Commitment = typename Curve::Element;
    using CommitmentHandle = typename Curve::Element;
    using NativeVerificationKey = flavor::GoblinUltra::VerificationKey;

    // Note(luke): Eventually this may not be needed at all
    using VerifierCommitmentKey = pcs::VerifierCommitmentKey<Curve>;

    static constexpr size_t NUM_WIRES = flavor::GoblinUltra::NUM_WIRES;
    // The number of multivariate polynomials on which a sumcheck prover sumcheck operates (including shifts). We often
    // need containers of this size to hold related data, so we choose a name more agnostic than `NUM_POLYNOMIALS`.
    // Note: this number does not include the individual sorted list polynomials.
    static constexpr size_t NUM_ALL_ENTITIES = flavor::GoblinUltra::NUM_ALL_ENTITIES;
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = flavor::GoblinUltra::NUM_PRECOMPUTED_ENTITIES;
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES = flavor::GoblinUltra::NUM_WITNESS_ENTITIES;

    // define the tuple of Relations that comprise the Sumcheck relation
    // Reuse the Relations from GoblinUltra
    using Relations = GoblinUltra::Relations_<FF>;

    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_partial_relation_length<Relations>();

    // BATCHED_RELATION_PARTIAL_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta`
    // random polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation
    // length = 3
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = MAX_PARTIAL_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size<Relations>::value;

    // For instances of this flavour, used in folding, we need a unique sumcheck batching challenge for each
    // subrelation. This is because using powers of alpha would increase the degree of Protogalaxy polynomial $G$ (the
    // combiner) to much.
    static constexpr size_t NUM_SUBRELATIONS = compute_number_of_subrelations<Relations>();
    using RelationSeparator = std::array<FF, NUM_SUBRELATIONS - 1>;

    // define the container for storing the univariate contribution from each relation in Sumcheck
    using SumcheckTupleOfTuplesOfUnivariates = decltype(create_sumcheck_tuple_of_tuples_of_univariates<Relations>());
    using TupleOfArraysOfValues = decltype(create_tuple_of_arrays_of_values<Relations>());

    /**
     * @brief A field element for each entity of the flavor. These entities represent the prover polynomials evaluated
     * at one point.
     */
    class AllValues : public GoblinUltra::AllEntities<FF> {
      public:
        using Base = GoblinUltra::AllEntities<FF>;
        using Base::Base;
    };
    /**
     * @brief The verification key is responsible for storing the the commitments to the precomputed (non-witnessk)
     * polynomials used by the verifier.
     *
     * @note Note the discrepancy with what sort of data is stored here vs in the proving key. We may want to resolve
     * that, and split out separate PrecomputedPolynomials/Commitments data for clarity but also for portability of our
     * circuits.
     * This differs from GoblinUltra in how we construct the commitments.
     */
    class VerificationKey : public VerificationKey_<GoblinUltra::PrecomputedEntities<Commitment>> {
      public:
        /**
         * @brief Construct a new Verification Key with stdlib types from a provided native verification
         * key
         *
         * @param builder
         * @param native_key Native verification key from which to extract the precomputed commitments
         */
        VerificationKey(CircuitBuilder* builder, const std::shared_ptr<NativeVerificationKey>& native_key)
            : VerificationKey_<GoblinUltra::PrecomputedEntities<Commitment>>(native_key->circuit_size,
                                                                             native_key->num_public_inputs)
        {
            this->q_m = Commitment::from_witness(builder, native_key->q_m);
            this->q_l = Commitment::from_witness(builder, native_key->q_l);
            this->q_r = Commitment::from_witness(builder, native_key->q_r);
            this->q_o = Commitment::from_witness(builder, native_key->q_o);
            this->q_4 = Commitment::from_witness(builder, native_key->q_4);
            this->q_c = Commitment::from_witness(builder, native_key->q_c);
            this->q_arith = Commitment::from_witness(builder, native_key->q_arith);
            this->q_sort = Commitment::from_witness(builder, native_key->q_sort);
            this->q_elliptic = Commitment::from_witness(builder, native_key->q_elliptic);
            this->q_aux = Commitment::from_witness(builder, native_key->q_aux);
            this->q_lookup = Commitment::from_witness(builder, native_key->q_lookup);
            this->q_busread = Commitment::from_witness(builder, native_key->q_busread);
            this->q_poseidon2_external = Commitment::from_witness(builder, native_key->q_poseidon2_external);
            this->q_poseidon2_internal = Commitment::from_witness(builder, native_key->q_poseidon2_internal);
            this->sigma_1 = Commitment::from_witness(builder, native_key->sigma_1);
            this->sigma_2 = Commitment::from_witness(builder, native_key->sigma_2);
            this->sigma_3 = Commitment::from_witness(builder, native_key->sigma_3);
            this->sigma_4 = Commitment::from_witness(builder, native_key->sigma_4);
            this->id_1 = Commitment::from_witness(builder, native_key->id_1);
            this->id_2 = Commitment::from_witness(builder, native_key->id_2);
            this->id_3 = Commitment::from_witness(builder, native_key->id_3);
            this->id_4 = Commitment::from_witness(builder, native_key->id_4);
            this->table_1 = Commitment::from_witness(builder, native_key->table_1);
            this->table_2 = Commitment::from_witness(builder, native_key->table_2);
            this->table_3 = Commitment::from_witness(builder, native_key->table_3);
            this->table_4 = Commitment::from_witness(builder, native_key->table_4);
            this->lagrange_first = Commitment::from_witness(builder, native_key->lagrange_first);
            this->lagrange_last = Commitment::from_witness(builder, native_key->lagrange_last);
            this->lagrange_ecc_op = Commitment::from_witness(builder, native_key->lagrange_ecc_op);
            this->databus_id = Commitment::from_witness(builder, native_key->databus_id);
        };
    };

    using CommitmentLabels = GoblinUltra::CommitmentLabels;
    // Reuse the VerifierCommitments from GoblinUltra
    using VerifierCommitments = GoblinUltra::VerifierCommitments_<Commitment, VerificationKey>;
    // Reuse the transcript from GoblinUltra
    using Transcript = bb::plonk::stdlib::recursion::honk::Transcript<CircuitBuilder>;
};

} // namespace bb::honk::flavor
