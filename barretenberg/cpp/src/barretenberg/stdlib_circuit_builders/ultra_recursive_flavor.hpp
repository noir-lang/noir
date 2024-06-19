#pragma once
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/polynomials/barycentric.hpp"
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/relations/auxiliary_relation.hpp"
#include "barretenberg/relations/delta_range_constraint_relation.hpp"
#include "barretenberg/relations/elliptic_relation.hpp"
#include "barretenberg/relations/lookup_relation.hpp"
#include "barretenberg/relations/permutation_relation.hpp"
#include "barretenberg/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"

#include <array>
#include <concepts>
#include <span>
#include <string>
#include <type_traits>
#include <vector>

#include "barretenberg/stdlib/honk_recursion/transcript/transcript.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"

namespace bb {

/**
 * @brief The recursive counterpart to the "native" Ultra flavor.
 * @details This flavor can be used to instantiate a recursive Ultra Honk verifier for a proof created using the
 * conventional Ultra flavor. It is similar in structure to its native counterpart with two main differences: 1) the
 * curve types are stdlib types (e.g. field_t instead of field) and 2) it does not specify any Prover related types
 * (e.g. Polynomial, ProverUnivariates, etc.) since we do not emulate prover computation in circuits, i.e. it only makes
 * sense to instantiate a Verifier with this flavor.
 *
 * @note Unlike conventional flavors, "recursive" flavors are templated by a builder (much like native vs stdlib types).
 * This is because the flavor itself determines the details of the underlying verifier algorithm (i.e. the set of
 * relations), while the Builder determines the arithmetization of that algorithm into a circuit.
 *
 * @tparam BuilderType Determines the arithmetization of the verifier circuit defined based on this flavor.
 */
template <typename BuilderType> class UltraRecursiveFlavor_ {
  public:
    using CircuitBuilder = BuilderType; // Determines arithmetization of circuit instantiated with this flavor
    using Curve = stdlib::bn254<CircuitBuilder>;
    using PCS = KZG<Curve>;
    using GroupElement = typename Curve::Element;
    using Commitment = typename Curve::Element;
    using FF = typename Curve::ScalarField;
    using NativeFlavor = UltraFlavor;
    using NativeVerificationKey = NativeFlavor::VerificationKey;

    // Note(luke): Eventually this may not be needed at all
    using VerifierCommitmentKey = bb::VerifierCommitmentKey<NativeFlavor::Curve>;

    static constexpr size_t NUM_WIRES = UltraFlavor::NUM_WIRES;
    // The number of multivariate polynomials on which a sumcheck prover sumcheck operates (including shifts). We often
    // need containers of this size to hold related data, so we choose a name more agnostic than `NUM_POLYNOMIALS`.
    // Note: this number does not include the individual sorted list polynomials.
    static constexpr size_t NUM_ALL_ENTITIES = 43;
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 25;
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES = 7;

    // define the tuple of Relations that comprise the Sumcheck relation
    using Relations = UltraFlavor::Relations_<FF>;

    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_partial_relation_length<Relations>();
    static_assert(MAX_PARTIAL_RELATION_LENGTH == 6);
    static constexpr size_t MAX_TOTAL_RELATION_LENGTH = compute_max_total_relation_length<Relations>();
    static_assert(MAX_TOTAL_RELATION_LENGTH == 11);

    // BATCHED_RELATION_PARTIAL_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta`
    // random polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation
    // length = 3
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = MAX_PARTIAL_RELATION_LENGTH + 1;
    static constexpr size_t BATCHED_RELATION_TOTAL_LENGTH = MAX_TOTAL_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size<Relations>::value;

    // For instances of this flavour, used in folding, we need a unique sumcheck batching challenges for each
    // subrelation to avoid increasing the degree of Protogalaxy polynomial $G$ (the
    // combiner) too much.
    static constexpr size_t NUM_SUBRELATIONS = compute_number_of_subrelations<Relations>();
    using RelationSeparator = std::array<FF, NUM_SUBRELATIONS - 1>;

    // define the container for storing the univariate contribution from each relation in Sumcheck
    using TupleOfArraysOfValues = decltype(create_tuple_of_arrays_of_values<Relations>());

  public:
    /**
     * @brief The verification key is responsible for storing the the commitments to the precomputed (non-witnessk)
     * polynomials used by the verifier.
     *
     * @note Note the discrepancy with what sort of data is stored here vs in the proving key. We may want to resolve
     * that, and split out separate PrecomputedPolynomials/Commitments data for clarity but also for portability of our
     * circuits.
     */
    class VerificationKey
        : public VerificationKey_<UltraFlavor::PrecomputedEntities<Commitment>, VerifierCommitmentKey> {
      public:
        VerificationKey(const size_t circuit_size, const size_t num_public_inputs)
        {
            // TODO(https://github.com/AztecProtocol/barretenberg/issues/983): Think about if these should be witnesses
            this->circuit_size = circuit_size;
            this->log_circuit_size = numeric::get_msb(circuit_size);
            this->num_public_inputs = num_public_inputs;
        };
        /**
         * @brief Construct a new Verification Key with stdlib types from a provided native verification key
         *
         * @param builder
         * @param native_key Native verification key from which to extract the precomputed commitments
         */
        VerificationKey(CircuitBuilder* builder, const std::shared_ptr<NativeVerificationKey>& native_key)
        {
            this->pcs_verification_key = native_key->pcs_verification_key;
            this->circuit_size = native_key->circuit_size;
            this->log_circuit_size = numeric::get_msb(this->circuit_size);
            this->num_public_inputs = native_key->num_public_inputs;
            this->pub_inputs_offset = native_key->pub_inputs_offset;
            this->q_m = Commitment::from_witness(builder, native_key->q_m);
            this->q_l = Commitment::from_witness(builder, native_key->q_l);
            this->q_r = Commitment::from_witness(builder, native_key->q_r);
            this->q_o = Commitment::from_witness(builder, native_key->q_o);
            this->q_4 = Commitment::from_witness(builder, native_key->q_4);
            this->q_c = Commitment::from_witness(builder, native_key->q_c);
            this->q_arith = Commitment::from_witness(builder, native_key->q_arith);
            this->q_delta_range = Commitment::from_witness(builder, native_key->q_delta_range);
            this->q_elliptic = Commitment::from_witness(builder, native_key->q_elliptic);
            this->q_aux = Commitment::from_witness(builder, native_key->q_aux);
            this->q_lookup = Commitment::from_witness(builder, native_key->q_lookup);
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
        };

        /**
         * @brief Deserialize a verification key from a vector of field elements
         *
         * @param builder
         * @param elements
         */
        VerificationKey(CircuitBuilder& builder, std::span<FF> elements)
        {
            // deserialize circuit size
            size_t num_frs_read = 0;
            size_t num_frs_FF = bb::stdlib::field_conversion::calc_num_bn254_frs<CircuitBuilder, FF>();
            size_t num_frs_Comm = bb::stdlib::field_conversion::calc_num_bn254_frs<CircuitBuilder, Commitment>();

            this->circuit_size = uint64_t(stdlib::field_conversion::convert_from_bn254_frs<CircuitBuilder, FF>(
                                              builder, elements.subspan(num_frs_read, num_frs_FF))
                                              .get_value());
            num_frs_read += num_frs_FF;
            this->num_public_inputs = uint64_t(stdlib::field_conversion::convert_from_bn254_frs<CircuitBuilder, FF>(
                                                   builder, elements.subspan(num_frs_read, num_frs_FF))
                                                   .get_value());
            num_frs_read += num_frs_FF;

            this->pub_inputs_offset = uint64_t(stdlib::field_conversion::convert_from_bn254_frs<CircuitBuilder, FF>(
                                                   builder, elements.subspan(num_frs_read, num_frs_FF))
                                                   .get_value());
            num_frs_read += num_frs_FF;

            for (Commitment& comm : this->get_all()) {
                comm = bb::stdlib::field_conversion::convert_from_bn254_frs<CircuitBuilder, Commitment>(
                    builder, elements.subspan(num_frs_read, num_frs_Comm));
                num_frs_read += num_frs_Comm;
            }
        }
    };

    /**
     * @brief A field element for each entity of the flavor. These entities represent the prover polynomials
     * evaluated at one point.
     */
    class AllValues : public UltraFlavor::AllEntities<FF> {
      public:
        using Base = UltraFlavor::AllEntities<FF>;
        using Base::Base;
    };

    using CommitmentLabels = UltraFlavor::CommitmentLabels;

    using WitnessCommitments = UltraFlavor::WitnessEntities<Commitment>;

    class VerifierCommitments : public UltraFlavor::AllEntities<Commitment> {
      public:
        VerifierCommitments(const std::shared_ptr<VerificationKey>& verification_key,
                            const std::optional<WitnessCommitments>& witness_commitments = std::nullopt)
        {
            this->q_m = verification_key->q_m;
            this->q_l = verification_key->q_l;
            this->q_r = verification_key->q_r;
            this->q_o = verification_key->q_o;
            this->q_4 = verification_key->q_4;
            this->q_c = verification_key->q_c;
            this->q_arith = verification_key->q_arith;
            this->q_delta_range = verification_key->q_delta_range;
            this->q_elliptic = verification_key->q_elliptic;
            this->q_aux = verification_key->q_aux;
            this->q_lookup = verification_key->q_lookup;
            this->sigma_1 = verification_key->sigma_1;
            this->sigma_2 = verification_key->sigma_2;
            this->sigma_3 = verification_key->sigma_3;
            this->sigma_4 = verification_key->sigma_4;
            this->id_1 = verification_key->id_1;
            this->id_2 = verification_key->id_2;
            this->id_3 = verification_key->id_3;
            this->id_4 = verification_key->id_4;
            this->table_1 = verification_key->table_1;
            this->table_2 = verification_key->table_2;
            this->table_3 = verification_key->table_3;
            this->table_4 = verification_key->table_4;
            this->lagrange_first = verification_key->lagrange_first;
            this->lagrange_last = verification_key->lagrange_last;

            if (witness_commitments.has_value()) {
                auto commitments = witness_commitments.value();
                this->w_l = commitments.w_l;
                this->w_r = commitments.w_r;
                this->w_o = commitments.w_o;
                this->sorted_accum = commitments.sorted_accum;
                this->w_4 = commitments.w_4;
                this->z_perm = commitments.z_perm;
                this->z_lookup = commitments.z_lookup;
            }
        }
    };

    using Transcript = bb::BaseTranscript<bb::stdlib::recursion::honk::StdlibTranscriptParams<CircuitBuilder>>;
};

} // namespace bb