#pragma once
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/relation_types.hpp"
#include "barretenberg/relations/toy_avm/generic_permutation_relation.hpp"
#include "barretenberg/relations/toy_avm/relation_definer.hpp"
#include "relation_definitions_fwd.hpp"
#include <array>
#include <concepts>
#include <span>
#include <string>
#include <type_traits>
#include <vector>

// NOLINTBEGIN(cppcoreguidelines-avoid-const-or-ref-data-members)

namespace proof_system::honk {
namespace flavor {

/**
 * @brief This class provides an example flavor for using GenericPermutationRelations with various settings to make
 * integrating those mechanisms into AVM easier
 *
 */
class ToyAVM {
  public:
    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using GroupElement = Curve::Element;
    using Commitment = Curve::AffineElement;
    using CommitmentHandle = Curve::AffineElement;
    using PCS = pcs::kzg::KZG<Curve>;
    using Polynomial = barretenberg::Polynomial<FF>;
    using PolynomialHandle = std::span<FF>;
    using CommitmentKey = pcs::CommitmentKey<Curve>;
    using VerifierCommitmentKey = pcs::VerifierCommitmentKey<Curve>;

    // The number of wires is 5. The set of tuples (permutation_set_column_1,permutation_set_column_2) should be
    // equivalent to (permutation_set_column_3, permutation_set_column_4) and the self_permutation_column contains 2
    // subsets which are permutations of each other
    static constexpr size_t NUM_WIRES = 5;

    // The number of multivariate polynomials on which a sumcheck prover sumcheck operates (including shifts). We often
    // need containers of this size to hold related data, so we choose a name more agnostic than `NUM_POLYNOMIALS`.
    // Note: this number does not include the individual sorted list polynomials.
    static constexpr size_t NUM_ALL_ENTITIES = 12;
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 5;
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES = 7;

    // define the tuple of Relations that comprise the Sumcheck relation
    using Relations = std::tuple<sumcheck::GenericPermutationRelation<sumcheck::ExampleTuplePermutationSettings, FF>>;

    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_partial_relation_length<Relations>();

    // BATCHED_RELATION_PARTIAL_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta`
    // random polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation
    // length = 3
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = MAX_PARTIAL_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size<Relations>::value;

    // Instantiate the BarycentricData needed to extend each Relation Univariate

    // define the containers for storing the contributions from each relation in Sumcheck
    using SumcheckTupleOfTuplesOfUnivariates = decltype(create_sumcheck_tuple_of_tuples_of_univariates<Relations>());
    using TupleOfArraysOfValues = decltype(create_tuple_of_arrays_of_values<Relations>());

  private:
    /**
     * @brief A base class labelling precomputed entities and (ordered) subsets of interest.
     * @details Used to build the proving key and verification key.
     */
    template <typename DataType_> class PrecomputedEntities : public PrecomputedEntitiesBase {
      public:
        using DataType = DataType_;
        DEFINE_FLAVOR_MEMBERS(DataType,
                              lagrange_first,                   // column 0
                              enable_tuple_set_permutation,     // column 1
                              enable_single_column_permutation, // column 2
                              enable_first_set_permutation,     // column 3
                              enable_second_set_permutation)    // column 4

        RefVector<DataType> get_selectors()
        {
            return { lagrange_first,
                     enable_tuple_set_permutation,
                     enable_single_column_permutation,
                     enable_first_set_permutation,
                     enable_second_set_permutation };
        };
        RefVector<DataType> get_sigma_polynomials() { return {}; };
        RefVector<DataType> get_id_polynomials() { return {}; };
        RefVector<DataType> get_table_polynomials() { return {}; };
    };

    /**
     * @brief Container for all witness polynomials used/constructed by the prover.
     * @details Shifts are not included here since they do not occupy their own memory.
     */

    template <typename DataType> class WitnessEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              permutation_set_column_1,    // Column 0
                              permutation_set_column_2,    // Column 1
                              permutation_set_column_3,    // Column 2
                              permutation_set_column_4,    // Column 3
                              self_permutation_column,     // Column 4
                              tuple_permutation_inverses,  // Column 5
                              single_permutation_inverses) // Column 6

        RefVector<DataType> get_wires()
        {
            return { permutation_set_column_1,
                     permutation_set_column_2,
                     permutation_set_column_3,
                     permutation_set_column_4,
                     self_permutation_column };
        };
    };

