#pragma once
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/honk/pcs/kzg/kzg.hpp"
#include "barretenberg/honk/sumcheck/polynomials/barycentric_data.hpp"
#include "barretenberg/honk/sumcheck/polynomials/univariate.hpp"
#include "barretenberg/honk/sumcheck/relations/auxiliary_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/ecc_op_queue_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/elliptic_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/gen_perm_sort_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/lookup_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/permutation_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include <array>
#include <concepts>
#include <span>
#include <string>
#include <type_traits>
#include <vector>

namespace proof_system::honk::flavor {

class GoblinUltra {
  public:
    using CircuitBuilder = UltraCircuitBuilder;
    using Curve = curve::BN254;
    using PCS = pcs::kzg::KZG<Curve>;
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
    // need containers of this size to hold related data, so we choose a name more agnostic than `NUM_POLYNOMIALS`.
    // Note: this number does not include the individual sorted list polynomials.
    static constexpr size_t NUM_ALL_ENTITIES = 48; // 43 (UH) + 4 op wires + 1 op wire "selector"
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 26; // 25 (UH) + 1 op wire "selector"
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES = 15; // 11 (UH) + 4 op wires

    using GrandProductRelations = std::tuple<sumcheck::UltraPermutationRelation<FF>, sumcheck::LookupRelation<FF>>;

    // define the tuple of Relations that comprise the Sumcheck relation
    using Relations = std::tuple<sumcheck::UltraArithmeticRelation<FF>,
                                 sumcheck::UltraPermutationRelation<FF>,
                                 sumcheck::LookupRelation<FF>,
                                 sumcheck::GenPermSortRelation<FF>,
                                 sumcheck::EllipticRelation<FF>,
                                 sumcheck::AuxiliaryRelation<FF>,
                                 sumcheck::EccOpQueueRelation<FF>>;

    static constexpr size_t MAX_RELATION_LENGTH = get_max_relation_length<Relations>();

    // MAX_RANDOM_RELATION_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta` random
    // polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation length = 3
    static constexpr size_t MAX_RANDOM_RELATION_LENGTH = MAX_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size<Relations>::value;

    // define the container for storing the univariate contribution from each relation in Sumcheck
    using RelationUnivariates = decltype(create_relation_univariates_container<FF, Relations>());
    using RelationValues = decltype(create_relation_values_container<FF, Relations>());

    // Whether or not the first row of the execution trace is reserved for 0s to enable shifts
    static constexpr bool has_zero_row = true;

  private:
    template <typename DataType, typename HandleType>
    /**
     * @brief A base class labelling precomputed entities and (ordered) subsets of interest.
     * @details Used to build the proving key and verification key.
     */
    class PrecomputedEntities : public PrecomputedEntities_<DataType, HandleType, NUM_PRECOMPUTED_ENTITIES> {
      public:
        DataType& q_m = std::get<0>(this->_data);
        DataType& q_c = std::get<1>(this->_data);
        DataType& q_l = std::get<2>(this->_data);
        DataType& q_r = std::get<3>(this->_data);
        DataType& q_o = std::get<4>(this->_data);
        DataType& q_4 = std::get<5>(this->_data);
        DataType& q_arith = std::get<6>(this->_data);
        DataType& q_sort = std::get<7>(this->_data);
        DataType& q_elliptic = std::get<8>(this->_data);
        DataType& q_aux = std::get<9>(this->_data);
        DataType& q_lookup = std::get<10>(this->_data);
        DataType& sigma_1 = std::get<11>(this->_data);
        DataType& sigma_2 = std::get<12>(this->_data);
        DataType& sigma_3 = std::get<13>(this->_data);
        DataType& sigma_4 = std::get<14>(this->_data);
        DataType& id_1 = std::get<15>(this->_data);
        DataType& id_2 = std::get<16>(this->_data);
        DataType& id_3 = std::get<17>(this->_data);
        DataType& id_4 = std::get<18>(this->_data);
        DataType& table_1 = std::get<19>(this->_data);
        DataType& table_2 = std::get<20>(this->_data);
        DataType& table_3 = std::get<21>(this->_data);
        DataType& table_4 = std::get<22>(this->_data);
        DataType& lagrange_first = std::get<23>(this->_data);
        DataType& lagrange_last = std::get<24>(this->_data);
        DataType& lagrange_ecc_op = std::get<25>(this->_data); // indicator poly for ecc op gates

        static constexpr CircuitType CIRCUIT_TYPE = CircuitBuilder::CIRCUIT_TYPE;

        std::vector<HandleType> get_selectors() override
        {
            return { q_m, q_c, q_l, q_r, q_o, q_4, q_arith, q_sort, q_elliptic, q_aux, q_lookup };
        };
        std::vector<HandleType> get_sigma_polynomials() override { return { sigma_1, sigma_2, sigma_3, sigma_4 }; };
        std::vector<HandleType> get_id_polynomials() override { return { id_1, id_2, id_3, id_4 }; };

