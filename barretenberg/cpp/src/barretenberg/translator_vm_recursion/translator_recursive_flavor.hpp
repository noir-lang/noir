#pragma once
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/stdlib/honk_recursion/transcript/transcript.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_circuit_builder.hpp"
#include "barretenberg/translator_vm/translator_flavor.hpp"

namespace bb {

/**
 * @brief The recursive counterpart of the native Translator flavor.
 * @details is flavor can be used to instantiate a recursive Translator verifier for a proof created using the
 * Translator flavor. It is similar in structure to its native counterpart with two main differences: 1) the
 * curve types are stdlib types (e.g. field_t instead of field) and 2) it does not specify any Prover related types
 * (e.g. Polynomial, ExtendedEdges, etc.) since we do not emulate prover computation in circuits, i.e. it only makes
 * sense to instantiate a Verifier with this flavor. We reuse the native flavor to initialise identical  constructions.
 * @tparam BuilderType Determines the arithmetization of the verifier circuit defined based on this flavor.
 */
template <typename BuilderType> class TranslatorRecursiveFlavor_ {

  public:
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/990): Establish whether mini_circuit_size pattern is
    // needed
    static constexpr size_t mini_circuit_size = 2048;
    using CircuitBuilder = BuilderType;
    using Curve = stdlib::bn254<CircuitBuilder>;
    using PCS = KZG<Curve>;
    using GroupElement = Curve::Element;
    using Commitment = Curve::AffineElement;
    using FF = Curve::ScalarField;
    using BF = Curve::BaseField;
    using RelationSeparator = FF;

    using NativeFlavor = TranslatorFlavor;
    using NativeVerificationKey = NativeFlavor::VerificationKey;

    using VerifierCommitmentKey = bb::VerifierCommitmentKey<NativeFlavor::Curve>;
    static constexpr size_t MINIMUM_MINI_CIRCUIT_SIZE = 2048;

    // The size of the circuit which is filled with non-zero values for most polynomials. Most relations (everything
    // except for Permutation and DeltaRangeConstraint) can be evaluated just on the first chunk
    // It is also the only parameter that can be changed without updating relations or structures in the flavor
    static constexpr size_t MINI_CIRCUIT_SIZE = mini_circuit_size;

    // None of this parameters can be changed

    // How many mini_circuit_size polynomials are concatenated in one concatenated_*
    static constexpr size_t CONCATENATION_GROUP_SIZE = NativeFlavor::CONCATENATION_GROUP_SIZE;

    // The number of concatenated_* wires
    static constexpr size_t NUM_CONCATENATED_WIRES = NativeFlavor::NUM_CONCATENATED_WIRES;

    // Actual circuit size
    static constexpr size_t FULL_CIRCUIT_SIZE = MINI_CIRCUIT_SIZE * CONCATENATION_GROUP_SIZE;

    // Number of wires
    static constexpr size_t NUM_WIRES = NativeFlavor::NUM_WIRES;

    // The step in the DeltaRangeConstraint relation
    static constexpr size_t SORT_STEP = NativeFlavor::SORT_STEP;

    // The bitness of the range constraint
    static constexpr size_t MICRO_LIMB_BITS = NativeFlavor::MICRO_LIMB_BITS;

    // The limbs of the modulus we are emulating in the goblin translator. 4 binary 68-bit limbs and the prime one
    static constexpr auto NEGATIVE_MODULUS_LIMBS = NativeFlavor::NEGATIVE_MODULUS_LIMBS;

    // Number of bits in a binary limb
    // This is not a configurable value. Relations are sepcifically designed for it to be 68
    static constexpr size_t NUM_LIMB_BITS = NativeFlavor::NUM_LIMB_BITS;

    // The number of multivariate polynomials on which a sumcheck prover sumcheck operates (including shifts). We
    // often need containers of this size to hold related data, so we choose a name more agnostic than
    // `NUM_POLYNOMIALS`. Note: this number does not include the individual sorted list polynomials.
    static constexpr size_t NUM_ALL_ENTITIES = NativeFlavor::NUM_ALL_ENTITIES;
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = NativeFlavor::NUM_PRECOMPUTED_ENTITIES;
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES = NativeFlavor::NUM_WITNESS_ENTITIES;

    using Relations = TranslatorFlavor::Relations_<FF>;

    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_partial_relation_length<Relations>();
    static constexpr size_t MAX_TOTAL_RELATION_LENGTH = compute_max_total_relation_length<Relations>();

    // BATCHED_RELATION_PARTIAL_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta`
    // random polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation
    // length = 3
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = MAX_PARTIAL_RELATION_LENGTH + 1;
    static constexpr size_t BATCHED_RELATION_TOTAL_LENGTH = MAX_TOTAL_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size_v<Relations>;

    // define the containers for storing the contributions from each relation in Sumcheck
    using TupleOfArraysOfValues = decltype(create_tuple_of_arrays_of_values<Relations>());

    /**
     * @brief A field element for each entity of the flavor.  These entities represent the prover polynomials
     * evaluated at one point.
     */
    class AllValues : public TranslatorFlavor::AllEntities<FF> {
      public:
        using Base = TranslatorFlavor::AllEntities<FF>;
        using Base::Base;
    };
    /**
     * @brief The verification key is responsible for storing the the commitments to the precomputed (non-witnessk)
     * polynomials used by the verifier.
     *
     * @note Note the discrepancy with what sort of data is stored here vs in the proving key. We may want to
     * resolve that, and split out separate PrecomputedPolynomials/Commitments data for clarity but also for
     * portability of our circuits.
     */
    class VerificationKey
        : public VerificationKey_<TranslatorFlavor::PrecomputedEntities<Commitment>, VerifierCommitmentKey> {
      public:
        VerificationKey(const size_t circuit_size, const size_t num_public_inputs)
        {
            this->circuit_size = circuit_size;
            this->log_circuit_size = numeric::get_msb(circuit_size);
            this->num_public_inputs = num_public_inputs;
        }

        VerificationKey(CircuitBuilder* builder, const std::shared_ptr<NativeVerificationKey>& native_key)
        {
            this->pcs_verification_key = std::make_shared<VerifierCommitmentKey>(); // ?
            this->circuit_size = native_key->circuit_size;
            this->log_circuit_size = numeric::get_msb(this->circuit_size);
            this->num_public_inputs = native_key->num_public_inputs;
            this->pub_inputs_offset = native_key->pub_inputs_offset;

            for (auto [native_comm, comm] : zip_view(native_key->get_all(), this->get_all())) {
                comm = Commitment::from_witness(builder, native_comm);
            }
        }
    };

    /**
     * @brief A container for the witness commitments.
     */
    using WitnessCommitments = TranslatorFlavor::WitnessEntities<Commitment>;

    using CommitmentLabels = TranslatorFlavor::CommitmentLabels;
    // Reuse the VerifierCommitments from Translator
    using VerifierCommitments = TranslatorFlavor::VerifierCommitments_<Commitment, VerificationKey>;
    // Reuse the transcript from Translator
    using Transcript = bb::BaseTranscript<bb::stdlib::recursion::honk::StdlibTranscriptParams<CircuitBuilder>>;
};
} // namespace bb