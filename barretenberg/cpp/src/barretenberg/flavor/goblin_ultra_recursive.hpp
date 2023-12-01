#pragma once
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/polynomials/barycentric.hpp"
#include "barretenberg/polynomials/univariate.hpp"

#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/relations/auxiliary_relation.hpp"
#include "barretenberg/relations/ecc_op_queue_relation.hpp"
#include "barretenberg/relations/elliptic_relation.hpp"
#include "barretenberg/relations/gen_perm_sort_relation.hpp"
#include "barretenberg/relations/lookup_relation.hpp"
#include "barretenberg/relations/permutation_relation.hpp"
#include "barretenberg/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include "barretenberg/transcript/transcript.hpp"
#include <array>
#include <concepts>
#include <span>
#include <string>
#include <type_traits>
#include <vector>

#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"

namespace proof_system::honk::flavor {

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
    using Commitment = typename Curve::Element;
    using CommitmentHandle = typename Curve::Element;
    using FF = typename Curve::ScalarField;

    // Note(luke): Eventually this may not be needed at all
    using VerifierCommitmentKey = pcs::VerifierCommitmentKey<Curve>;

    static constexpr size_t NUM_WIRES = flavor::GoblinUltra::NUM_WIRES;
    // The number of multivariate polynomials on which a sumcheck prover sumcheck operates (including shifts). We often
    // need containers of this size to hold related data, so we choose a name more agnostic than `NUM_POLYNOMIALS`.
    // Note: this number does not include the individual sorted list polynomials.
    static constexpr size_t NUM_ALL_ENTITIES = 53;
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 28;
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES = 14;

    // define the tuple of Relations that comprise the Sumcheck relation
    using Relations = std::tuple<proof_system::UltraArithmeticRelation<FF>,
                                 proof_system::UltraPermutationRelation<FF>,
                                 proof_system::LookupRelation<FF>,
                                 proof_system::GenPermSortRelation<FF>,
                                 proof_system::EllipticRelation<FF>,
                                 proof_system::AuxiliaryRelation<FF>,
                                 proof_system::EccOpQueueRelation<FF>,
                                 proof_system::DatabusLookupRelation<FF>>;

    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_partial_relation_length<Relations>();

    // BATCHED_RELATION_PARTIAL_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta`
    // random polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation
    // length = 3
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = MAX_PARTIAL_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size<Relations>::value;

    // define the container for storing the univariate contribution from each relation in Sumcheck
    using SumcheckTupleOfTuplesOfUnivariates = decltype(create_sumcheck_tuple_of_tuples_of_univariates<Relations>());
    using TupleOfArraysOfValues = decltype(create_tuple_of_arrays_of_values<Relations>());

  private:
    template <typename DataType>
    /**
     * @brief A base class labelling precomputed entities and (ordered) subsets of interest.
     * @details Used to build the proving key and verification key.
     */
    class PrecomputedEntities : public PrecomputedEntitiesBase {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              q_m,             // column 0
                              q_c,             // column 1
                              q_l,             // column 2
                              q_r,             // column 3
                              q_o,             // column 4
                              q_4,             // column 5
                              q_arith,         // column 6
                              q_sort,          // column 7
                              q_elliptic,      // column 8
                              q_aux,           // column 9
                              q_lookup,        // column 10
                              q_busread,       // column 11
                              sigma_1,         // column 12
                              sigma_2,         // column 13
                              sigma_3,         // column 14
                              sigma_4,         // column 15
                              id_1,            // column 16
                              id_2,            // column 17
                              id_3,            // column 18
                              id_4,            // column 19
                              table_1,         // column 20
                              table_2,         // column 21
                              table_3,         // column 22
                              table_4,         // column 23
                              lagrange_first,  // column 24
                              lagrange_last,   // column 25
                              lagrange_ecc_op, // column 26 // indicator poly for ecc op gates
                              databus_id       // column 27 // id polynomial, i.e. id_i = i
        )

        static constexpr CircuitType CIRCUIT_TYPE = CircuitBuilder::CIRCUIT_TYPE;