        std::vector<HandleType> get_table_polynomials() { return { table_1, table_2, table_3, table_4 }; };
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
        DataType& w_4 = std::get<3>(this->_data);
        DataType& sorted_1 = std::get<4>(this->_data);
        DataType& sorted_2 = std::get<5>(this->_data);
        DataType& sorted_3 = std::get<6>(this->_data);
        DataType& sorted_4 = std::get<7>(this->_data);
        DataType& sorted_accum = std::get<8>(this->_data);
        DataType& z_perm = std::get<9>(this->_data);
        DataType& z_lookup = std::get<10>(this->_data);
        DataType& ecc_op_wire_1 = std::get<11>(this->_data);
        DataType& ecc_op_wire_2 = std::get<12>(this->_data);
        DataType& ecc_op_wire_3 = std::get<13>(this->_data);
        DataType& ecc_op_wire_4 = std::get<14>(this->_data);

        std::vector<HandleType> get_wires() override { return { w_l, w_r, w_o, w_4 }; };
        std::vector<HandleType> get_ecc_op_wires()
        {
            return { ecc_op_wire_1, ecc_op_wire_2, ecc_op_wire_3, ecc_op_wire_4 };
        };
        // The sorted concatenations of table and witness data needed for plookup.
        std::vector<HandleType> get_sorted_polynomials() { return { sorted_1, sorted_2, sorted_3, sorted_4 }; };
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
    template <typename DataType, typename HandleType>
    class AllEntities : public AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES> {
      public:
        DataType& q_c = std::get<0>(this->_data);
        DataType& q_l = std::get<1>(this->_data);
        DataType& q_r = std::get<2>(this->_data);
        DataType& q_o = std::get<3>(this->_data);
        DataType& q_4 = std::get<4>(this->_data);
        DataType& q_m = std::get<5>(this->_data);
        DataType& q_arith = std::get<6>(this->_data);
        DataType& q_sort = std::get<7>(this->_data);
        DataType& q_elliptic = std::get<8>(this->_data);
        DataType& q_aux = std::get<9>(this->_data);
        DataType& q_lookup = std::get<10>(this->_data);
        DataType& sigma_1 = std::get<11>(this->_data);
        DataType& sigma_2 = std::get<12>(this->_data);
        DataType& sigma_3 = std::get<13>(this->_data);
        DataType& sigma_4 = std::get<14>(this->_data);
        DataType& id_1 = std::get<15>(this->_data);
        DataType& id_2 = std::get<16>(this->_data);
        DataType& id_3 = std::get<17>(this->_data);
        DataType& id_4 = std::get<18>(this->_data);
        DataType& table_1 = std::get<19>(this->_data);
        DataType& table_2 = std::get<20>(this->_data);
        DataType& table_3 = std::get<21>(this->_data);
        DataType& table_4 = std::get<22>(this->_data);
        DataType& lagrange_first = std::get<23>(this->_data);
        DataType& lagrange_last = std::get<24>(this->_data);
        DataType& lagrange_ecc_op = std::get<25>(this->_data);
        DataType& w_l = std::get<26>(this->_data);
        DataType& w_r = std::get<27>(this->_data);
        DataType& w_o = std::get<28>(this->_data);
        DataType& w_4 = std::get<29>(this->_data);
        DataType& sorted_accum = std::get<30>(this->_data);
        DataType& z_perm = std::get<31>(this->_data);
        DataType& z_lookup = std::get<32>(this->_data);
        DataType& ecc_op_wire_1 = std::get<33>(this->_data);
        DataType& ecc_op_wire_2 = std::get<34>(this->_data);
        DataType& ecc_op_wire_3 = std::get<35>(this->_data);
        DataType& ecc_op_wire_4 = std::get<36>(this->_data);
        DataType& table_1_shift = std::get<37>(this->_data);
        DataType& table_2_shift = std::get<38>(this->_data);
        DataType& table_3_shift = std::get<39>(this->_data);
        DataType& table_4_shift = std::get<40>(this->_data);
        DataType& w_l_shift = std::get<41>(this->_data);
        DataType& w_r_shift = std::get<42>(this->_data);
        DataType& w_o_shift = std::get<43>(this->_data);
        DataType& w_4_shift = std::get<44>(this->_data);
        DataType& sorted_accum_shift = std::get<45>(this->_data);
        DataType& z_perm_shift = std::get<46>(this->_data);
        DataType& z_lookup_shift = std::get<47>(this->_data);

