#pragma once
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/honk/pcs/ipa/ipa.hpp"
#include "barretenberg/polynomials/barycentric.hpp"
#include "barretenberg/polynomials/univariate.hpp"

#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"
#include "barretenberg/proof_system/relations/arithmetic_relation.hpp"
#include "barretenberg/proof_system/relations/permutation_relation.hpp"
#include <array>
#include <concepts>
#include <span>
#include <string>
#include <type_traits>
#include <vector>

namespace proof_system::honk::flavor {
class StandardGrumpkin {
    // TODO(Mara): At the moment this class is a duplicate of the Standard flavor with a different PCS for testing
    // purposes. This will be changed to Grumpkin once generating Honk proofs over Grumpkin has been enabled.
  public:
    using CircuitBuilder = StandardGrumpkinCircuitBuilder;
    using Curve = curve::Grumpkin;
    using PCS = pcs::ipa::IPA<Curve>;
    using GroupElement = Curve::Element;
    using Commitment = Curve::AffineElement;
    using CommitmentHandle = Curve::AffineElement;
    using FF = Curve::ScalarField;
    using Polynomial = barretenberg::Polynomial<FF>;
    using PolynomialHandle = std::span<FF>;
    using CommitmentKey = pcs::CommitmentKey<Curve>;
    using VerifierCommitmentKey = pcs::VerifierCommitmentKey<Curve>;

    static constexpr size_t NUM_WIRES = CircuitBuilder::NUM_WIRES;
    // The number of multivariate polynomials on which a sumcheck prover sumcheck operates (including shifts). We often
    // need containers of this size to hold related data, so we choose a name more agnostic than `NUM_POLYNOMIALS`
    static constexpr size_t NUM_ALL_ENTITIES = 18;
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 13;
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES = 4;

    // define the tuple of Relations that require grand products
    using GrandProductRelations = std::tuple<proof_system::PermutationRelation<FF>>;
    // define the tuple of Relations that comprise the Sumcheck relation
    using Relations = std::tuple<proof_system::ArithmeticRelation<FF>, proof_system::PermutationRelation<FF>>;

    static constexpr size_t MAX_RELATION_LENGTH = get_max_relation_length<Relations>();

    // MAX_RANDOM_RELATION_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta` random
    // polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation length = 3
    static constexpr size_t MAX_RANDOM_RELATION_LENGTH = MAX_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size<Relations>::value;

    // define the containers for storing the contributions from each relation in Sumcheck
    using RelationUnivariates = decltype(create_relation_univariates_container<FF, Relations>());
    using RelationValues = decltype(create_relation_values_container<FF, Relations>());

    // Whether or not the first row of the execution trace is reserved for 0s to enable shifts
    static constexpr bool has_zero_row = false;

  private:
    /**
     * @brief A base class labelling precomputed entities and (ordered) subsets of interest.
     * @details Used to build the proving key and verification key.
     */
    template <typename DataType, typename HandleType>
    class PrecomputedEntities : public PrecomputedEntities_<DataType, HandleType, NUM_PRECOMPUTED_ENTITIES> {
      public:
        DataType& q_m = std::get<0>(this->_data);
        DataType& q_l = std::get<1>(this->_data);
        DataType& q_r = std::get<2>(this->_data);
        DataType& q_o = std::get<3>(this->_data);
        DataType& q_c = std::get<4>(this->_data);
        DataType& sigma_1 = std::get<5>(this->_data);
        DataType& sigma_2 = std::get<6>(this->_data);
        DataType& sigma_3 = std::get<7>(this->_data);
        DataType& id_1 = std::get<8>(this->_data);
        DataType& id_2 = std::get<9>(this->_data);
        DataType& id_3 = std::get<10>(this->_data);
        DataType& lagrange_first = std::get<11>(this->_data);
        DataType& lagrange_last = std::get<12>(this->_data); // = LAGRANGE_N-1 whithout ZK, but can be less

        std::vector<HandleType> get_selectors() override { return { q_m, q_l, q_r, q_o, q_c }; };
        std::vector<HandleType> get_sigma_polynomials() override { return { sigma_1, sigma_2, sigma_3 }; };
        std::vector<HandleType> get_id_polynomials() override { return { id_1, id_2, id_3 }; };
    };

