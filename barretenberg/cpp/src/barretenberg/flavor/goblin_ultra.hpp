#pragma once
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/relations/auxiliary_relation.hpp"
#include "barretenberg/relations/databus_lookup_relation.hpp"
#include "barretenberg/relations/ecc_op_queue_relation.hpp"
#include "barretenberg/relations/elliptic_relation.hpp"
#include "barretenberg/relations/gen_perm_sort_relation.hpp"
#include "barretenberg/relations/lookup_relation.hpp"
#include "barretenberg/relations/permutation_relation.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/transcript/transcript.hpp"
#include "relation_definitions_fwd.hpp"

namespace proof_system::honk::flavor {

class GoblinUltra {
  public:
    using CircuitBuilder = GoblinUltraCircuitBuilder;
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

    static constexpr size_t NUM_WIRES = CircuitBuilder::NUM_WIRES;
    // The number of multivariate polynomials on which a sumcheck prover sumcheck operates (including shifts). We often
    // need containers of this size to hold related data, so we choose a name more agnostic than `NUM_POLYNOMIALS`.
    // Note: this number does not include the individual sorted list polynomials.
    static constexpr size_t NUM_ALL_ENTITIES = 53;
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 28;
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES = 18;

    using GrandProductRelations =
        std::tuple<proof_system::UltraPermutationRelation<FF>, proof_system::LookupRelation<FF>>;

    // define the tuple of Relations that comprise the Sumcheck relation
    using Relations = std::tuple<proof_system::UltraArithmeticRelation<FF>,
                                 proof_system::UltraPermutationRelation<FF>,
                                 proof_system::LookupRelation<FF>,
                                 proof_system::GenPermSortRelation<FF>,
                                 proof_system::EllipticRelation<FF>,
                                 proof_system::AuxiliaryRelation<FF>,
                                 proof_system::EccOpQueueRelation<FF>,
                                 proof_system::DatabusLookupRelation<FF>>;

    using LogDerivLookupRelation = proof_system::DatabusLookupRelation<FF>;

    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_partial_relation_length<Relations>();
    static constexpr size_t MAX_TOTAL_RELATION_LENGTH = compute_max_total_relation_length<Relations>();
    static constexpr size_t NUMBER_OF_SUBRELATIONS = compute_number_of_subrelations<Relations>();

    // BATCHED_RELATION_PARTIAL_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta`
    // random polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation
    // length = 3
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = MAX_PARTIAL_RELATION_LENGTH + 1;
    static constexpr size_t BATCHED_RELATION_TOTAL_LENGTH = MAX_TOTAL_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size_v<Relations>;

    template <size_t NUM_INSTANCES>
    using ProtogalaxyTupleOfTuplesOfUnivariates =
        decltype(create_protogalaxy_tuple_of_tuples_of_univariates<Relations, NUM_INSTANCES>());
    using SumcheckTupleOfTuplesOfUnivariates = decltype(create_sumcheck_tuple_of_tuples_of_univariates<Relations>());
    using TupleOfArraysOfValues = decltype(create_tuple_of_arrays_of_values<Relations>());

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
        DataType q_m;             // column 0
        DataType q_c;             // column 1
        DataType q_l;             // column 2
        DataType q_r;             // column 3
        DataType q_o;             // column 4
        DataType q_4;             // column 5
        DataType q_arith;         // column 6
        DataType q_sort;          // column 7
        DataType q_elliptic;      // column 8
        DataType q_aux;           // column 9
        DataType q_lookup;        // column 10
        DataType q_busread;       // column 11
        DataType sigma_1;         // column 12
        DataType sigma_2;         // column 13
        DataType sigma_3;         // column 14
        DataType sigma_4;         // column 15
        DataType id_1;            // column 16
        DataType id_2;            // column 17
        DataType id_3;            // column 18
        DataType id_4;            // column 19
        DataType table_1;         // column 20
        DataType table_2;         // column 21
        DataType table_3;         // column 22
        DataType table_4;         // column 23
        DataType lagrange_first;  // column 24
        DataType lagrange_last;   // column 25
        DataType lagrange_ecc_op; // column 26 // indicator poly for ecc op gates
        DataType databus_id;      // column 27 // id polynomial, i.e. id_i = i

        DEFINE_POINTER_VIEW(NUM_PRECOMPUTED_ENTITIES,
                            &q_m,
                            &q_c,
                            &q_l,
                            &q_r,
                            &q_o,
                            &q_4,
                            &q_arith,
                            &q_sort,
                            &q_elliptic,
                            &q_aux,
                            &q_lookup,
                            &q_busread,
                            &sigma_1,
                            &sigma_2,
                            &sigma_3,
                            &sigma_4,
                            &id_1,
                            &id_2,
                            &id_3,
                            &id_4,
                            &table_1,
                            &table_2,
                            &table_3,
                            &table_4,
                            &lagrange_first,
                            &lagrange_last,
                            &lagrange_ecc_op,
                            &databus_id)