        std::vector<HandleType> get_wires() override { return { w_l, w_r, w_o, w_4 }; };
        std::vector<HandleType> get_ecc_op_wires()
        {
            return { ecc_op_wire_1, ecc_op_wire_2, ecc_op_wire_3, ecc_op_wire_4 };
        };
        // Gemini-specific getters.
        std::vector<HandleType> get_unshifted() override
        {
            return { q_c,           q_l,
                     q_r,           q_o,
                     q_4,           q_m,
                     q_arith,       q_sort,
                     q_elliptic,    q_aux,
                     q_lookup,      sigma_1,
                     sigma_2,       sigma_3,
                     sigma_4,       id_1,
                     id_2,          id_3,
                     id_4,          table_1,
                     table_2,       table_3,
                     table_4,       lagrange_first,
                     lagrange_last, lagrange_ecc_op,
                     w_l,           w_r,
                     w_o,           w_4,
                     sorted_accum,  z_perm,
                     z_lookup,      ecc_op_wire_1,
                     ecc_op_wire_2, ecc_op_wire_3,
                     ecc_op_wire_4 };
        };
        std::vector<HandleType> get_to_be_shifted() override
        {
            return { table_1, table_2, table_3, table_4, w_l, w_r, w_o, w_4, sorted_accum, z_perm, z_lookup };
        };
        std::vector<HandleType> get_shifted() override
        {
            return { table_1_shift, table_2_shift, table_3_shift,      table_4_shift, w_l_shift,     w_r_shift,
                     w_o_shift,     w_4_shift,     sorted_accum_shift, z_perm_shift,  z_lookup_shift };
        };

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
        // Expose constructors on the base class
        using Base = ProvingKey_<PrecomputedEntities<Polynomial, PolynomialHandle>,
                                 WitnessEntities<Polynomial, PolynomialHandle>>;
        using Base::Base;

        std::vector<uint32_t> memory_read_records;
        std::vector<uint32_t> memory_write_records;

        size_t num_ecc_op_gates; // needed to determine public input offset

        // The plookup wires that store plookup read data.
        std::array<PolynomialHandle, 3> get_table_column_wires() { return { w_l, w_r, w_o }; };
    };

    /**
     * @brief The verification key is responsible for storing the the commitments to the precomputed (non-witnessk)
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
    using ExtendedEdges =
        AllEntities<sumcheck::Univariate<FF, MAX_RELATION_LENGTH>, sumcheck::Univariate<FF, MAX_RELATION_LENGTH>>;

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
        {
            w_l = "W_L";
            w_r = "W_R";
            w_o = "W_O";
            w_4 = "W_4";
            z_perm = "Z_PERM";
            z_lookup = "Z_LOOKUP";
            sorted_accum = "SORTED_ACCUM";
            ecc_op_wire_1 = "ECC_OP_WIRE_1";
            ecc_op_wire_2 = "ECC_OP_WIRE_2";
            ecc_op_wire_3 = "ECC_OP_WIRE_3";
            ecc_op_wire_4 = "ECC_OP_WIRE_4";

            // The ones beginning with "__" are only used for debugging
            q_c = "__Q_C";
            q_l = "__Q_L";
            q_r = "__Q_R";
            q_o = "__Q_O";
            q_4 = "__Q_4";
            q_m = "__Q_M";
            q_arith = "__Q_ARITH";
            q_sort = "__Q_SORT";
            q_elliptic = "__Q_ELLIPTIC";
            q_aux = "__Q_AUX";
            q_lookup = "__Q_LOOKUP";
            sigma_1 = "__SIGMA_1";
            sigma_2 = "__SIGMA_2";
            sigma_3 = "__SIGMA_3";
            sigma_4 = "__SIGMA_4";
            id_1 = "__ID_1";
            id_2 = "__ID_2";
            id_3 = "__ID_3";
            id_4 = "__ID_4";
            table_1 = "__TABLE_1";
            table_2 = "__TABLE_2";
            table_3 = "__TABLE_3";
            table_4 = "__TABLE_4";
            lagrange_first = "__LAGRANGE_FIRST";
            lagrange_last = "__LAGRANGE_LAST";
            lagrange_ecc_op = "__Q_ECC_OP_QUEUE";
        };
    };

    class VerifierCommitments : public AllEntities<Commitment, CommitmentHandle> {
      public:
        VerifierCommitments(std::shared_ptr<VerificationKey> verification_key, VerifierTranscript<FF> transcript)
        {
            static_cast<void>(transcript);
            q_m = verification_key->q_m;
            q_l = verification_key->q_l;
            q_r = verification_key->q_r;
            q_o = verification_key->q_o;
            q_4 = verification_key->q_4;
            q_c = verification_key->q_c;
            q_arith = verification_key->q_arith;
            q_sort = verification_key->q_sort;
            q_elliptic = verification_key->q_elliptic;
            q_aux = verification_key->q_aux;
            q_lookup = verification_key->q_lookup;
            sigma_1 = verification_key->sigma_1;
            sigma_2 = verification_key->sigma_2;
            sigma_3 = verification_key->sigma_3;
            sigma_4 = verification_key->sigma_4;
            id_1 = verification_key->id_1;
            id_2 = verification_key->id_2;
            id_3 = verification_key->id_3;
            id_4 = verification_key->id_4;
            table_1 = verification_key->table_1;
            table_2 = verification_key->table_2;
            table_3 = verification_key->table_3;
            table_4 = verification_key->table_4;
            lagrange_first = verification_key->lagrange_first;
            lagrange_last = verification_key->lagrange_last;
            lagrange_ecc_op = verification_key->lagrange_ecc_op;
        }
    };
};

} // namespace proof_system::honk::flavor