        RefVector<DataType> get_selectors()
        {
            return { q_m, q_c, q_l, q_r, q_o, q_4, q_arith, q_sort, q_elliptic, q_aux, q_lookup, q_busread };
        };
        RefVector<DataType> get_sigma_polynomials() { return { sigma_1, sigma_2, sigma_3, sigma_4 }; };
        RefVector<DataType> get_id_polynomials() { return { id_1, id_2, id_3, id_4 }; };

        RefVector<DataType> get_table_polynomials() { return { table_1, table_2, table_3, table_4 }; };
    };

    /**
     * @brief Container for all witness polynomials used/constructed by the prover.
     * @details Shifts are not included here since they do not occupy their own memory.
     */
    template <typename DataType> class WitnessEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              w_l,                  // column 0
                              w_r,                  // column 1
                              w_o,                  // column 2
                              w_4,                  // column 3
                              sorted_accum,         // column 4
                              z_perm,               // column 5
                              z_lookup,             // column 6
                              ecc_op_wire_1,        // column 7
                              ecc_op_wire_2,        // column 8
                              ecc_op_wire_3,        // column 9
                              ecc_op_wire_4,        // column 10
                              calldata,             // column 11
                              calldata_read_counts, // column 12
                              lookup_inverses       // column 13
        )

        RefVector<DataType> get_wires() { return { w_l, w_r, w_o, w_4 }; };
        RefVector<DataType> get_ecc_op_wires()
        {
            return { ecc_op_wire_1, ecc_op_wire_2, ecc_op_wire_3, ecc_op_wire_4 };
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
                              q_c,                  // column 0
                              q_l,                  // column 1
                              q_r,                  // column 2
                              q_o,                  // column 3
                              q_4,                  // column 4
                              q_m,                  // column 5
                              q_arith,              // column 6
                              q_sort,               // column 7
                              q_elliptic,           // column 8
                              q_aux,                // column 9
                              q_lookup,             // column 10
                              q_busread,            // column 11
                              sigma_1,              // column 12
                              sigma_2,              // column 13
                              sigma_3,              // column 14
                              sigma_4,              // column 15
                              id_1,                 // column 16
                              id_2,                 // column 17
                              id_3,                 // column 18
                              id_4,                 // column 19
                              table_1,              // column 20
                              table_2,              // column 21
                              table_3,              // column 22
                              table_4,              // column 23
                              lagrange_first,       // column 24
                              lagrange_last,        // column 25
                              lagrange_ecc_op,      // column 26
                              databus_id,           // column 27
                              w_l,                  // column 28
                              w_r,                  // column 29
                              w_o,                  // column 30
                              w_4,                  // column 31
                              sorted_accum,         // column 32
                              z_perm,               // column 33
                              z_lookup,             // column 34
                              ecc_op_wire_1,        // column 35
                              ecc_op_wire_2,        // column 36
                              ecc_op_wire_3,        // column 37
                              ecc_op_wire_4,        // column 38
                              calldata,             // column 39
                              calldata_read_counts, // column 40
                              lookup_inverses,      // column 41
                              table_1_shift,        // column 42
                              table_2_shift,        // column 43
                              table_3_shift,        // column 44
                              table_4_shift,        // column 45
                              w_l_shift,            // column 46
                              w_r_shift,            // column 47
                              w_o_shift,            // column 48
                              w_4_shift,            // column 49
                              sorted_accum_shift,   // column 50
                              z_perm_shift,         // column 51
                              z_lookup_shift);      // column 52

        RefVector<DataType> get_wires() { return { w_l, w_r, w_o, w_4 }; };
        RefVector<DataType> get_ecc_op_wires()
        {
            return { ecc_op_wire_1, ecc_op_wire_2, ecc_op_wire_3, ecc_op_wire_4 };
        };
        // Gemini-specific getters.
        RefVector<DataType> get_unshifted()
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
        RefVector<DataType> get_to_be_shifted()
        {
            return { table_1, table_2, table_3, table_4, w_l, w_r, w_o, w_4, sorted_accum, z_perm, z_lookup };
        };
        RefVector<DataType> get_shifted()
        {
            return { table_1_shift, table_2_shift, table_3_shift,      table_4_shift, w_l_shift,     w_r_shift,
                     w_o_shift,     w_4_shift,     sorted_accum_shift, z_perm_shift,  z_lookup_shift };
        };
    };

  public:
    /**
     * @brief The verification key is responsible for storing the the commitments to the precomputed (non-witnessk)
     * polynomials used by the verifier.
     *
     * @note Note the discrepancy with what sort of data is stored here vs in the proving key. We may want to resolve
     * that, and split out separate PrecomputedPolynomials/Commitments data for clarity but also for portability of our
     * circuits.
     */
    class VerificationKey : public VerificationKey_<PrecomputedEntities<Commitment>> {
      public:
        /**
         * @brief Construct a new Verification Key with stdlib types from a provided native verification
         * key
         *
         * @param builder
         * @param native_key Native verification key from which to extract the precomputed commitments
         */
        VerificationKey(CircuitBuilder* builder, auto native_key)
            : VerificationKey_<PrecomputedEntities<Commitment>>(native_key->circuit_size, native_key->num_public_inputs)
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

    /**
     * @brief A field element for each entity of the flavor. These entities represent the prover polynomials evaluated
     * at one point.
     */
    class AllValues : public AllEntities<FF> {
      public:
        using Base = AllEntities<FF>;
        using Base::Base;
        AllValues(std::array<FF, NUM_ALL_ENTITIES> _data_in) { this->_data = _data_in; }
    };

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
            this->w_l = "W_L";
            this->w_r = "W_R";
            this->w_o = "W_O";
            this->w_4 = "W_4";
            this->z_perm = "Z_PERM";
            this->z_lookup = "Z_LOOKUP";
            this->sorted_accum = "SORTED_ACCUM";
            this->ecc_op_wire_1 = "ECC_OP_WIRE_1";
            this->ecc_op_wire_2 = "ECC_OP_WIRE_2";
            this->ecc_op_wire_3 = "ECC_OP_WIRE_3";
            this->ecc_op_wire_4 = "ECC_OP_WIRE_4";
            this->calldata = "CALLDATA";
            this->calldata_read_counts = "CALLDATA_READ_COUNTS";
            this->lookup_inverses = "LOOKUP_INVERSES";

            // The ones beginning with "__" are only used for debugging
            this->q_c = "__Q_C";
            this->q_l = "__Q_L";
            this->q_r = "__Q_R";
            this->q_o = "__Q_O";
            this->q_4 = "__Q_4";
            this->q_m = "__Q_M";
            this->q_arith = "__Q_ARITH";
            this->q_sort = "__Q_SORT";
            this->q_elliptic = "__Q_ELLIPTIC";
            this->q_aux = "__Q_AUX";
            this->q_lookup = "__Q_LOOKUP";
            this->q_busread = "__Q_BUSREAD";
            this->sigma_1 = "__SIGMA_1";
            this->sigma_2 = "__SIGMA_2";
            this->sigma_3 = "__SIGMA_3";
            this->sigma_4 = "__SIGMA_4";
            this->id_1 = "__ID_1";
            this->id_2 = "__ID_2";
            this->id_3 = "__ID_3";
            this->id_4 = "__ID_4";
            this->table_1 = "__TABLE_1";
            this->table_2 = "__TABLE_2";
            this->table_3 = "__TABLE_3";
            this->table_4 = "__TABLE_4";
            this->lagrange_first = "__LAGRANGE_FIRST";
            this->lagrange_last = "__LAGRANGE_LAST";
            this->lagrange_ecc_op = "__Q_ECC_OP_QUEUE";
        };
    };

    class VerifierCommitments : public AllEntities<Commitment> {
      public:
        VerifierCommitments(std::shared_ptr<VerificationKey> verification_key)
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
    };

    /**
     * @brief Derived class that defines proof structure for GoblinUltraRecursive proofs, as well as supporting
     * functions.
     *
     */
    class Transcript : public BaseTranscript {
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
            : BaseTranscript(proof)
        {}
        /**
         * @brief Takes a FULL GoblinUltraRecursive proof and deserializes it into the public member
         * variables that compose the structure. Must be called in order to access the structure of the
         * proof.
         *
         */
        void deserialize_full_transcript()
        {
            // take current proof and put them into the struct
            size_t num_bytes_read = 0;
            circuit_size = deserialize_from_buffer<uint32_t>(BaseTranscript::proof_data, num_bytes_read);
            size_t log_n = numeric::get_msb(circuit_size);

            public_input_size = deserialize_from_buffer<uint32_t>(BaseTranscript::proof_data, num_bytes_read);
            pub_inputs_offset = deserialize_from_buffer<uint32_t>(BaseTranscript::proof_data, num_bytes_read);
            for (size_t i = 0; i < public_input_size; ++i) {
                public_inputs.push_back(deserialize_from_buffer<FF>(BaseTranscript::proof_data, num_bytes_read));
            }
            w_l_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            w_r_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            w_o_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            ecc_op_wire_1_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            ecc_op_wire_2_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            ecc_op_wire_3_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            ecc_op_wire_4_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            calldata_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            calldata_read_counts_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            lookup_inverses_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            sorted_accum_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            w_4_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            z_perm_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            z_lookup_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            for (size_t i = 0; i < log_n; ++i) {
                sumcheck_univariates.push_back(
                    deserialize_from_buffer<barretenberg::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>>(
                        BaseTranscript::proof_data, num_bytes_read));
            }
            sumcheck_evaluations =
                deserialize_from_buffer<std::array<FF, NUM_ALL_ENTITIES>>(BaseTranscript::proof_data, num_bytes_read);
            for (size_t i = 0; i < log_n; ++i) {
                zm_cq_comms.push_back(deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read));
            }
            zm_cq_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            zm_pi_comm = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
        }

        /**
         * @brief Serializes the structure variables into a FULL GoblinUltraRecursive proof. Should be
         * called only if deserialize_full_transcript() was called and some transcript variable was
         * modified.
         *
         */
        void serialize_full_transcript()
        {
            size_t old_proof_length = BaseTranscript::proof_data.size();
            BaseTranscript::proof_data.clear();
            size_t log_n = numeric::get_msb(circuit_size);
            serialize_to_buffer(circuit_size, BaseTranscript::proof_data);
            serialize_to_buffer(public_input_size, BaseTranscript::proof_data);
            serialize_to_buffer(pub_inputs_offset, BaseTranscript::proof_data);
            for (size_t i = 0; i < public_input_size; ++i) {
                serialize_to_buffer(public_inputs[i], BaseTranscript::proof_data);
            }
            serialize_to_buffer(w_l_comm, BaseTranscript::proof_data);
            serialize_to_buffer(w_r_comm, BaseTranscript::proof_data);
            serialize_to_buffer(w_o_comm, BaseTranscript::proof_data);
            serialize_to_buffer(ecc_op_wire_1_comm, BaseTranscript::proof_data);
            serialize_to_buffer(ecc_op_wire_2_comm, BaseTranscript::proof_data);
            serialize_to_buffer(ecc_op_wire_3_comm, BaseTranscript::proof_data);
            serialize_to_buffer(ecc_op_wire_4_comm, BaseTranscript::proof_data);
            serialize_to_buffer(calldata_comm, BaseTranscript::proof_data);
            serialize_to_buffer(calldata_read_counts_comm, BaseTranscript::proof_data);
            serialize_to_buffer(lookup_inverses_comm, BaseTranscript::proof_data);
            serialize_to_buffer(sorted_accum_comm, BaseTranscript::proof_data);
            serialize_to_buffer(w_4_comm, BaseTranscript::proof_data);
            serialize_to_buffer(z_perm_comm, BaseTranscript::proof_data);
            serialize_to_buffer(z_lookup_comm, BaseTranscript::proof_data);
            for (size_t i = 0; i < log_n; ++i) {
                serialize_to_buffer(sumcheck_univariates[i], BaseTranscript::proof_data);
            }
            serialize_to_buffer(sumcheck_evaluations, BaseTranscript::proof_data);
            for (size_t i = 0; i < log_n; ++i) {
                serialize_to_buffer(zm_cq_comms[i], BaseTranscript::proof_data);
            }
            serialize_to_buffer(zm_cq_comm, BaseTranscript::proof_data);
            serialize_to_buffer(zm_pi_comm, BaseTranscript::proof_data);

            // sanity check to make sure we generate the same length of proof as before.
            ASSERT(BaseTranscript::proof_data.size() == old_proof_length);
        }
    };
};

} // namespace proof_system::honk::flavor