    /**
     * @brief A base class labelling all entities (for instance, all of the polynomials used by the prover during
     * sumcheck) in this Honk variant along with particular subsets of interest
     * @details Used to build containers for: the prover's polynomial during sumcheck; the sumcheck's folded
     * polynomials; the univariates consturcted during during sumcheck; the evaluations produced by sumcheck.
     *
     * Symbolically we have: AllEntities = PrecomputedEntities + WitnessEntities + "ShiftedEntities". It could be
     * implemented as such, but we have this now.
     */

    template <typename DataType> class AllEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              lagrange_first,                   // Column 0
                              enable_tuple_set_permutation,     // Column 1
                              enable_single_column_permutation, // Column 2
                              enable_first_set_permutation,     // Column 3
                              enable_second_set_permutation,    // Column 4
                              permutation_set_column_1,         // Column 5
                              permutation_set_column_2,         // Column 6
                              permutation_set_column_3,         // Column 7
                              permutation_set_column_4,         // Column 8
                              self_permutation_column,          // Column 9
                              tuple_permutation_inverses,       // Column 10
                              single_permutation_inverses)      // Column 11

        RefVector<DataType> get_wires()
        {
            return {
                permutation_set_column_1, permutation_set_column_2, permutation_set_column_3, permutation_set_column_4
            };
        };
        RefVector<DataType> get_unshifted()
        {
            return { lagrange_first,
                     enable_tuple_set_permutation,
                     enable_single_column_permutation,
                     enable_first_set_permutation,
                     enable_second_set_permutation,
                     permutation_set_column_1,
                     permutation_set_column_2,
                     permutation_set_column_3,
                     permutation_set_column_4,
                     self_permutation_column,
                     tuple_permutation_inverses,
                     single_permutation_inverses };
        };
        RefVector<DataType> get_to_be_shifted() { return {}; };
        RefVector<DataType> get_shifted() { return {}; };
    };

  public:
    /**
     * @brief The proving key is responsible for storing the polynomials used by the prover.
     * @note TODO(Cody): Maybe multiple inheritance is the right thing here. In that case, nothing should eve inherit
     * from ProvingKey.
     */
    class ProvingKey : public ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>> {
      public:
        // Expose constructors on the base class
        using Base = ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>>;
        using Base::Base;

        // The plookup wires that store plookup read data.
        std::array<PolynomialHandle, 3> get_table_column_wires() { return {}; };
    };

    /**
     * @brief The verification key is responsible for storing the the commitments to the precomputed (non-witnessk)
     * polynomials used by the verifier.
     *
     * @note Note the discrepancy with what sort of data is stored here vs in the proving key. We may want to resolve
     * that, and split out separate PrecomputedPolynomials/Commitments data for clarity but also for portability of our
     * circuits.
     */
    using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment>>;

    /**
     * @brief A field element for each entity of the flavor.  These entities represent the prover polynomials evaluated
     * at one point.
     */
    class AllValues : public AllEntities<FF> {
      public:
        using Base = AllEntities<FF>;
        using Base::Base;
    };

    /**
     * @brief An owning container of polynomials.
     * @warning When this was introduced it broke some of our design principles.
     *   - Execution trace builders don't handle "polynomials" because the interpretation of the execution trace
     * columns as polynomials is a detail of the proving system, and trace builders are (sometimes in practice,
     * always in principle) reusable for different proving protocols (e.g., Plonk and Honk).
     *   - Polynomial storage is handled by key classes. Polynomials aren't moved, but are accessed elsewhere by
     * std::spans.
     *
     *  We will consider revising this data model: TODO(https://github.com/AztecProtocol/barretenberg/issues/743)
     */
    class AllPolynomials : public AllEntities<Polynomial> {
      public:
        [[nodiscard]] size_t get_polynomial_size() const { return this->lagrange_first.size(); }
        AllValues get_row(const size_t row_idx) const
        {
            AllValues result;
            for (auto [result_field, polynomial] : zip_view(result.get_all(), this->get_all())) {
                result_field = polynomial[row_idx];
            }
            return result;
        }
    };
    /**
     * @brief A container for polynomials handles; only stores spans.
     */
    class ProverPolynomials : public AllEntities<PolynomialHandle> {
      public:
        [[nodiscard]] size_t get_polynomial_size() const { return enable_tuple_set_permutation.size(); }
        [[nodiscard]] AllValues get_row(const size_t row_idx) const
        {
            AllValues result;
            for (auto [result_field, polynomial] : zip_view(result.get_all(), get_all())) {
                result_field = polynomial[row_idx];
            }
            return result;
        }
    };

    /**
     * @brief A container for storing the partially evaluated multivariates produced by sumcheck.
     */
    class PartiallyEvaluatedMultivariates : public AllEntities<Polynomial> {

      public:
        PartiallyEvaluatedMultivariates() = default;
        PartiallyEvaluatedMultivariates(const size_t circuit_size)
        {
            // Storage is only needed after the first partial evaluation, hence polynomials of size (n / 2)
            for (auto& poly : this->get_all()) {
                poly = Polynomial(circuit_size / 2);
            }
        }
    };
    /**
     * @brief A container for univariates used during Protogalaxy folding and sumcheck.
     * @details During folding and sumcheck, the prover evaluates the relations on these univariates.
     */
    template <size_t LENGTH> using ProverUnivariates = AllEntities<barretenberg::Univariate<FF, LENGTH>>;

    /**
     * @brief A container for univariates produced during the hot loop in sumcheck.
     */
    using ExtendedEdges = ProverUnivariates<MAX_PARTIAL_RELATION_LENGTH>;

    /**
     * @brief A container for the witness commitments.
     */

    using WitnessCommitments = WitnessEntities<Commitment>;

    /**
     * @brief A container for commitment labels.
     * @note It's debatable whether this should inherit from AllEntities. since most entries are not strictly needed. It
     * has, however, been useful during debugging to have these labels available.
     *
     */
    class CommitmentLabels : public AllEntities<std::string> {
      private:
        using Base = AllEntities<std::string>;

      public:
        CommitmentLabels()
            : AllEntities<std::string>()
        {
            Base::permutation_set_column_1 = "PERMUTATION_SET_COLUMN_1";
            Base::permutation_set_column_2 = "PERMUTATION_SET_COLUMN_2";
            Base::permutation_set_column_3 = "PERMUTATION_SET_COLUMN_3";
            Base::permutation_set_column_4 = "PERMUTATION_SET_COLUMN_4";
            Base::self_permutation_column = "SELF_PERMUTATION_COLUMN";
            Base::tuple_permutation_inverses = "TUPLE_PERMUTATION_INVERSES";
            Base::single_permutation_inverses = "SINGLE_PERMUTATION_INVERSES";
            // The ones beginning with "__" are only used for debugging
            Base::lagrange_first = "__LAGRANGE_FIRST";
            Base::enable_tuple_set_permutation = "__ENABLE_SET_PERMUTATION";
            Base::enable_single_column_permutation = "__ENABLE_SINGLE_COLUMN_PERMUTATION";
            Base::enable_first_set_permutation = "__ENABLE_FIRST_SET_PERMUTATION";
            Base::enable_second_set_permutation = "__ENABLE_SECOND_SET_PERMUTATION";
        };
    };

    class VerifierCommitments : public AllEntities<Commitment> {

      public:
        VerifierCommitments(const std::shared_ptr<VerificationKey>& verification_key)
        {
            lagrange_first = verification_key->lagrange_first;
            enable_tuple_set_permutation = verification_key->enable_tuple_set_permutation;
            enable_single_column_permutation = verification_key->enable_single_column_permutation;
            enable_first_set_permutation = verification_key->enable_first_set_permutation;
            enable_second_set_permutation = verification_key->enable_second_set_permutation;
        }
    };

    /**
     * @brief Derived class that defines proof structure for ECCVM proofs, as well as supporting functions.
     *
     */
    class Transcript : public BaseTranscript {
      public:
        uint32_t circuit_size;
        Commitment column_0_comm;
        Commitment column_1_comm;
        Commitment permutation_inverses_comm;
        std::vector<barretenberg::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>> sumcheck_univariates;
        std::array<FF, NUM_ALL_ENTITIES> sumcheck_evaluations;

        std::vector<Commitment> zm_cq_comms;
        Commitment zm_cq_comm;
        Commitment zm_pi_comm;

        Transcript() = default;

        Transcript(const std::vector<uint8_t>& proof)
            : BaseTranscript(proof)
        {}

        void deserialize_full_transcript()
        {
            // TODO. Codepath is dead for now, because there is no composer
            abort();
            // take current proof and put them into the struct
        }

        void serialize_full_transcript()
        {
            // TODO. Codepath is dead for now, because there is no composer
            abort();
        }
    };
};

// NOLINTEND(cppcoreguidelines-avoid-const-or-ref-data-members)

} // namespace flavor
namespace sumcheck {

DECLARE_IMPLEMENTATIONS_FOR_ALL_SETTINGS(GenericPermutationRelationImpl, flavor::ToyAVM)

} // namespace sumcheck
} // namespace proof_system::honk
