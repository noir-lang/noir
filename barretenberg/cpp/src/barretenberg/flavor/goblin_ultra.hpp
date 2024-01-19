#pragma once
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/common/ref_vector.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/relations/auxiliary_relation.hpp"
#include "barretenberg/relations/databus_lookup_relation.hpp"
#include "barretenberg/relations/ecc_op_queue_relation.hpp"
#include "barretenberg/relations/elliptic_relation.hpp"
#include "barretenberg/relations/gen_perm_sort_relation.hpp"
#include "barretenberg/relations/lookup_relation.hpp"
#include "barretenberg/relations/permutation_relation.hpp"
#include "barretenberg/relations/poseidon2_external_relation.hpp"
#include "barretenberg/relations/poseidon2_internal_relation.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/transcript/transcript.hpp"
#include "relation_definitions.hpp"

namespace bb::honk::flavor {

class GoblinUltra {
  public:
    using CircuitBuilder = GoblinUltraCircuitBuilder;
    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using GroupElement = Curve::Element;
    using Commitment = Curve::AffineElement;
    using CommitmentHandle = Curve::AffineElement;
    using PCS = pcs::kzg::KZG<Curve>;
    using Polynomial = bb::Polynomial<FF>;
    using PolynomialHandle = std::span<FF>;
    using CommitmentKey = pcs::CommitmentKey<Curve>;
    using VerifierCommitmentKey = pcs::VerifierCommitmentKey<Curve>;

    static constexpr size_t NUM_WIRES = CircuitBuilder::NUM_WIRES;
    // The number of multivariate polynomials on which a sumcheck prover sumcheck operates (including shifts). We often
    // need containers of this size to hold related data, so we choose a name more agnostic than `NUM_POLYNOMIALS`.
    // Note: this number does not include the individual sorted list polynomials.
    static constexpr size_t NUM_ALL_ENTITIES = 55;
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 30;
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES = 14;

    using GrandProductRelations = std::tuple<bb::UltraPermutationRelation<FF>, bb::LookupRelation<FF>>;

    // define the tuple of Relations that comprise the Sumcheck relation
    // Note: made generic for use in GoblinUltraRecursive.
    template <typename FF>
    using Relations_ = std::tuple<bb::UltraArithmeticRelation<FF>,
                                  bb::UltraPermutationRelation<FF>,
                                  bb::LookupRelation<FF>,
                                  bb::GenPermSortRelation<FF>,
                                  bb::EllipticRelation<FF>,
                                  bb::AuxiliaryRelation<FF>,
                                  bb::EccOpQueueRelation<FF>,
                                  bb::DatabusLookupRelation<FF>,
                                  bb::Poseidon2ExternalRelation<FF>,
                                  bb::Poseidon2InternalRelation<FF>>;
    using Relations = Relations_<FF>;

    using LogDerivLookupRelation = bb::DatabusLookupRelation<FF>;

    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_partial_relation_length<Relations>();
    static constexpr size_t MAX_TOTAL_RELATION_LENGTH = compute_max_total_relation_length<Relations>();

    // BATCHED_RELATION_PARTIAL_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta`
    // random polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation
    // length = 3
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = MAX_PARTIAL_RELATION_LENGTH + 1;
    static constexpr size_t BATCHED_RELATION_TOTAL_LENGTH = MAX_TOTAL_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size_v<Relations>;

    // For instances of this flavour, used in folding, we need a unique sumcheck batching challenges for each
    // subrelation. This
    // is because using powers of alpha would increase the degree of Protogalaxy polynomial $G$ (the combiner) to much.
    static constexpr size_t NUM_SUBRELATIONS = compute_number_of_subrelations<Relations>();
    using RelationSeparator = std::array<FF, NUM_SUBRELATIONS - 1>;

    template <size_t NUM_INSTANCES>
    using ProtogalaxyTupleOfTuplesOfUnivariates =
        decltype(create_protogalaxy_tuple_of_tuples_of_univariates<Relations, NUM_INSTANCES>());
    using SumcheckTupleOfTuplesOfUnivariates = decltype(create_sumcheck_tuple_of_tuples_of_univariates<Relations>());
    using TupleOfArraysOfValues = decltype(create_tuple_of_arrays_of_values<Relations>());

