#pragma once
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/polynomials/barycentric.hpp"
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/relations/auxiliary_relation.hpp"
#include "barretenberg/relations/elliptic_relation.hpp"
#include "barretenberg/relations/gen_perm_sort_relation.hpp"
#include "barretenberg/relations/lookup_relation.hpp"
#include "barretenberg/relations/permutation_relation.hpp"
#include "barretenberg/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb::honk::flavor {

class Ultra {
  public:
    using CircuitBuilder = UltraCircuitBuilder;
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
    static constexpr size_t NUM_ALL_ENTITIES = 43;
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 25;
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES = 7;

    using GrandProductRelations = std::tuple<bb::UltraPermutationRelation<FF>, bb::LookupRelation<FF>>;
    // define the tuple of Relations that comprise the Sumcheck relation
    using Relations = std::tuple<bb::UltraArithmeticRelation<FF>,
                                 bb::UltraPermutationRelation<FF>,
                                 bb::LookupRelation<FF>,
                                 bb::GenPermSortRelation<FF>,
                                 bb::EllipticRelation<FF>,
                                 bb::AuxiliaryRelation<FF>>;

    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_partial_relation_length<Relations>();
    static_assert(MAX_PARTIAL_RELATION_LENGTH == 6);
    static constexpr size_t MAX_TOTAL_RELATION_LENGTH = compute_max_total_relation_length<Relations>();
    static_assert(MAX_TOTAL_RELATION_LENGTH == 12);
    static constexpr size_t NUM_SUBRELATIONS = compute_number_of_subrelations<Relations>();
    // For instances of this flavour, used in folding, we need a unique sumcheck batching challenge for each
    // subrelation. This is because using powers of alpha would increase the degree of Protogalaxy polynomial $G$ (the
    // combiner) too much.
    using RelationSeparator = std::array<FF, NUM_SUBRELATIONS - 1>;

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

    static constexpr bool is_decider = true;

  private:
    /**
     * @brief A base class labelling precomputed entities and (ordered) subsets of interest.
     * @details Used to build the proving key and verification key.
     */
    template <typename DataType_> class PrecomputedEntities : public PrecomputedEntitiesBase {
      public:
        using DataType = DataType_;
        DEFINE_FLAVOR_MEMBERS(DataType,
                              q_m,            // column 0
                              q_c,            // column 1
                              q_l,            // column 2
                              q_r,            // column 3
                              q_o,            // column 4
                              q_4,            // column 5
                              q_arith,        // column 6
                              q_sort,         // column 7
                              q_elliptic,     // column 8
                              q_aux,          // column 9
                              q_lookup,       // column 10
                              sigma_1,        // column 11
                              sigma_2,        // column 12
                              sigma_3,        // column 13
                              sigma_4,        // column 14
                              id_1,           // column 15
                              id_2,           // column 16
                              id_3,           // column 17
                              id_4,           // column 18
                              table_1,        // column 19
                              table_2,        // column 20
                              table_3,        // column 21
                              table_4,        // column 22
                              lagrange_first, // column 23
                              lagrange_last)  // column 24

        static constexpr CircuitType CIRCUIT_TYPE = CircuitBuilder::CIRCUIT_TYPE;

        RefVector<DataType> get_selectors()
        {
            return { q_m, q_c, q_l, q_r, q_o, q_4, q_arith, q_sort, q_elliptic, q_aux, q_lookup };
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
                              w_l,          // column 0
                              w_r,          // column 1
                              w_o,          // column 2
                              w_4,          // column 3
                              sorted_accum, // column 4
                              z_perm,       // column 5
                              z_lookup)     // column 6

        RefVector<DataType> get_wires() { return { w_l, w_r, w_o, w_4, sorted_accum, z_perm, z_lookup }; };
    };

