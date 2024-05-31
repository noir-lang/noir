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
#include "barretenberg/stdlib/primitives/curves/grumpkin.hpp"

// NOLINTBEGIN(cppcoreguidelines-avoid-const-or-ref-data-members) ?

namespace bb {

template <typename BuilderType> class ECCVMRecursiveFlavor_ {
  public:
    using CircuitBuilder = BuilderType; // determines the arithmetisation of recursive verifier
    using Curve = stdlib::grumpkin<CircuitBuilder>;
    using Commitment = Curve::AffineElement;
    using GroupElement = Curve::Element;
    using FF = Curve::ScalarField;
    using BF = Curve::BaseField;
    using RelationSeparator = FF;
    using NativeFlavor = ECCVMFlavor;
    using NativeVerificationKey = NativeFlavor::VerificationKey;
    using PCS = IPA<Curve>;

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

    using VerifierCommitmentKey = bb::VerifierCommitmentKey<Curve>;
    /**
     * @brief The verification key is responsible for storing the the commitments to the precomputed (non-witness)
     * polynomials used by the verifier.
     *
     * @note Note the discrepancy with what sort of data is stored here vs in the proving key. We may want to
     * resolve that, and split out separate PrecomputedPolynomials/Commitments data for clarity but also for
     * portability of our circuits.
     */
    class VerificationKey
        : public VerificationKey_<ECCVMFlavor::PrecomputedEntities<Commitment>, VerifierCommitmentKey> {
      public:
        VerificationKey(const size_t circuit_size, const size_t num_public_inputs)
        {
            this->circuit_size = circuit_size;
            this->log_circuit_size = numeric::get_msb(circuit_size);
            this->num_public_inputs = num_public_inputs;
        };

        /**
         * @brief Construct a new Verification Key with stdlib types from a provided native verification
         * key
         *
         * @param builder
         * @param native_key Native verification key from which to extract the precomputed commitments
         */

        VerificationKey(CircuitBuilder* builder, const std::shared_ptr<NativeVerificationKey>& native_key)
        {
            this->pcs_verification_key = std::make_shared<VerifierCommitmentKey>(
                builder, native_key->circuit_size, native_key->pcs_verification_key);
            this->circuit_size = native_key->circuit_size;
            this->log_circuit_size = numeric::get_msb(this->circuit_size);
            this->num_public_inputs = native_key->num_public_inputs;
            this->pub_inputs_offset = native_key->pub_inputs_offset;

            for (auto [native_commitment, commitment] : zip_view(native_key->get_all(), this->get_all())) {
                commitment = Commitment::from_witness(builder, native_commitment);
            }
        }
    };

    /**
     * @brief A container for the witness commitments.
     */
    using WitnessCommitments = ECCVMFlavor::WitnessEntities<Commitment>;

    using CommitmentLabels = ECCVMFlavor::CommitmentLabels;
    // Reuse the VerifierCommitments from ECCVM
    using VerifierCommitments = ECCVMFlavor::VerifierCommitments_<Commitment, VerificationKey>;
    // Reuse the transcript from ECCVM
    using Transcript = bb::BaseTranscript<bb::stdlib::recursion::honk::StdlibTranscriptParams<CircuitBuilder>>;

}; // NOLINTEND(cppcoreguidelines-avoid-const-or-ref-data-members)

} // namespace bb
