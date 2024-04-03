#pragma once
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/common/ref_vector.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_delta.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_library.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/relations/auxiliary_relation.hpp"
#include "barretenberg/relations/databus_lookup_relation.hpp"
#include "barretenberg/relations/delta_range_constraint_relation.hpp"
#include "barretenberg/relations/ecc_op_queue_relation.hpp"
#include "barretenberg/relations/elliptic_relation.hpp"
#include "barretenberg/relations/lookup_relation.hpp"
#include "barretenberg/relations/permutation_relation.hpp"
#include "barretenberg/relations/poseidon2_external_relation.hpp"
#include "barretenberg/relations/poseidon2_internal_relation.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb {

class GoblinUltraFlavor {
  public:
    using CircuitBuilder = GoblinUltraCircuitBuilder;
    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using GroupElement = Curve::Element;
    using Commitment = Curve::AffineElement;
    using PCS = KZG<Curve>;
    using Polynomial = bb::Polynomial<FF>;
    using CommitmentKey = bb::CommitmentKey<Curve>;
    using VerifierCommitmentKey = bb::VerifierCommitmentKey<Curve>;

    static constexpr size_t NUM_WIRES = CircuitBuilder::NUM_WIRES;
    // The number of multivariate polynomials on which a sumcheck prover sumcheck operates (including shifts). We often
    // need containers of this size to hold related data, so we choose a name more agnostic than `NUM_POLYNOMIALS`.
    // Note: this number does not include the individual sorted list polynomials.
    static constexpr size_t NUM_ALL_ENTITIES = 58;
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 30;
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES = 17;
    // Total number of folded polynomials, which is just all polynomials except the shifts
    static constexpr size_t NUM_FOLDED_ENTITIES = NUM_PRECOMPUTED_ENTITIES + NUM_WITNESS_ENTITIES;

    using GrandProductRelations = std::tuple<bb::UltraPermutationRelation<FF>, bb::LookupRelation<FF>>;

    // define the tuple of Relations that comprise the Sumcheck relation
    // Note: made generic for use in GoblinUltraRecursive.
    template <typename FF>
    using Relations_ = std::tuple<bb::UltraArithmeticRelation<FF>,
                                  bb::UltraPermutationRelation<FF>,
                                  bb::LookupRelation<FF>,
                                  bb::DeltaRangeConstraintRelation<FF>,
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
                              q_delta_range,        // column 7
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