        static constexpr CircuitType CIRCUIT_TYPE = CircuitBuilder::CIRCUIT_TYPE;

        std::vector<HandleType> get_selectors() override
        {
            return { q_m, q_c, q_l, q_r, q_o, q_4, q_arith, q_sort, q_elliptic, q_aux, q_lookup, q_busread };
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
        DataType w_l;                  // column 0
        DataType w_r;                  // column 1
        DataType w_o;                  // column 2
        DataType w_4;                  // column 3
        DataType sorted_1;             // column 4
        DataType sorted_2;             // column 5
        DataType sorted_3;             // column 6
        DataType sorted_4;             // column 7
        DataType sorted_accum;         // column 8
        DataType z_perm;               // column 9
        DataType z_lookup;             // column 10
        DataType ecc_op_wire_1;        // column 11
        DataType ecc_op_wire_2;        // column 12
        DataType ecc_op_wire_3;        // column 13
        DataType ecc_op_wire_4;        // column 14
        DataType calldata;             // column 15
        DataType calldata_read_counts; // column 16
        DataType lookup_inverses;      // column 17

        DEFINE_POINTER_VIEW(NUM_WITNESS_ENTITIES,
                            &w_l,
                            &w_r,
                            &w_o,
                            &w_4,
                            &sorted_1,
                            &sorted_2,
                            &sorted_3,
                            &sorted_4,
                            &sorted_accum,
                            &z_perm,
                            &z_lookup,
                            &ecc_op_wire_1,
                            &ecc_op_wire_2,
                            &ecc_op_wire_3,
                            &ecc_op_wire_4,
                            &calldata,
                            &calldata_read_counts,
                            &lookup_inverses)

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
        DataType q_c;                  // column 0
        DataType q_l;                  // column 1
        DataType q_r;                  // column 2
        DataType q_o;                  // column 3
        DataType q_4;                  // column 4
        DataType q_m;                  // column 5
        DataType q_arith;              // column 6
        DataType q_sort;               // column 7
        DataType q_elliptic;           // column 8
        DataType q_aux;                // column 9
        DataType q_lookup;             // column 10
        DataType q_busread;            // column 11
        DataType sigma_1;              // column 12
        DataType sigma_2;              // column 13
        DataType sigma_3;              // column 14
        DataType sigma_4;              // column 15
        DataType id_1;                 // column 16
        DataType id_2;                 // column 17
        DataType id_3;                 // column 18
        DataType id_4;                 // column 19
        DataType table_1;              // column 20
        DataType table_2;              // column 21
        DataType table_3;              // column 22
        DataType table_4;              // column 23
        DataType lagrange_first;       // column 24
        DataType lagrange_last;        // column 25
        DataType lagrange_ecc_op;      // column 26
        DataType databus_id;           // column 27
        DataType w_l;                  // column 28
        DataType w_r;                  // column 29
        DataType w_o;                  // column 30
        DataType w_4;                  // column 31
        DataType sorted_accum;         // column 32
        DataType z_perm;               // column 33
        DataType z_lookup;             // column 34
        DataType ecc_op_wire_1;        // column 35
        DataType ecc_op_wire_2;        // column 36
        DataType ecc_op_wire_3;        // column 37
        DataType ecc_op_wire_4;        // column 38
        DataType calldata;             // column 39
        DataType calldata_read_counts; // column 40
        DataType lookup_inverses;      // column 41
        DataType table_1_shift;        // column 42
        DataType table_2_shift;        // column 43
        DataType table_3_shift;        // column 44
        DataType table_4_shift;        // column 45
        DataType w_l_shift;            // column 46
        DataType w_r_shift;            // column 47
        DataType w_o_shift;            // column 48
        DataType w_4_shift;            // column 49
        DataType sorted_accum_shift;   // column 50
        DataType z_perm_shift;         // column 51
        DataType z_lookup_shift;       // column 52

        // defines a method pointer_view that returns the following, with const and non-const variants
        DEFINE_POINTER_VIEW(NUM_ALL_ENTITIES,
                            &q_c,
                            &q_l,
                            &q_r,
                            &q_o,
                            &q_4,
                            &q_m,
                            &q_arith,
                            &q_sort,
                            &q_elliptic,
                            &q_aux,
                            &q_lookup,
                            &q_busread,
                            &sigma_1,
                            &sigma_2,
                            &sigma_3,
                            &sigma_4,
                            &id_1,
                            &id_2,
                            &id_3,
                            &id_4,
                            &table_1,
                            &table_2,
                            &table_3,
                            &table_4,
                            &lagrange_first,
                            &lagrange_last,
                            &lagrange_ecc_op,
                            &databus_id,
                            &w_l,
                            &w_r,
                            &w_o,
                            &w_4,
                            &sorted_accum,
                            &z_perm,
                            &z_lookup,
                            &ecc_op_wire_1,
                            &ecc_op_wire_2,
                            &ecc_op_wire_3,
                            &ecc_op_wire_4,
                            &calldata,
                            &calldata_read_counts,
                            &lookup_inverses,
                            &table_1_shift,
                            &table_2_shift,
                            &table_3_shift,
                            &table_4_shift,
                            &w_l_shift,
                            &w_r_shift,
                            &w_o_shift,
                            &w_4_shift,
                            &sorted_accum_shift,
                            &z_perm_shift,
                            &z_lookup_shift);

        std::vector<HandleType> get_wires() override { return { w_l, w_r, w_o, w_4 }; };
        std::vector<HandleType> get_ecc_op_wires()
        {
            return { ecc_op_wire_1, ecc_op_wire_2, ecc_op_wire_3, ecc_op_wire_4 };
        };
        // Gemini-specific getters.
        std::vector<HandleType> get_unshifted() override
        {
            return { q_c,
                     q_l,
                     q_r,
                     q_o,
                     q_4,
                     q_m,
                     q_arith,
                     q_sort,
                     q_elliptic,
                     q_aux,
                     q_lookup,
                     q_busread,
                     sigma_1,
                     sigma_2,
                     sigma_3,
                     sigma_4,
                     id_1,
                     id_2,
                     id_3,
                     id_4,
                     table_1,
                     table_2,
                     table_3,
                     table_4,
                     lagrange_first,
                     lagrange_last,
                     lagrange_ecc_op,
                     databus_id,
                     w_l,
                     w_r,
                     w_o,
                     w_4,
                     sorted_accum,
                     z_perm,
                     z_lookup,
                     ecc_op_wire_1,
                     ecc_op_wire_2,
                     ecc_op_wire_3,
                     ecc_op_wire_4,
                     calldata,
                     calldata_read_counts,
                     lookup_inverses };
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
     * @brief A container for storing the partially evaluated multivariates produced by sumcheck.
     */
    class PartiallyEvaluatedMultivariates : public AllEntities<Polynomial, PolynomialHandle> {

      public:
        PartiallyEvaluatedMultivariates() = default;
        PartiallyEvaluatedMultivariates(const size_t circuit_size)
        {
            // Storage is only needed after the first partial evaluation, hence polynomials of size (n / 2)
            for (auto* poly : pointer_view()) {
                *poly = Polynomial(circuit_size / 2);
            }
        }
    };

    /**
     * @brief A container for univariates used during Protogalaxy folding and sumcheck.
     * @details During folding and sumcheck, the prover evaluates the relations on these univariates.
     */
    template <size_t LENGTH>
    using ProverUnivariates = AllEntities<barretenberg::Univariate<FF, LENGTH>, barretenberg::Univariate<FF, LENGTH>>;

    /**
     * @brief A container for univariates produced during the hot loop in sumcheck.
     */
    using ExtendedEdges = ProverUnivariates<MAX_PARTIAL_RELATION_LENGTH>;

    /**
     * @brief A field element for each entity of the flavor. These entities represent the prover polynomials evaluated
     * at one point.
     */
    class AllValues : public AllEntities<FF, FF> {
      public:
        using Base = AllEntities<FF, FF>;
        using Base::Base;
    };

    /**
     * @brief A container for the prover polynomials handles; only stores spans.
     */
    class ProverPolynomials : public AllEntities<PolynomialHandle, PolynomialHandle> {
      public:
        [[nodiscard]] size_t get_polynomial_size() const { return q_c.size(); }
        [[nodiscard]] AllValues get_row(size_t row_idx) const
        {
            AllValues result;
            for (auto [result_field, polynomial] : zip_view(result.pointer_view(), pointer_view())) {
                *result_field = (*polynomial)[row_idx];
            }
            return result;
        }
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
            calldata = "CALLDATA";
            calldata_read_counts = "CALLDATA_READ_COUNTS";
            lookup_inverses = "LOOKUP_INVERSES";

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
            q_busread = "__Q_BUSREAD";
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
        VerifierCommitments(std::shared_ptr<VerificationKey> verification_key,
                            [[maybe_unused]] const BaseTranscript<FF>& transcript)
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
            q_busread = verification_key->q_busread;
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
            databus_id = verification_key->databus_id;
        }
    };

    class FoldingParameters {
      public:
        std::vector<FF> gate_separation_challenges;
        FF target_sum;
    };

    /**
     * @brief Derived class that defines proof structure for GoblinUltra proofs, as well as supporting functions.
     *
     */
    class Transcript : public BaseTranscript<FF> {
      public:
        uint32_t circuit_size;
        uint32_t public_input_size;
        uint32_t pub_inputs_offset;
        std::vector<FF> public_inputs;
        Commitment w_l_comm;
        Commitment w_r_comm;
        Commitment w_o_comm;
        Commitment ecc_op_wire_1_comm;
        Commitment ecc_op_wire_2_comm;
        Commitment ecc_op_wire_3_comm;
        Commitment ecc_op_wire_4_comm;
        Commitment calldata_comm;
        Commitment calldata_read_counts_comm;
        Commitment lookup_inverses_comm;
        Commitment sorted_accum_comm;
        Commitment w_4_comm;
        Commitment z_perm_comm;
        Commitment z_lookup_comm;
        std::vector<barretenberg::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>> sumcheck_univariates;
        std::array<FF, NUM_ALL_ENTITIES> sumcheck_evaluations;
        std::vector<Commitment> zm_cq_comms;
        Commitment zm_cq_comm;
        Commitment zm_pi_comm;

        Transcript() = default;

        Transcript(const std::vector<uint8_t>& proof)
            : BaseTranscript<FF>(proof)
        {}

        void deserialize_full_transcript() override
        {
            // take current proof and put them into the struct
            size_t num_bytes_read = 0;
            circuit_size = deserialize_from_buffer<uint32_t>(proof_data, num_bytes_read);
            size_t log_n = numeric::get_msb(circuit_size);

            public_input_size = deserialize_from_buffer<uint32_t>(proof_data, num_bytes_read);
            pub_inputs_offset = deserialize_from_buffer<uint32_t>(proof_data, num_bytes_read);
            for (size_t i = 0; i < public_input_size; ++i) {
                public_inputs.push_back(deserialize_from_buffer<FF>(proof_data, num_bytes_read));
            }
            w_l_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            w_r_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            w_o_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            ecc_op_wire_1_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            ecc_op_wire_2_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            ecc_op_wire_3_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            ecc_op_wire_4_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            calldata_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            calldata_read_counts_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            lookup_inverses_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            sorted_accum_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            w_4_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            z_perm_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            z_lookup_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            for (size_t i = 0; i < log_n; ++i) {
                sumcheck_univariates.push_back(
                    deserialize_from_buffer<barretenberg::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>>(
                        proof_data, num_bytes_read));
            }
            sumcheck_evaluations =
                deserialize_from_buffer<std::array<FF, NUM_ALL_ENTITIES>>(proof_data, num_bytes_read);
            for (size_t i = 0; i < log_n; ++i) {
                zm_cq_comms.push_back(deserialize_from_buffer<Commitment>(proof_data, num_bytes_read));
            }
            zm_cq_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            zm_pi_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
        }

        void serialize_full_transcript() override
        {
            size_t old_proof_length = proof_data.size();
            proof_data.clear();
            size_t log_n = numeric::get_msb(circuit_size);
            serialize_to_buffer(circuit_size, proof_data);
            serialize_to_buffer(public_input_size, proof_data);
            serialize_to_buffer(pub_inputs_offset, proof_data);
            for (size_t i = 0; i < public_input_size; ++i) {
                serialize_to_buffer(public_inputs[i], proof_data);
            }
            serialize_to_buffer(w_l_comm, proof_data);
            serialize_to_buffer(w_r_comm, proof_data);
            serialize_to_buffer(w_o_comm, proof_data);
            serialize_to_buffer(ecc_op_wire_1_comm, proof_data);
            serialize_to_buffer(ecc_op_wire_2_comm, proof_data);
            serialize_to_buffer(ecc_op_wire_3_comm, proof_data);
            serialize_to_buffer(ecc_op_wire_4_comm, proof_data);
            serialize_to_buffer(calldata_comm, proof_data);
            serialize_to_buffer(calldata_read_counts_comm, proof_data);
            serialize_to_buffer(lookup_inverses_comm, proof_data);
            serialize_to_buffer(sorted_accum_comm, proof_data);
            serialize_to_buffer(w_4_comm, proof_data);
            serialize_to_buffer(z_perm_comm, proof_data);
            serialize_to_buffer(z_lookup_comm, proof_data);
            for (size_t i = 0; i < log_n; ++i) {
                serialize_to_buffer(sumcheck_univariates[i], proof_data);
            }
            serialize_to_buffer(sumcheck_evaluations, proof_data);
            for (size_t i = 0; i < log_n; ++i) {
                serialize_to_buffer(zm_cq_comms[i], proof_data);
            }
            serialize_to_buffer(zm_cq_comm, proof_data);
            serialize_to_buffer(zm_pi_comm, proof_data);

            ASSERT(proof_data.size() == old_proof_length);
        }
    };
};

} // namespace proof_system::honk::flavor