    // Whether or not the first row of the execution trace is reserved for 0s to enable shifts
    static constexpr bool has_zero_row = true;
    /**
     * @brief A base class labelling precomputed entities and (ordered) subsets of interest.
     * @details Used to build the proving key and verification key.
     */
    template <typename DataType_> class PrecomputedEntities : public PrecomputedEntitiesBase {
      public:
        using DataType = DataType_;
        DEFINE_FLAVOR_MEMBERS(DataType,
                              q_m,                  // column 0
                              q_c,                  // column 1
                              q_l,                  // column 2
                              q_r,                  // column 3
                              q_o,                  // column 4
                              q_4,                  // column 5
                              q_arith,              // column 6
                              q_sort,               // column 7
                              q_elliptic,           // column 8
                              q_aux,                // column 9
                              q_lookup,             // column 10
                              q_busread,            // column 11
                              q_poseidon2_external, // column 12
                              q_poseidon2_internal, // column 13
                              sigma_1,              // column 14
                              sigma_2,              // column 15
                              sigma_3,              // column 16
                              sigma_4,              // column 17
                              id_1,                 // column 18
                              id_2,                 // column 19
                              id_3,                 // column 20
                              id_4,                 // column 21
                              table_1,              // column 22
                              table_2,              // column 23
                              table_3,              // column 24
                              table_4,              // column 25
                              lagrange_first,       // column 26
                              lagrange_last,        // column 27
                              lagrange_ecc_op,      // column 28 // indicator poly for ecc op gates
                              databus_id            // column 29 // id polynomial, i.e. id_i = i
        )

        static constexpr CircuitType CIRCUIT_TYPE = CircuitBuilder::CIRCUIT_TYPE;

        RefVector<DataType> get_selectors()
        {
            return { q_m,
                     q_c,
                     q_l,
                     q_r,
                     q_o,
                     q_4,
                     q_arith,
                     q_sort,
                     q_elliptic,
                     q_aux,
                     q_lookup,
                     q_busread,
                     q_poseidon2_external,
                     q_poseidon2_internal };
        };
        RefVector<DataType> get_sigma_polynomials() { return { sigma_1, sigma_2, sigma_3, sigma_4 }; };
        RefVector<DataType> get_id_polynomials() { return { id_1, id_2, id_3, id_4 }; };
        RefVector<DataType> get_table_polynomials() { return { table_1, table_2, table_3, table_4 }; };
    };