        auto get_selectors()
        {
            return RefArray{ q_m,
                             q_c,
                             q_l,
                             q_r,
                             q_o,
                             q_4,
                             q_arith,
                             q_delta_range,
                             q_elliptic,
                             q_aux,
                             q_lookup,
                             q_busread,
                             q_poseidon2_external,
                             q_poseidon2_internal };
        };
        auto get_sigma_polynomials() { return RefArray{ sigma_1, sigma_2, sigma_3, sigma_4 }; };
        auto get_id_polynomials() { return RefArray{ id_1, id_2, id_3, id_4 }; };
        auto get_table_polynomials() { return RefArray{ table_1, table_2, table_3, table_4 }; };
    };

    // GoblinUltra needs to expose more public classes than most flavors due to GoblinUltraRecursive reuse, but these
    // are internal:
  public:
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
                              sorted_accum,            // column 4
                              z_perm,                  // column 5
                              z_lookup,                // column 6
                              ecc_op_wire_1,           // column 7
                              ecc_op_wire_2,           // column 8
                              ecc_op_wire_3,           // column 9
                              ecc_op_wire_4,           // column 10
                              calldata,                // column 11
                              calldata_read_counts,    // column 12
                              calldata_inverses,       // column 13
                              return_data,             // column 14
                              return_data_read_counts, // column 15
                              return_data_inverses);   // column 16
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

        auto get_wires() { return WireEntities<DataType>::get_all(); };
        auto get_ecc_op_wires()
        {
            return RefArray{ this->ecc_op_wire_1, this->ecc_op_wire_2, this->ecc_op_wire_3, this->ecc_op_wire_4 };
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

        auto get_wires() { return RefArray{ this->w_l, this->w_r, this->w_o, this->w_4 }; };
        auto get_ecc_op_wires()
        {
            return RefArray{ this->ecc_op_wire_1, this->ecc_op_wire_2, this->ecc_op_wire_3, this->ecc_op_wire_4 };
        };
        // Gemini-specific getters.
        auto get_unshifted()
        {
            return concatenate(PrecomputedEntities<DataType>::get_all(), WitnessEntities<DataType>::get_all());
        };

        auto get_witness() { return WitnessEntities<DataType>::get_all(); };
        auto get_to_be_shifted()
        {
            return RefArray{ this->table_1, this->table_2, this->table_3,      this->table_4, this->w_l,     this->w_r,
                             this->w_o,     this->w_4,     this->sorted_accum, this->z_perm,  this->z_lookup };
        };
        auto get_precomputed() { return PrecomputedEntities<DataType>::get_all(); }
        auto get_shifted() { return ShiftedEntities<DataType>::get_all(); };
    };

    /**
     * @brief The proving key is responsible for storing the polynomials used by the prover.
     * @note TODO(Cody): Maybe multiple inheritance is the right thing here. In that case, nothing should eve inherit
     * from ProvingKey.
     */
    class ProvingKey : public ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>, CommitmentKey> {
      public:
        // Expose constructors on the base class
        using Base = ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>, CommitmentKey>;
        using Base::Base;

        std::vector<uint32_t> memory_read_records;
        std::vector<uint32_t> memory_write_records;
        std::array<Polynomial, 4> sorted_polynomials;

        auto get_to_be_shifted()
        {
            return RefArray{ this->table_1, this->table_2, this->table_3,      this->table_4, this->w_l,     this->w_r,
                             this->w_o,     this->w_4,     this->sorted_accum, this->z_perm,  this->z_lookup };
        };
        // The plookup wires that store plookup read data.
        auto get_table_column_wires() { return RefArray{ w_l, w_r, w_o }; };

        void compute_sorted_accumulator_polynomials(const FF& eta, const FF& eta_two, const FF& eta_three)
        {
            // Compute sorted witness-table accumulator
            compute_sorted_list_accumulator(eta, eta_two, eta_three);

            // Finalize fourth wire polynomial by adding lookup memory records
            add_plookup_memory_records_to_wire_4(eta, eta_two, eta_three);
        }

        /**
         * @brief Construct sorted list accumulator polynomial 's'.
         *
         * @details Compute s = s_1 + η*s_2 + η²*s_3 + η³*s_4 (via Horner) where s_i are the
         * sorted concatenated witness/table polynomials
         *
         * @param key proving key
         * @param sorted_list_polynomials sorted concatenated witness/table polynomials
         * @param eta random challenge
         * @return Polynomial
         */
        void compute_sorted_list_accumulator(const FF& eta, const FF& eta_two, const FF& eta_three)
        {

            auto sorted_list_accumulator = Polynomial{ this->circuit_size };

            // Construct s via Horner, i.e. s = s_1 + η(s_2 + η(s_3 + η*s_4))
            for (size_t i = 0; i < this->circuit_size; ++i) {
                FF T0 = sorted_polynomials[3][i] * eta_three;
                T0 += sorted_polynomials[2][i] * eta_two;
                T0 += sorted_polynomials[1][i] * eta;
                T0 += sorted_polynomials[0][i];
                sorted_list_accumulator[i] = T0;
            }
            sorted_accum = sorted_list_accumulator.share();
        }

        /**
         * @brief Add plookup memory records to the fourth wire polynomial
         *
         * @details This operation must be performed after the first three wires have been committed to, hence the
         * dependence on the `eta` challenge.
         *
         * @tparam Flavor
         * @param eta challenge produced after commitment to first three wire polynomials
         */
        void add_plookup_memory_records_to_wire_4(const FF& eta, const FF& eta_two, const FF& eta_three)
        {
            // The plookup memory record values are computed at the indicated indices as
            // w4 = w3 * eta^3 + w2 * eta^2 + w1 * eta + read_write_flag;
            // (See plookup_auxiliary_widget.hpp for details)
            auto wires = get_wires();

            // Compute read record values
            for (const auto& gate_idx : memory_read_records) {
                wires[3][gate_idx] += wires[2][gate_idx] * eta_three;
                wires[3][gate_idx] += wires[1][gate_idx] * eta_two;
                wires[3][gate_idx] += wires[0][gate_idx] * eta;
            }

            // Compute write record values
            for (const auto& gate_idx : memory_write_records) {
                wires[3][gate_idx] += wires[2][gate_idx] * eta_three;
                wires[3][gate_idx] += wires[1][gate_idx] * eta_two;
                wires[3][gate_idx] += wires[0][gate_idx] * eta;
                wires[3][gate_idx] += 1;
            }
        }

        /**
         * @brief Compute the inverse polynomial used in the databus log derivative lookup argument
         *
         * @tparam Flavor
         * @param beta
         * @param gamma
         */
        void compute_logderivative_inverse(const RelationParameters<FF>& relation_parameters)
        {
            auto prover_polynomials = ProverPolynomials(*this);

            // Compute inverses for calldata reads
            DatabusLookupRelation<FF>::compute_logderivative_inverse</*bus_idx=*/0>(
                prover_polynomials, relation_parameters, this->circuit_size);
            this->calldata_inverses = prover_polynomials.calldata_inverses;

            // Compute inverses for return data reads
            DatabusLookupRelation<FF>::compute_logderivative_inverse</*bus_idx=*/1>(
                prover_polynomials, relation_parameters, this->circuit_size);
            this->return_data_inverses = prover_polynomials.return_data_inverses;
        }

        /**
         * @brief Computes public_input_delta, lookup_grand_product_delta, the z_perm and z_lookup polynomials
         *
         * @param relation_parameters
         */
        void compute_grand_product_polynomials(RelationParameters<FF>& relation_parameters)
        {
            auto public_input_delta = compute_public_input_delta<GoblinUltraFlavor>(this->public_inputs,
                                                                                    relation_parameters.beta,
                                                                                    relation_parameters.gamma,
                                                                                    this->circuit_size,
                                                                                    this->pub_inputs_offset);
            relation_parameters.public_input_delta = public_input_delta;
            auto lookup_grand_product_delta = compute_lookup_grand_product_delta(
                relation_parameters.beta, relation_parameters.gamma, this->circuit_size);
            relation_parameters.lookup_grand_product_delta = lookup_grand_product_delta;

            // Compute permutation and lookup grand product polynomials
            auto prover_polynomials = ProverPolynomials(*this);
            compute_grand_products<GoblinUltraFlavor>(*this, prover_polynomials, relation_parameters);
            this->z_perm = prover_polynomials.z_perm;
            this->z_lookup = prover_polynomials.z_lookup;
        }
    };

    /**
     * @brief The verification key is responsible for storing the the commitments to the precomputed (non-witnessk)
     * polynomials used by the verifier.
     *
     * @note Note the discrepancy with what sort of data is stored here vs in the proving key. We may want to resolve
     * that, and split out separate PrecomputedPolynomials/Commitments data for clarity but also for portability of our
     * circuits.
     * @todo TODO(https://github.com/AztecProtocol/barretenberg/issues/876)
     */
    // using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment>, VerifierCommitmentKey>;
    class VerificationKey : public VerificationKey_<PrecomputedEntities<Commitment>, VerifierCommitmentKey> {
      public:
        VerificationKey() = default;
        VerificationKey(const size_t circuit_size, const size_t num_public_inputs)
            : VerificationKey_(circuit_size, num_public_inputs)
        {}

        VerificationKey(ProvingKey& proving_key)
        {
            this->pcs_verification_key = std::make_shared<VerifierCommitmentKey>();
            this->circuit_size = proving_key.circuit_size;
            this->log_circuit_size = numeric::get_msb(this->circuit_size);
            this->num_public_inputs = proving_key.num_public_inputs;
            this->pub_inputs_offset = proving_key.pub_inputs_offset;

            for (auto [polynomial, commitment] : zip_view(proving_key.get_precomputed_polynomials(), this->get_all())) {
                commitment = proving_key.commitment_key->commit(polynomial);
            }
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
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/925), proving_key could be const ref
        ProverPolynomials(ProvingKey& proving_key)
        {
            for (auto [prover_poly, key_poly] : zip_view(this->get_unshifted(), proving_key.get_all())) {
                ASSERT(flavor_get_label(*this, prover_poly) == flavor_get_label(proving_key, key_poly));
                prover_poly = key_poly.share();
            }
            for (auto [prover_poly, key_poly] : zip_view(this->get_shifted(), proving_key.get_to_be_shifted())) {
                ASSERT(flavor_get_label(*this, prover_poly) == (flavor_get_label(proving_key, key_poly) + "_shift"));
                prover_poly = key_poly.shifted();
            }
        }
        // Define all operations as default, except copy construction/assignment
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
            calldata_inverses = "CALLDATA_INVERSES";
            return_data = "RETURN_DATA";
            return_data_read_counts = "RETURN_DATA_READ_COUNTS";
            return_data_inverses = "RETURN_DATA_INVERSES";

            q_c = "Q_C";
            q_l = "Q_L";
            q_r = "Q_R";
            q_o = "Q_O";
            q_4 = "Q_4";
            q_m = "Q_M";
            q_arith = "Q_ARITH";
            q_delta_range = "Q_SORT";
            q_elliptic = "Q_ELLIPTIC";
            q_aux = "Q_AUX";
            q_lookup = "Q_LOOKUP";
            q_busread = "Q_BUSREAD";
            q_poseidon2_external = "Q_POSEIDON2_EXTERNAL";
            q_poseidon2_internal = "Q_POSEIDON2_INTERNAL";
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
            lagrange_ecc_op = "Q_ECC_OP_QUEUE";
        };
    };

    /**
     * Note: Made generic for use in GoblinUltraRecursive.
     **/
    template <typename Commitment, typename VerificationKey>
    class VerifierCommitments_ : public AllEntities<Commitment> {
      public:
        VerifierCommitments_(const std::shared_ptr<VerificationKey>& verification_key,
                             const std::optional<WitnessEntities<Commitment>>& witness_commitments = std::nullopt)
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

            if (witness_commitments.has_value()) {
                auto commitments = witness_commitments.value();
                this->w_l = commitments.w_l;
                this->w_r = commitments.w_r;
                this->w_o = commitments.w_o;
                this->w_4 = commitments.w_4;
                this->sorted_accum = commitments.sorted_accum;
                this->z_perm = commitments.z_perm;
                this->z_lookup = commitments.z_lookup;
                this->ecc_op_wire_1 = commitments.ecc_op_wire_1;
                this->ecc_op_wire_2 = commitments.ecc_op_wire_2;
                this->ecc_op_wire_3 = commitments.ecc_op_wire_3;
                this->ecc_op_wire_4 = commitments.ecc_op_wire_4;
                this->calldata = commitments.calldata;
                this->calldata_read_counts = commitments.calldata_read_counts;
                this->calldata_inverses = commitments.calldata_inverses;
                this->return_data = commitments.return_data;
                this->return_data_read_counts = commitments.return_data_read_counts;
                this->return_data_inverses = commitments.return_data_inverses;
            }
        }
    };
    // Specialize for GoblinUltra (general case used in GoblinUltraRecursive).
    using VerifierCommitments = VerifierCommitments_<Commitment, VerificationKey>;

    /**
     * @brief Derived class that defines proof structure for GoblinUltra proofs, as well as supporting functions.
     * Note: Made generic for use in GoblinUltraRecursive.
     * TODO(https://github.com/AztecProtocol/barretenberg/issues/877): Remove this Commitment template parameter
     */
    template <typename Commitment> class Transcript_ : public NativeTranscript {
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
        Commitment calldata_inverses_comm;
        Commitment return_data_comm;
        Commitment return_data_read_counts_comm;
        Commitment return_data_inverses_comm;
        Commitment sorted_accum_comm;
        Commitment w_4_comm;
        Commitment z_perm_comm;
        Commitment z_lookup_comm;
        std::vector<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>> sumcheck_univariates;
        std::array<FF, NUM_ALL_ENTITIES> sumcheck_evaluations;
        std::vector<Commitment> zm_cq_comms;
        Commitment zm_cq_comm;
        Commitment kzg_w_comm;

        Transcript_() = default;

        Transcript_(const HonkProof& proof)
            : NativeTranscript(proof)
        {}

        static std::shared_ptr<Transcript_> prover_init_empty()
        {
            auto transcript = std::make_shared<Transcript_>();
            constexpr uint32_t init{ 42 }; // arbitrary
            transcript->send_to_verifier("Init", init);
            return transcript;
        };

        static std::shared_ptr<Transcript_> verifier_init_empty(const std::shared_ptr<Transcript_>& transcript)
        {
            auto verifier_transcript = std::make_shared<Transcript_>(transcript->proof_data);
            [[maybe_unused]] auto _ = verifier_transcript->template receive_from_prover<uint32_t>("Init");
            return verifier_transcript;
        };

        void deserialize_full_transcript()
        {
            // take current proof and put them into the struct
            size_t num_frs_read = 0;
            circuit_size = deserialize_from_buffer<uint32_t>(proof_data, num_frs_read);
            size_t log_n = numeric::get_msb(circuit_size);

            public_input_size = deserialize_from_buffer<uint32_t>(proof_data, num_frs_read);
            pub_inputs_offset = deserialize_from_buffer<uint32_t>(proof_data, num_frs_read);
            for (size_t i = 0; i < public_input_size; ++i) {
                public_inputs.push_back(deserialize_from_buffer<FF>(proof_data, num_frs_read));
            }
            w_l_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            w_r_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            w_o_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            ecc_op_wire_1_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            ecc_op_wire_2_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            ecc_op_wire_3_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            ecc_op_wire_4_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            calldata_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            calldata_read_counts_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            calldata_inverses_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            return_data_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            return_data_read_counts_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            return_data_inverses_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            sorted_accum_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            w_4_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            z_perm_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            z_lookup_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            for (size_t i = 0; i < log_n; ++i) {
                sumcheck_univariates.push_back(
                    deserialize_from_buffer<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>>(proof_data,
                                                                                                 num_frs_read));
            }
            sumcheck_evaluations = deserialize_from_buffer<std::array<FF, NUM_ALL_ENTITIES>>(proof_data, num_frs_read);
            for (size_t i = 0; i < log_n; ++i) {
                zm_cq_comms.push_back(deserialize_from_buffer<Commitment>(proof_data, num_frs_read));
            }
            zm_cq_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
            kzg_w_comm = deserialize_from_buffer<Commitment>(proof_data, num_frs_read);
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
            serialize_to_buffer(calldata_inverses_comm, proof_data);
            serialize_to_buffer(return_data_comm, proof_data);
            serialize_to_buffer(return_data_read_counts_comm, proof_data);
            serialize_to_buffer(return_data_inverses_comm, proof_data);
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
            serialize_to_buffer(kzg_w_comm, proof_data);

            ASSERT(proof_data.size() == old_proof_length);
        }
    };
    // Specialize for GoblinUltra (general case used in GoblinUltraRecursive).
    using Transcript = Transcript_<Commitment>;
};

} // namespace bb