    /**
     * @brief Class for ShiftedEntities, containing shifted witness and table polynomials.
     */
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
                              z_lookup_shift)     // column 10

        RefVector<DataType> get_shifted()
        {
            return { table_1_shift, table_2_shift, table_3_shift,      table_4_shift, w_l_shift,     w_r_shift,
                     w_o_shift,     w_4_shift,     sorted_accum_shift, z_perm_shift,  z_lookup_shift };
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
                              q_c,                // column 0
                              q_l,                // column 1
                              q_r,                // column 2
                              q_o,                // column 3
                              q_4,                // column 4
                              q_m,                // column 5
                              q_arith,            // column 6
                              q_sort,             // column 7
                              q_elliptic,         // column 8
                              q_aux,              // column 9
                              q_lookup,           // column 10
                              sigma_1,            // column 11
                              sigma_2,            // column 12
                              sigma_3,            // column 13
                              sigma_4,            // column 14
                              id_1,               // column 15
                              id_2,               // column 16
                              id_3,               // column 17
                              id_4,               // column 18
                              table_1,            // column 19
                              table_2,            // column 20
                              table_3,            // column 21
                              table_4,            // column 22
                              lagrange_first,     // column 23
                              lagrange_last,      // column 24
                              w_l,                // column 25
                              w_r,                // column 26
                              w_o,                // column 27
                              w_4,                // column 28
                              sorted_accum,       // column 29
                              z_perm,             // column 30
                              z_lookup,           // column 31
                              table_1_shift,      // column 32
                              table_2_shift,      // column 33
                              table_3_shift,      // column 34
                              table_4_shift,      // column 35
                              w_l_shift,          // column 36
                              w_r_shift,          // column 37
                              w_o_shift,          // column 38
                              w_4_shift,          // column 39
                              sorted_accum_shift, // column 40
                              z_perm_shift,       // column 41
                              z_lookup_shift)     // column 42

        RefVector<DataType> get_wires() { return { w_l, w_r, w_o, w_4 }; };
        // Gemini-specific getters.
        RefVector<DataType> get_unshifted()
        {
            return { q_m,           q_c,   q_l,      q_r,     q_o,     q_4,          q_arith, q_sort,
                     q_elliptic,    q_aux, q_lookup, sigma_1, sigma_2, sigma_3,      sigma_4, id_1,
                     id_2,          id_3,  id_4,     table_1, table_2, table_3,      table_4, lagrange_first,
                     lagrange_last, w_l,   w_r,      w_o,     w_4,     sorted_accum, z_perm,  z_lookup

            };
        };

        RefVector<DataType> get_precomputed()
        {
            return { q_m,          q_c,   q_l,      q_r,     q_o,     q_4,     q_arith, q_sort,
                     q_elliptic,   q_aux, q_lookup, sigma_1, sigma_2, sigma_3, sigma_4, id_1,
                     id_2,         id_3,  id_4,     table_1, table_2, table_3, table_4, lagrange_first,
                     lagrange_last

            };
        }

        RefVector<DataType> get_witness() { return { w_l, w_r, w_o, w_4, sorted_accum, z_perm, z_lookup }; };
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
     * @brief A field element for each entity of the flavor. These entities represent the prover polynomials
     * evaluated at one point.
     */
    class AllValues : public AllEntities<FF> {
      public:
        using Base = AllEntities<FF>;
        using Base::Base;
    };

    /**
     * @brief A container for polynomials handles.
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
    template <size_t LENGTH> using ProverUnivariates = AllEntities<bb::Univariate<FF, LENGTH>>;

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

            q_c = "Q_C";
            q_l = "Q_L";
            q_r = "Q_R";
            q_o = "Q_O";
            q_4 = "Q_4";
            q_m = "Q_M";
            q_arith = "Q_ARITH";
            q_sort = "Q_SORT";
            q_elliptic = "Q_ELLIPTIC";
            q_aux = "Q_AUX";
            q_lookup = "Q_LOOKUP";
            sigma_1 = "SIGMA_1";
            sigma_2 = "SIGMA_2";
            sigma_3 = "SIGMA_3";
            sigma_4 = "SIGMA_4";
            id_1 = "ID_1";
            id_2 = "ID_2";
            id_3 = "ID_3";
            id_4 = "ID_4";
            table_1 = "TABLE_1";
            table_2 = "TABLE_2";
            table_3 = "TABLE_3";
            table_4 = "TABLE_4";
            lagrange_first = "LAGRANGE_FIRST";
            lagrange_last = "LAGRANGE_LAST";
        };
    };

    /**
     * @brief A container encapsulating all the commitments that the verifier receives (to precomputed polynomials and
     * witness polynomials).
     *
     */
    class VerifierCommitments : public AllEntities<Commitment> {
      public:
        VerifierCommitments(const std::shared_ptr<VerificationKey>& verification_key)
        {
            q_m = verification_key->q_m;
            q_c = verification_key->q_c;
            q_l = verification_key->q_l;
            q_r = verification_key->q_r;
            q_o = verification_key->q_o;
            q_4 = verification_key->q_4;
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
        }

        VerifierCommitments(const std::shared_ptr<VerificationKey>& verification_key,
                            const WitnessCommitments& witness_commitments)
        {
            q_m = verification_key->q_m;
            q_c = verification_key->q_c;
            q_l = verification_key->q_l;
            q_r = verification_key->q_r;
            q_o = verification_key->q_o;
            q_4 = verification_key->q_4;
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

            w_l = witness_commitments.w_l;
            w_r = witness_commitments.w_r;
            w_o = witness_commitments.w_o;
            sorted_accum = witness_commitments.sorted_accum;
            w_4 = witness_commitments.w_4;
            z_perm = witness_commitments.z_perm;
            z_lookup = witness_commitments.z_lookup;
        }
    };

    /**
     * @brief Derived class that defines proof structure for Ultra proofs, as well as supporting functions.
     *
     */
    class Transcript : public BaseTranscript {
      public:
        // Transcript objects defined as public member variables for easy access and modification
        uint32_t circuit_size;
        uint32_t public_input_size;
        uint32_t pub_inputs_offset;
        std::vector<FF> public_inputs;
        Commitment w_l_comm;
        Commitment w_r_comm;
        Commitment w_o_comm;
        Commitment sorted_accum_comm;
        Commitment w_4_comm;
        Commitment z_perm_comm;
        Commitment z_lookup_comm;
        std::vector<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>> sumcheck_univariates;
        std::array<FF, NUM_ALL_ENTITIES> sumcheck_evaluations;
        std::vector<Commitment> zm_cq_comms;
        Commitment zm_cq_comm;
        Commitment zm_pi_comm;

        Transcript() = default;

        // Used by verifier to initialize the transcript
        Transcript(const std::vector<uint8_t>& proof)
            : BaseTranscript(proof)
        {}

        static std::shared_ptr<Transcript> prover_init_empty()
        {
            auto transcript = std::make_shared<Transcript>();
            constexpr uint32_t init{ 42 }; // arbitrary
            transcript->send_to_verifier("Init", init);
            return transcript;
        };

        static std::shared_ptr<Transcript> verifier_init_empty(const std::shared_ptr<Transcript>& transcript)
        {
            auto verifier_transcript = std::make_shared<Transcript>(transcript->proof_data);
            [[maybe_unused]] auto _ = verifier_transcript->template receive_from_prover<uint32_t>("Init");
            return verifier_transcript;
        };

        /**
         * @brief Takes a FULL Ultra proof and deserializes it into the public member variables that compose the
         * structure. Must be called in order to access the structure of the proof.
         *
         */
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
        /**
         * @brief Serializes the structure variables into a FULL Ultra proof. Should be called only if
         * deserialize_full_transcript() was called and some transcript variable was modified.
         *
         */
        void serialize_full_transcript()
        {
            size_t old_proof_length = proof_data.size();
            proof_data.clear(); // clear proof_data so the rest of the function can replace it
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

            // sanity check to make sure we generate the same length of proof as before.
            ASSERT(proof_data.size() == old_proof_length);
        }
    };
};

} // namespace bb::honk::flavor