    /**
     * @brief Container for all witness polynomials used/constructed by the prover.
     * @details Shifts are not included here since they do not occupy their own memory.
     */
    template <typename DataType, typename HandleType>
    class WitnessEntities : public WitnessEntities_<DataType, HandleType, NUM_WITNESS_ENTITIES> {
      public:
        DataType& w_l = std::get<0>(this->_data);
        DataType& w_r = std::get<1>(this->_data);
        DataType& w_o = std::get<2>(this->_data);
        DataType& z_perm = std::get<3>(this->_data);

        std::vector<HandleType> get_wires() override { return { w_l, w_r, w_o }; };
    };

    /**
     * @brief A base class labelling all entities (for instance, all of the polynomials used by the prover during
     * sumcheck) in this Honk variant along with particular subsets of interest
     * @details Used to build containers for: the prover's polynomial during sumcheck; the sumcheck's folded
     * polynomials; the univariates consturcted during during sumcheck; the evaluations produced by sumcheck.
     *
     * Symbolically we have: AllEntities = PrecomputedEntities + WitnessEntities + "ShiftedEntities". It could be
     * implemented as such, but we don't have this now.
     */
    template <typename DataType, typename HandleType>
    class AllEntities : public AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES> {
      public:
        DataType& q_c = std::get<0>(this->_data);
        DataType& q_l = std::get<1>(this->_data);
        DataType& q_r = std::get<2>(this->_data);
        DataType& q_o = std::get<3>(this->_data);
        DataType& q_m = std::get<4>(this->_data);
        DataType& sigma_1 = std::get<5>(this->_data);
        DataType& sigma_2 = std::get<6>(this->_data);
        DataType& sigma_3 = std::get<7>(this->_data);
        DataType& id_1 = std::get<8>(this->_data);
        DataType& id_2 = std::get<9>(this->_data);
        DataType& id_3 = std::get<10>(this->_data);
        DataType& lagrange_first = std::get<11>(this->_data);
        DataType& lagrange_last = std::get<12>(this->_data);
        DataType& w_l = std::get<13>(this->_data);
        DataType& w_r = std::get<14>(this->_data);
        DataType& w_o = std::get<15>(this->_data);
        DataType& z_perm = std::get<16>(this->_data);
        DataType& z_perm_shift = std::get<17>(this->_data);

        std::vector<HandleType> get_wires() override { return { w_l, w_r, w_o }; };

        // Gemini-specific getters.
        std::vector<HandleType> get_unshifted() override
        {
            return { q_c,           q_l, q_r, q_o, q_m,   sigma_1, sigma_2, sigma_3, id_1, id_2, id_3, lagrange_first,
                     lagrange_last, w_l, w_r, w_o, z_perm };
        };
        std::vector<HandleType> get_to_be_shifted() override { return { z_perm }; };
        std::vector<HandleType> get_shifted() override { return { z_perm_shift }; };

        // TODO(Cody): It would be nice to define these constructors once in a base class template.
        AllEntities() = default;

        AllEntities(const AllEntities& other)
            : AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES>(other){};

        AllEntities(AllEntities&& other)
            : AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES>(other){};

        AllEntities& operator=(const AllEntities& other)
        {
            if (this == &other) {
                return *this;
            }
            AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES>::operator=(other);
            return *this;
        }

        AllEntities& operator=(AllEntities&& other)
        {
            AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES>::operator=(other);
            return *this;
        }

        ~AllEntities() = default;
    };

  public:
    /**
     * @brief The proving key is responsible for storing the polynomials used by the prover.
     * @note TODO(Cody): Maybe multiple inheritance is the right thing here. In that case, nothing should eve inherit
     * from ProvingKey.
     */
    class ProvingKey : public ProvingKey_<PrecomputedEntities<Polynomial, PolynomialHandle>,
                                          WitnessEntities<Polynomial, PolynomialHandle>> {
      public:
        // Expose constructors of the base class
        using Base = ProvingKey_<PrecomputedEntities<Polynomial, PolynomialHandle>,
                                 WitnessEntities<Polynomial, PolynomialHandle>>;
        using Base::Base;
    };