    // GoblinUltra needs to expose more public classes than most flavors due to GoblinUltraRecursive reuse, but these
    // are internal:
  private:
    // WireEntities for basic witness entities
    template <typename DataType> class WireEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              w_l,  // column 0
                              w_r,  // column 1
                              w_o,  // column 2
                              w_4); // column 3
    };

    // DerivedEntities for derived witness entities
    template <typename DataType> class DerivedEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              sorted_accum,         // column 4
                              z_perm,               // column 5
                              z_lookup,             // column 6
                              ecc_op_wire_1,        // column 7
                              ecc_op_wire_2,        // column 8
                              ecc_op_wire_3,        // column 9
                              ecc_op_wire_4,        // column 10
                              calldata,             // column 11
                              calldata_read_counts, // column 12
                              lookup_inverses);     // column 13
    };

    /**
     * @brief Container for all witness polynomials used/constructed by the prover.
     * @details Shifts are not included here since they do not occupy their own memory.
     * Combines WireEntities + DerivedEntities.
     */
    template <typename DataType>
    class WitnessEntities : public WireEntities<DataType>, public DerivedEntities<DataType> {
      public:
        DEFINE_COMPOUND_GET_ALL(WireEntities<DataType>, DerivedEntities<DataType>)

        RefVector<DataType> get_wires() { return WireEntities<DataType>::get_all(); };
        RefVector<DataType> get_ecc_op_wires()
        {
            return { this->ecc_op_wire_1, this->ecc_op_wire_2, this->ecc_op_wire_3, this->ecc_op_wire_4 };
        }
    };

    template <typename DataType> class ShiftedEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              table_1_shift,      // column 0
                              table_2_shift,      // column 1
                              table_3_shift,      // column 2
                              table_4_shift,      // column 3
                              w_l_shift,          // column 4
                              w_r_shift,          // column 5
                              w_o_shift,          // column 6
                              w_4_shift,          // column 7
                              sorted_accum_shift, // column 8
                              z_perm_shift,       // column 9
                              z_lookup_shift      // column 10
        )
    };

  public:
    /**
     * @brief A base class labelling all entities (for instance, all of the polynomials used by the prover during
     * sumcheck) in this Honk variant along with particular subsets of interest
     * @details Used to build containers for: the prover's polynomial during sumcheck; the sumcheck's folded
     * polynomials; the univariates consturcted during during sumcheck; the evaluations produced by sumcheck.
     *
     * Symbolically we have: AllEntities = PrecomputedEntities + WitnessEntities + "ShiftedEntities". It could be
     * implemented as such, but we have this now.
     */
    template <typename DataType>
    class AllEntities : public PrecomputedEntities<DataType>,
                        public WitnessEntities<DataType>,
                        public ShiftedEntities<DataType> {
      public:
        DEFINE_COMPOUND_GET_ALL(PrecomputedEntities<DataType>, WitnessEntities<DataType>, ShiftedEntities<DataType>)

        RefVector<DataType> get_wires() { return { this->w_l, this->w_r, this->w_o, this->w_4 }; };
        RefVector<DataType> get_ecc_op_wires()
        {
            return { this->ecc_op_wire_1, this->ecc_op_wire_2, this->ecc_op_wire_3, this->ecc_op_wire_4 };
        };
        // Gemini-specific getters.
        RefVector<DataType> get_unshifted()
        {
            return concatenate(PrecomputedEntities<DataType>::get_all(), WitnessEntities<DataType>::get_all());
        };

        RefVector<DataType> get_witness() { return WitnessEntities<DataType>::get_all(); };
        RefVector<DataType> get_to_be_shifted()
        {
            return { this->table_1, this->table_2, this->table_3,      this->table_4, this->w_l,     this->w_r,
                     this->w_o,     this->w_4,     this->sorted_accum, this->z_perm,  this->z_lookup };
        };
        RefVector<DataType> get_precomputed() { return PrecomputedEntities<DataType>::get_all(); }
        RefVector<DataType> get_shifted() { return ShiftedEntities<DataType>::get_all(); };
    };

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

        std::vector<uint32_t> memory_read_records;
        std::vector<uint32_t> memory_write_records;

        size_t num_ecc_op_gates; // needed to determine public input offset

        RefVector<DataType> get_to_be_shifted()
        {
            return { this->table_1, this->table_2, this->table_3,      this->table_4, this->w_l,     this->w_r,
                     this->w_o,     this->w_4,     this->sorted_accum, this->z_perm,  this->z_lookup };
        };
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
    using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment>>;

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
    template <size_t LENGTH> using ProverUnivariates = AllEntities<bb::Univariate<FF, LENGTH>>;

    /**
     * @brief A container for univariates produced during the hot loop in sumcheck.
     */
    using ExtendedEdges = ProverUnivariates<MAX_PARTIAL_RELATION_LENGTH>;

    /**
     * @brief A field element for each entity of the flavor. These entities represent the prover polynomials evaluated
     * at one point.
     */
    class AllValues : public AllEntities<FF> {
      public:
        using Base = AllEntities<FF>;
        using Base::Base;
    };

    /**
     * @brief A container for the prover polynomials handles.
     */
    class ProverPolynomials : public AllEntities<Polynomial> {
      public:
        // Define all operations as default, except move construction/assignment
        ProverPolynomials() = default;
        ProverPolynomials& operator=(const ProverPolynomials&) = delete;
        ProverPolynomials(const ProverPolynomials& o) = delete;
        ProverPolynomials(ProverPolynomials&& o) noexcept = default;
        ProverPolynomials& operator=(ProverPolynomials&& o) noexcept = default;
        ~ProverPolynomials() = default;
        [[nodiscard]] size_t get_polynomial_size() const { return q_c.size(); }
        [[nodiscard]] AllValues get_row(size_t row_idx) const
        {
            AllValues result;
            for (auto [result_field, polynomial] : zip_view(result.get_all(), this->get_all())) {
                result_field = polynomial[row_idx];
            }
            return result;
        }
    };

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
            q_poseidon2_external = "__Q_POSEIDON2_EXTERNAL";
            q_poseidon2_internal = "__Q_POSEIDON2_INTERNAL";
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

    /**
     * Note: Made generic for use in GoblinUltraRecursive.
     **/
    template <typename Commitment, typename VerificationKey>
    class VerifierCommitments_ : public AllEntities<Commitment> {
      public:
        VerifierCommitments_(const std::shared_ptr<VerificationKey>& verification_key)
        {
            this->q_m = verification_key->q_m;
            this->q_l = verification_key->q_l;
            this->q_r = verification_key->q_r;
            this->q_o = verification_key->q_o;
            this->q_4 = verification_key->q_4;
            this->q_c = verification_key->q_c;
            this->q_arith = verification_key->q_arith;
            this->q_sort = verification_key->q_sort;
            this->q_elliptic = verification_key->q_elliptic;
            this->q_aux = verification_key->q_aux;
            this->q_lookup = verification_key->q_lookup;
            this->q_busread = verification_key->q_busread;
            this->q_poseidon2_external = verification_key->q_poseidon2_external;
            this->q_poseidon2_internal = verification_key->q_poseidon2_internal;
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
            this->lagrange_ecc_op = verification_key->lagrange_ecc_op;
            this->databus_id = verification_key->databus_id;
        }

        VerifierCommitments_(const std::shared_ptr<VerificationKey>& verification_key,
                             const WitnessCommitments& witness_commitments)
        {
            this->q_m = verification_key->q_m;
            this->q_l = verification_key->q_l;
            this->q_r = verification_key->q_r;
            this->q_o = verification_key->q_o;
            this->q_4 = verification_key->q_4;
            this->q_c = verification_key->q_c;
            this->q_arith = verification_key->q_arith;
            this->q_sort = verification_key->q_sort;
            this->q_elliptic = verification_key->q_elliptic;
            this->q_aux = verification_key->q_aux;
            this->q_lookup = verification_key->q_lookup;
            this->q_busread = verification_key->q_busread;
            this->q_poseidon2_external = verification_key->q_poseidon2_external;
            this->q_poseidon2_internal = verification_key->q_poseidon2_internal;
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
            this->lagrange_ecc_op = verification_key->lagrange_ecc_op;
            this->databus_id = verification_key->databus_id;

            this->w_l = witness_commitments.w_l;
            this->w_r = witness_commitments.w_r;
            this->w_o = witness_commitments.w_o;
            this->sorted_accum = witness_commitments.sorted_accum;
            this->w_4 = witness_commitments.w_4;
            this->z_perm = witness_commitments.z_perm;
            this->z_lookup = witness_commitments.z_lookup;
            this->ecc_op_wire_1 = witness_commitments.ecc_op_wire_1;
            this->ecc_op_wire_2 = witness_commitments.ecc_op_wire_2;
            this->ecc_op_wire_3 = witness_commitments.ecc_op_wire_3;
            this->calldata = witness_commitments.calldata;
            this->calldata = witness_commitments.calldata_read_counts;
            this->lookup_inverses = witness_commitments.lookup_inverses;
        }
    };
    // Specialize for GoblinUltra (general case used in GoblinUltraRecursive).
    using VerifierCommitments = VerifierCommitments_<Commitment, VerificationKey>;

    /**
     * @brief Derived class that defines proof structure for GoblinUltra proofs, as well as supporting functions.
     * Note: Made generic for use in GoblinUltraRecursive.
     */
    template <typename Commitment> class Transcript_ : public BaseTranscript {
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
        std::vector<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>> sumcheck_univariates;
        std::array<FF, NUM_ALL_ENTITIES> sumcheck_evaluations;
        std::vector<Commitment> zm_cq_comms;
        Commitment zm_cq_comm;
        Commitment zm_pi_comm;

        Transcript_() = default;

        Transcript_(const std::vector<uint8_t>& proof)
            : BaseTranscript(proof)
        {}

        void deserialize_full_transcript()
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
                    deserialize_from_buffer<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>>(proof_data,
                                                                                                 num_bytes_read));
            }
            sumcheck_evaluations =
                deserialize_from_buffer<std::array<FF, NUM_ALL_ENTITIES>>(proof_data, num_bytes_read);
            for (size_t i = 0; i < log_n; ++i) {
                zm_cq_comms.push_back(deserialize_from_buffer<Commitment>(proof_data, num_bytes_read));
            }
            zm_cq_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            zm_pi_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
        }

        void serialize_full_transcript()
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
    // Specialize for GoblinUltra (general case used in GoblinUltraRecursive).
    using Transcript = Transcript_<Commitment>;
};

} // namespace bb::honk::flavor