    /**
     * @brief The verification key is responsible for storing the the commitments to the precomputed (non-witness)
     * polynomials used by the verifier.
     *
     * @note Note the discrepancy with what sort of data is stored here vs in the proving key. We may want to resolve
     * that, and split out separate PrecomputedPolynomials/Commitments data for clarity but also for portability of our
     * circuits.
     */
    using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment, CommitmentHandle>>;

    /**
     * @brief A container for polynomials handles; only stores spans.
     */
    using ProverPolynomials = AllEntities<PolynomialHandle, PolynomialHandle>;

    /**
     * @brief A container for storing the partially evaluated multivariates produced by sumcheck.
     */
    class PartiallyEvaluatedMultivariates : public AllEntities<Polynomial, PolynomialHandle> {

      public:
        PartiallyEvaluatedMultivariates() = default;
        PartiallyEvaluatedMultivariates(const size_t circuit_size)
        {
            // Storage is only needed after the first partial evaluation, hence polynomials of size (n / 2)
            for (auto& poly : this->_data) {
                poly = Polynomial(circuit_size / 2);
            }
        }
    };

    /**
     * @brief A container for univariates produced during the hot loop in sumcheck.
     * @todo TODO(#390): Simplify this by moving MAX_RELATION_LENGTH?
     */
    template <size_t MAX_RELATION_LENGTH>
    using ExtendedEdges = AllEntities<barretenberg::Univariate<FF, MAX_RELATION_LENGTH>,
                                      barretenberg::Univariate<FF, MAX_RELATION_LENGTH>>;

    /**
     * @brief A container for the polynomials evaluations produced during sumcheck, which are purported to be the
     * evaluations of polynomials committed in earlier rounds.
     */
    class ClaimedEvaluations : public AllEntities<FF, FF> {
      public:
        using Base = AllEntities<FF, FF>;
        using Base::Base;
        ClaimedEvaluations(std::array<FF, NUM_ALL_ENTITIES> _data_in) { this->_data = _data_in; }
    };

    /**
     * @brief A container for commitment labels.
     * @note It's debatable whether this should inherit from AllEntities. since most entries are not strictly needed. It
     * has, however, been useful during debugging to have these labels available.
     *
     */
    class CommitmentLabels : public AllEntities<std::string, std::string> {
      public:
        CommitmentLabels()
            : AllEntities<std::string, std::string>()
        {
            w_l = "W_1";
            w_r = "W_2";
            w_o = "W_3";
            z_perm = "Z_PERM";
            // The ones beginning with "__" are only used for debugging
            z_perm_shift = "__Z_PERM_SHIFT";
            q_m = "__Q_M";
            q_l = "__Q_L";
            q_r = "__Q_R";
            q_o = "__Q_O";
            q_c = "__Q_C";
            sigma_1 = "__SIGMA_1";
            sigma_2 = "__SIGMA_2";
            sigma_3 = "__SIGMA_3";
            id_1 = "__ID_1";
            id_2 = "__ID_2";
            id_3 = "__ID_3";
            lagrange_first = "__LAGRANGE_FIRST";
            lagrange_last = "__LAGRANGE_LAST";
        };
    };

    /**
     * @brief A container for all commitments used by the verifier.
     */
    class VerifierCommitments : public AllEntities<Commitment, CommitmentHandle> {
      public:
        VerifierCommitments(std::shared_ptr<VerificationKey> verification_key)
        {
            // Initialize pre-computed commitments here, witness commitments during proof verification.
            q_m = verification_key->q_m;
            q_l = verification_key->q_l;
            q_r = verification_key->q_r;
            q_o = verification_key->q_o;
            q_c = verification_key->q_c;
            sigma_1 = verification_key->sigma_1;
            sigma_2 = verification_key->sigma_2;
            sigma_3 = verification_key->sigma_3;
            id_1 = verification_key->id_1;
            id_2 = verification_key->id_2;
            id_3 = verification_key->id_3;
            lagrange_first = verification_key->lagrange_first;
            lagrange_last = verification_key->lagrange_last;
        }
    };
};
} // namespace proof_system::honk::flavor