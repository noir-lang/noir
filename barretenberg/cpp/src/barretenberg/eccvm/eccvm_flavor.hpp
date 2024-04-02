#pragma once
#include "barretenberg/commitment_schemes/ipa/ipa.hpp"
#include "barretenberg/common/std_array.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/eccvm/eccvm_circuit_builder.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/relations/ecc_vm/ecc_lookup_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_msm_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_point_table_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_set_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_transcript_relation.hpp"
#include "barretenberg/relations/ecc_vm/ecc_wnaf_relation.hpp"
#include "barretenberg/relations/relation_parameters.hpp"

// NOLINTBEGIN(cppcoreguidelines-avoid-const-or-ref-data-members)

namespace bb {

class ECCVMFlavor {
  public:
    using CircuitBuilder = ECCVMCircuitBuilder;
    using CycleGroup = bb::g1;
    using Curve = curve::Grumpkin;
    using G1 = typename Curve::Group;
    using PCS = IPA<Curve>;
    using FF = typename G1::subgroup_field;
    using Polynomial = bb::Polynomial<FF>;
    using GroupElement = typename G1::element;
    using Commitment = typename G1::affine_element;
    using CommitmentKey = bb::CommitmentKey<Curve>;
    using VerifierCommitmentKey = bb::VerifierCommitmentKey<Curve>;
    using RelationSeparator = FF;

    static constexpr size_t NUM_WIRES = 74;

    // The number of multivariate polynomials on which a sumcheck prover sumcheck operates (including shifts). We often
    // need containers of this size to hold related data, so we choose a name more agnostic than `NUM_POLYNOMIALS`.
    // Note: this number does not include the individual sorted list polynomials.
    static constexpr size_t NUM_ALL_ENTITIES = 105;
    // The number of polynomials precomputed to describe a circuit and to aid a prover in constructing a satisfying
    // assignment of witnesses. We again choose a neutral name.
    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 3;
    // The total number of witness entities not including shifts.
    static constexpr size_t NUM_WITNESS_ENTITIES = 76;

    using GrandProductRelations = std::tuple<ECCVMSetRelation<FF>>;
    // define the tuple of Relations that comprise the Sumcheck relation
    using Relations = std::tuple<ECCVMTranscriptRelation<FF>,
                                 ECCVMPointTableRelation<FF>,
                                 ECCVMWnafRelation<FF>,
                                 ECCVMMSMRelation<FF>,
                                 ECCVMSetRelation<FF>,
                                 ECCVMLookupRelation<FF>>;

    using LookupRelation = ECCVMLookupRelation<FF>;
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
                              lagrange_first,  // column 0
                              lagrange_second, // column 1
                              lagrange_last);  // column 2

        DataType get_selectors() { return get_all(); };
        auto get_sigma_polynomials() { return RefArray<DataType, 0>{}; };
        auto get_id_polynomials() { return RefArray<DataType, 0>{}; };
        auto get_table_polynomials() { return RefArray<DataType, 0>{}; };
    };

    /**
     * @brief Container for all derived witness polynomials used/constructed by the prover.
     * @details Shifts are not included here since they do not occupy their own memory.
     */
    template <typename DataType> struct DerivedWitnessEntities {
        DEFINE_FLAVOR_MEMBERS(DataType,
                              z_perm,           // column 0
                              lookup_inverses); // column 1
    };

    /**
     * @brief Container for all witness polynomials used/constructed by the prover.
     * @details Shifts are not included here since they do not occupy their own memory.
     */
    template <typename DataType> class WireEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              transcript_add,               // column 0
                              transcript_mul,               // column 1
                              transcript_eq,                // column 2
                              transcript_collision_check,   // column 3
                              transcript_msm_transition,    // column 4
                              transcript_pc,                // column 5
                              transcript_msm_count,         // column 6
                              transcript_Px,                // column 7
                              transcript_Py,                // column 8
                              transcript_z1,                // column 9
                              transcript_z2,                // column 10
                              transcript_z1zero,            // column 11
                              transcript_z2zero,            // column 12
                              transcript_op,                // column 13
                              transcript_accumulator_x,     // column 14
                              transcript_accumulator_y,     // column 15
                              transcript_msm_x,             // column 16
                              transcript_msm_y,             // column 17
                              precompute_pc,                // column 18
                              precompute_point_transition,  // column 19
                              precompute_round,             // column 20
                              precompute_scalar_sum,        // column 21
                              precompute_s1hi,              // column 22
                              precompute_s1lo,              // column 23
                              precompute_s2hi,              // column 24
                              precompute_s2lo,              // column 25
                              precompute_s3hi,              // column 26
                              precompute_s3lo,              // column 27
                              precompute_s4hi,              // column 28
                              precompute_s4lo,              // column 29
                              precompute_skew,              // column 30
                              precompute_dx,                // column 31
                              precompute_dy,                // column 32
                              precompute_tx,                // column 33
                              precompute_ty,                // column 34
                              msm_transition,               // column 35
                              msm_add,                      // column 36
                              msm_double,                   // column 37
                              msm_skew,                     // column 38
                              msm_accumulator_x,            // column 39
                              msm_accumulator_y,            // column 40
                              msm_pc,                       // column 41
                              msm_size_of_msm,              // column 42
                              msm_count,                    // column 43
                              msm_round,                    // column 44
                              msm_add1,                     // column 45
                              msm_add2,                     // column 46
                              msm_add3,                     // column 47
                              msm_add4,                     // column 48
                              msm_x1,                       // column 49
                              msm_y1,                       // column 50
                              msm_x2,                       // column 51
                              msm_y2,                       // column 52
                              msm_x3,                       // column 53
                              msm_y3,                       // column 54
                              msm_x4,                       // column 55
                              msm_y4,                       // column 56
                              msm_collision_x1,             // column 57
                              msm_collision_x2,             // column 58
                              msm_collision_x3,             // column 59
                              msm_collision_x4,             // column 60
                              msm_lambda1,                  // column 61
                              msm_lambda2,                  // column 62
                              msm_lambda3,                  // column 63
                              msm_lambda4,                  // column 64
                              msm_slice1,                   // column 65
                              msm_slice2,                   // column 66
                              msm_slice3,                   // column 67
                              msm_slice4,                   // column 68
                              transcript_accumulator_empty, // column 69
                              transcript_reset_accumulator, // column 70
                              precompute_select,            // column 71
                              lookup_read_counts_0,         // column 72
                              lookup_read_counts_1);        // column 73
    };

    /**
     * @brief Container for all witness polynomials used/constructed by the prover.
     * @details Shifts are not included here since they do not occupy their own memory.
     */
    template <typename DataType>
    class WitnessEntities : public WireEntities<DataType>, public DerivedWitnessEntities<DataType> {
      public:
        DEFINE_COMPOUND_GET_ALL(WireEntities<DataType>, DerivedWitnessEntities<DataType>)
        auto get_wires() { return WireEntities<DataType>::get_all(); };
        // The sorted concatenations of table and witness data needed for plookup.
        auto get_sorted_polynomials() { return RefArray<DataType, 0>{}; };
    };

    /**
     * @brief Represents polynomials shifted by 1 or their evaluations, defined relative to WitnessEntities.
     */
    template <typename DataType> class ShiftedEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              transcript_mul_shift,               // column 0
                              transcript_msm_count_shift,         // column 1
                              transcript_accumulator_x_shift,     // column 2
                              transcript_accumulator_y_shift,     // column 3
                              precompute_scalar_sum_shift,        // column 4
                              precompute_s1hi_shift,              // column 5
                              precompute_dx_shift,                // column 6
                              precompute_dy_shift,                // column 7
                              precompute_tx_shift,                // column 8
                              precompute_ty_shift,                // column 9
                              msm_transition_shift,               // column 10
                              msm_add_shift,                      // column 11
                              msm_double_shift,                   // column 12
                              msm_skew_shift,                     // column 13
                              msm_accumulator_x_shift,            // column 14
                              msm_accumulator_y_shift,            // column 15
                              msm_count_shift,                    // column 16
                              msm_round_shift,                    // column 17
                              msm_add1_shift,                     // column 18
                              msm_pc_shift,                       // column 19
                              precompute_pc_shift,                // column 20
                              transcript_pc_shift,                // column 21
                              precompute_round_shift,             // column 22
                              transcript_accumulator_empty_shift, // column 23
                              precompute_select_shift,            // column 24
                              z_perm_shift);                      // column 25
    };

    template <typename DataType, typename PrecomputedAndWitnessEntitiesSuperset>
    static auto get_to_be_shifted(PrecomputedAndWitnessEntitiesSuperset& entities)
    {
        // NOTE: must match order of ShiftedEntities above!
        return RefArray{ entities.transcript_mul,
                         entities.transcript_msm_count,
                         entities.transcript_accumulator_x,
                         entities.transcript_accumulator_y,
                         entities.precompute_scalar_sum,
                         entities.precompute_s1hi,
                         entities.precompute_dx,
                         entities.precompute_dy,
                         entities.precompute_tx,
                         entities.precompute_ty,
                         entities.msm_transition,
                         entities.msm_add,
                         entities.msm_double,
                         entities.msm_skew,
                         entities.msm_accumulator_x,
                         entities.msm_accumulator_y,
                         entities.msm_count,
                         entities.msm_round,
                         entities.msm_add1,
                         entities.msm_pc,
                         entities.precompute_pc,
                         entities.transcript_pc,
                         entities.precompute_round,
                         entities.transcript_accumulator_empty,
                         entities.precompute_select,
                         entities.z_perm };
    }
    /**
     * @brief A base class labelling all entities (for instance, all of the polynomials used by the prover during
     * sumcheck) in this Honk variant along with particular subsets of interest
     * @details Used to build containers for: the prover's polynomial during sumcheck; the sumcheck's folded
     * polynomials; the univariates consturcted during during sumcheck; the evaluations produced by sumcheck.
     *
     * Symbolically we have: AllEntities = PrecomputedEntities + WitnessEntities + ShiftedEntities.
     * TODO(https://github.com/AztecProtocol/barretenberg/issues/788): Move to normal composition once comfortable
     * updating usage sites.
     */
    template <typename DataType>
    class AllEntities : public PrecomputedEntities<DataType>,
                        public WitnessEntities<DataType>,
                        public ShiftedEntities<DataType> {
      public:
        // Initialize members
        AllEntities()
            : PrecomputedEntities<DataType>{}
            , WitnessEntities<DataType>{}
            , ShiftedEntities<DataType>{}
        {}
        // get_wires is inherited

        DEFINE_COMPOUND_GET_ALL(PrecomputedEntities<DataType>, WitnessEntities<DataType>, ShiftedEntities<DataType>)
        // Gemini-specific getters.
        auto get_unshifted()
        {
            return concatenate(PrecomputedEntities<DataType>::get_all(), WitnessEntities<DataType>::get_all());
        };

        auto get_to_be_shifted() { return ECCVMFlavor::get_to_be_shifted<DataType>(*this); }
        auto get_shifted() { return ShiftedEntities<DataType>::get_all(); };
    };

  public:
    /**
     * @brief The proving key is responsible for storing the polynomials used by the prover.
     * @note TODO(Cody): Maybe multiple inheritance is the right thing here. In that case, nothing should eve
     * inherit from ProvingKey.
     */
    class ProvingKey : public ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>, CommitmentKey> {
      public:
        // Expose constructors on the base class
        using Base = ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>, CommitmentKey>;
        using Base::Base;

        auto get_to_be_shifted() { return ECCVMFlavor::get_to_be_shifted<Polynomial>(*this); }
        // The plookup wires that store plookup read data.
        RefArray<Polynomial, 0> get_table_column_wires() { return {}; };
    };

    /**
     * @brief The verification key is responsible for storing the the commitments to the precomputed (non-witnessk)
     * polynomials used by the verifier.
     *
     * @note Note the discrepancy with what sort of data is stored here vs in the proving key. We may want to
     * resolve that, and split out separate PrecomputedPolynomials/Commitments data for clarity but also for
     * portability of our circuits.
     */
    using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment>, VerifierCommitmentKey>;

    /**
     * @brief A container for polynomials produced after the first round of sumcheck.
     * @todo TODO(#394) Use polynomial classes for guaranteed memory alignment.
     */
    using FoldedPolynomials = AllEntities<std::vector<FF>>;

    /**
     * @brief A field element for each entity of the flavor.  These entities represent the prover polynomials
     * evaluated at one point.
     */
    class AllValues : public AllEntities<FF> {
      public:
        using Base = AllEntities<FF>;
        using Base::Base;
    };

    /**
     * @brief A container for polynomials produced after the first round of sumcheck.
     * @todo TODO(#394) Use polynomial classes for guaranteed memory alignment.
     */
    using RowPolynomials = AllEntities<FF>;

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
     * @brief A container for univariates used during sumcheck.
     */
    template <size_t LENGTH> using ProverUnivariates = AllEntities<bb::Univariate<FF, LENGTH>>;

    /**
     * @brief A container for univariates produced during the hot loop in sumcheck.
     */
    using ExtendedEdges = ProverUnivariates<MAX_PARTIAL_RELATION_LENGTH>;

    /**
     * @brief A container for the prover polynomials.
     */
    class ProverPolynomials : public AllEntities<Polynomial> {
      public:
        // Define all operations as default, except copy construction/assignment
        ProverPolynomials() = default;
        ProverPolynomials& operator=(const ProverPolynomials&) = delete;
        ProverPolynomials(const ProverPolynomials& o) = delete;
        ProverPolynomials(ProverPolynomials&& o) noexcept = default;
        ProverPolynomials& operator=(ProverPolynomials&& o) noexcept = default;
        ~ProverPolynomials() = default;
        [[nodiscard]] size_t get_polynomial_size() const { return this->lagrange_first.size(); }
        /**
         * @brief Returns the evaluations of all prover polynomials at one point on the boolean hypercube, which
         * represents one row in the execution trace.
         */
        AllValues get_row(const size_t row_idx)
        {
            AllValues result;
            for (auto [result_field, polynomial] : zip_view(result.get_all(), this->get_all())) {
                result_field = polynomial[row_idx];
            }
            return result;
        }

        /**
         * @brief Compute the ECCVM flavor polynomial data required to generate an ECCVM Proof
         *
         * @details RawPolynomial member polynomials that this fn must populate described below
         *          For full details see `eccvm/eccvm_flavor.hpp`
         *
         *          lagrange_first: lagrange_first[0] = 1, 0 elsewhere
         *          lagrange_second: lagrange_second[1] = 1, 0 elsewhere
         *          lagrange_last: lagrange_last[lagrange_last.size() - 1] = 1, 0 elsewhere
         *          transcript_add/mul/eq/reset_accumulator: boolean selectors that toggle add/mul/eq/reset opcodes
         *          transcript_collision_check: used to ensure any point being added into eccvm accumulator does not
         trigger
         * incomplete addition rules
         *          transcript_msm_transition: is current transcript row the final `mul` opcode of a multiscalar
         multiplication?
         *          transcript_pc: point counter for transcript columns
         *          transcript_msm_count: counts number of muls processed in an ongoing multiscalar multiplication
         *          transcript_Px: input transcript point, x-coordinate
         *          transcript_Py: input transcriot point, y-coordinate
         *          transcript_op: input transcript opcode value
         *          transcript_z1: input transcript scalar multiplier (low component, 128 bits max)
         *          transcript_z2: input transcript scalar multipplier (high component, 128 bits max)
         * N.B. scalar multiplier = transcript_z1 + \lambda * transcript_z2. \lambda = cube root of unity in scalar
         field
         *          transcript_z1zero: if 1, transcript_z1 must equal 0
         *          transcript_z2zero: if 1, transcript_z2 must equal 0
         *          transcript_accumulator_x: x-coordinate of eccvm accumulator register
         *          transcript_accumulator_y: y-coordinate of eccvm accumulator register
         *          transcript_msm_x: x-coordinate of MSM output
         *          transcript_msm_y: y-coordinate of MSM output
         *          transcript_accumulator_empty: if 1, transcript_accumulator = point at infinity
         *          precompute_pc: point counter for Straus precomputation columns
         *          precompute_select: if 1, evaluate Straus precomputation algorithm at current row
         *          precompute_point_transition: 1 if current row operating on a different point to previous row
         *          precompute_round: round counter for Straus precomputation algorithm
         *          precompute_scalar_sum: accumulating sum of Straus scalar slices
         *          precompute_s1hi/lo: 2-bit hi/lo components of a Straus 4-bit scalar slice
         *          precompute_s2hilo/precompute_s3hi/loprecompute_s4hi/lo: same as above but for a total of 4 Straus
         4-bit scalar slices
         *          precompute_skew: Straus WNAF skew parameter for a single scalar multiplier
         *          precompute_tx: x-coordinate of point accumulator used to generate Straus lookup table for an input
         point (from transcript)
         *          precompute_tx: x-coordinate of point accumulator used to generate Straus lookup table for an input
         point (from transcript)
         *          precompute_dx: x-coordinate of D = 2 * input point we are evaluating Straus over
         *          precompute_dy: y-coordinate of D
         *          msm_pc: point counter for Straus MSM columns
         *          msm_transition: 1 if current row evaluates different MSM to previous row
         *          msm_add: 1 if we are adding points in Straus MSM algorithm at current row
         *          msm_double: 1 if we are doubling accumulator in Straus MSM algorithm at current row
         *          msm_skew: 1 if we are adding skew points in Straus MSM algorithm at current row
         *          msm_size_of_msm: size of multiscalar multiplication current row is a part of
         *          msm_round: describes which round of the Straus MSM algorithm the current row represents
         *          msm_count: number of points processed for the round indicated by `msm_round`
         *          msm_x1: x-coordinate of potential point in Straus MSM round
         *          msm_y1: y-coordinate of potential point in Straus MSM round
         *          msm_x2: x-coordinate of potential point in Straus MSM round
         *          msm_y2: y-coordinate of potential point in Straus MSM round
         *          msm_x3: x-coordinate of potential point in Straus MSM round
         *          msm_y3: y-coordinate of potential point in Straus MSM round
         *          msm_x4: x-coordinate of potential point in Straus MSM round
         *          msm_y4: y-coordinate of potential point in Straus MSM round
         *          msm_add1: are we adding msm_x1/msm_y1 into accumulator at current round?
         *          msm_add2: are we adding msm_x2/msm_y2 into accumulator at current round?
         *          msm_add3: are we adding msm_x3/msm_y3 into accumulator at current round?
         *          msm_add4: are we adding msm_x4/msm_y4 into accumulator at current round?
         *          msm_lambda1: temp variable used for ecc point addition algorithm if msm_add1 = 1
         *          msm_lambda2: temp variable used for ecc point addition algorithm if msm_add2 = 1
         *          msm_lambda3: temp variable used for ecc point addition algorithm if msm_add3 = 1
         *          msm_lambda4: temp variable used for ecc point addition algorithm if msm_add4 = 1
         *          msm_collision_x1: used to ensure incomplete ecc addition exceptions not triggered if msm_add1 = 1
         *          msm_collision_x2: used to ensure incomplete ecc addition exceptions not triggered if msm_add2 = 1
         *          msm_collision_x3: used to ensure incomplete ecc addition exceptions not triggered if msm_add3 = 1
         *          msm_collision_x4: used to ensure incomplete ecc addition exceptions not triggered if msm_add4 = 1
         *          lookup_read_counts_0: stores number of times a point has been read from a Straus precomputation
         table (reads come from msm_x/y1, msm_x/y2)
         *          lookup_read_counts_1: stores number of times a point has been read from a Straus precomputation
         table (reads come from msm_x/y3, msm_x/y4)
         * @return ProverPolynomials
         */
        ProverPolynomials(CircuitBuilder& builder)
        {
            const auto msms = builder.get_msms();
            const auto flattened_muls = builder.get_flattened_scalar_muls(msms);

            std::array<std::vector<size_t>, 2> point_table_read_counts;
            const auto transcript_state = ECCVMTranscriptBuilder::compute_transcript_state(
                builder.op_queue->raw_ops, builder.get_number_of_muls());
            const auto precompute_table_state = ECCVMPrecomputedTablesBuilder::compute_precompute_state(flattened_muls);
            const auto msm_state = ECCVMMSMMBuilder::compute_msm_state(
                msms, point_table_read_counts, builder.get_number_of_muls(), builder.op_queue->get_num_msm_rows());

            const size_t msm_size = msm_state.size();
            const size_t transcript_size = transcript_state.size();
            const size_t precompute_table_size = precompute_table_state.size();

            const size_t num_rows = std::max(precompute_table_size, std::max(msm_size, transcript_size));

            const auto num_rows_log2 = static_cast<size_t>(numeric::get_msb64(num_rows));
            size_t num_rows_pow2 = 1UL << (num_rows_log2 + (1UL << num_rows_log2 == num_rows ? 0 : 1));
            for (auto& poly : get_all()) {
                poly = Polynomial(num_rows_pow2);
            }
            lagrange_first[0] = 1;
            lagrange_second[1] = 1;
            lagrange_last[lagrange_last.size() - 1] = 1;

            for (size_t i = 0; i < point_table_read_counts[0].size(); ++i) {
                // Explanation of off-by-one offset
                // When computing the WNAF slice for a point at point counter value `pc` and a round index `round`, the
                // row number that computes the slice can be derived. This row number is then mapped to the index of
                // `lookup_read_counts`. We do this mapping in `ecc_msm_relation`. We are off-by-one because we add an
                // empty row at the start of the WNAF columns that is not accounted for (index of lookup_read_counts
                // maps to the row in our WNAF columns that computes a slice for a given value of pc and round)
                lookup_read_counts_0[i + 1] = point_table_read_counts[0][i];
                lookup_read_counts_1[i + 1] = point_table_read_counts[1][i];
            }
            run_loop_in_parallel(transcript_state.size(), [&](size_t start, size_t end) {
                for (size_t i = start; i < end; i++) {
                    transcript_accumulator_empty[i] = transcript_state[i].accumulator_empty;
                    transcript_add[i] = transcript_state[i].q_add;
                    transcript_mul[i] = transcript_state[i].q_mul;
                    transcript_eq[i] = transcript_state[i].q_eq;
                    transcript_reset_accumulator[i] = transcript_state[i].q_reset_accumulator;
                    transcript_msm_transition[i] = transcript_state[i].msm_transition;
                    transcript_pc[i] = transcript_state[i].pc;
                    transcript_msm_count[i] = transcript_state[i].msm_count;
                    transcript_Px[i] = transcript_state[i].base_x;
                    transcript_Py[i] = transcript_state[i].base_y;
                    transcript_z1[i] = transcript_state[i].z1;
                    transcript_z2[i] = transcript_state[i].z2;
                    transcript_z1zero[i] = transcript_state[i].z1_zero;
                    transcript_z2zero[i] = transcript_state[i].z2_zero;
                    transcript_op[i] = transcript_state[i].opcode;
                    transcript_accumulator_x[i] = transcript_state[i].accumulator_x;
                    transcript_accumulator_y[i] = transcript_state[i].accumulator_y;
                    transcript_msm_x[i] = transcript_state[i].msm_output_x;
                    transcript_msm_y[i] = transcript_state[i].msm_output_y;
                    transcript_collision_check[i] = transcript_state[i].collision_check;
                }
            });

            // TODO(@zac-williamson) if final opcode resets accumulator, all subsequent "is_accumulator_empty" row
            // values must be 1. Ideally we find a way to tweak this so that empty rows that do nothing have column
            // values that are all zero (issue #2217)
            if (transcript_state[transcript_state.size() - 1].accumulator_empty == 1) {
                for (size_t i = transcript_state.size(); i < num_rows_pow2; ++i) {
                    transcript_accumulator_empty[i] = 1;
                }
            }
            run_loop_in_parallel(precompute_table_state.size(), [&](size_t start, size_t end) {
                for (size_t i = start; i < end; i++) {
                    // first row is always an empty row (to accommodate shifted polynomials which must have 0 as 1st
                    // coefficient). All other rows in the precompute_table_state represent active wnaf gates (i.e.
                    // precompute_select = 1)
                    precompute_select[i] = (i != 0) ? 1 : 0;
                    precompute_pc[i] = precompute_table_state[i].pc;
                    precompute_point_transition[i] = static_cast<uint64_t>(precompute_table_state[i].point_transition);
                    precompute_round[i] = precompute_table_state[i].round;
                    precompute_scalar_sum[i] = precompute_table_state[i].scalar_sum;

                    precompute_s1hi[i] = precompute_table_state[i].s1;
                    precompute_s1lo[i] = precompute_table_state[i].s2;
                    precompute_s2hi[i] = precompute_table_state[i].s3;
                    precompute_s2lo[i] = precompute_table_state[i].s4;
                    precompute_s3hi[i] = precompute_table_state[i].s5;
                    precompute_s3lo[i] = precompute_table_state[i].s6;
                    precompute_s4hi[i] = precompute_table_state[i].s7;
                    precompute_s4lo[i] = precompute_table_state[i].s8;
                    // If skew is active (i.e. we need to subtract a base point from the msm result),
                    // write `7` into rows.precompute_skew. `7`, in binary representation, equals `-1` when converted
                    // into WNAF form
                    precompute_skew[i] = precompute_table_state[i].skew ? 7 : 0;

                    precompute_dx[i] = precompute_table_state[i].precompute_double.x;
                    precompute_dy[i] = precompute_table_state[i].precompute_double.y;
                    precompute_tx[i] = precompute_table_state[i].precompute_accumulator.x;
                    precompute_ty[i] = precompute_table_state[i].precompute_accumulator.y;
                }
            });

            run_loop_in_parallel(msm_state.size(), [&](size_t start, size_t end) {
                for (size_t i = start; i < end; i++) {
                    msm_transition[i] = static_cast<int>(msm_state[i].msm_transition);
                    msm_add[i] = static_cast<int>(msm_state[i].q_add);
                    msm_double[i] = static_cast<int>(msm_state[i].q_double);
                    msm_skew[i] = static_cast<int>(msm_state[i].q_skew);
                    msm_accumulator_x[i] = msm_state[i].accumulator_x;
                    msm_accumulator_y[i] = msm_state[i].accumulator_y;
                    msm_pc[i] = msm_state[i].pc;
                    msm_size_of_msm[i] = msm_state[i].msm_size;
                    msm_count[i] = msm_state[i].msm_count;
                    msm_round[i] = msm_state[i].msm_round;
                    msm_add1[i] = static_cast<int>(msm_state[i].add_state[0].add);
                    msm_add2[i] = static_cast<int>(msm_state[i].add_state[1].add);
                    msm_add3[i] = static_cast<int>(msm_state[i].add_state[2].add);
                    msm_add4[i] = static_cast<int>(msm_state[i].add_state[3].add);
                    msm_x1[i] = msm_state[i].add_state[0].point.x;
                    msm_y1[i] = msm_state[i].add_state[0].point.y;
                    msm_x2[i] = msm_state[i].add_state[1].point.x;
                    msm_y2[i] = msm_state[i].add_state[1].point.y;
                    msm_x3[i] = msm_state[i].add_state[2].point.x;
                    msm_y3[i] = msm_state[i].add_state[2].point.y;
                    msm_x4[i] = msm_state[i].add_state[3].point.x;
                    msm_y4[i] = msm_state[i].add_state[3].point.y;
                    msm_collision_x1[i] = msm_state[i].add_state[0].collision_inverse;
                    msm_collision_x2[i] = msm_state[i].add_state[1].collision_inverse;
                    msm_collision_x3[i] = msm_state[i].add_state[2].collision_inverse;
                    msm_collision_x4[i] = msm_state[i].add_state[3].collision_inverse;
                    msm_lambda1[i] = msm_state[i].add_state[0].lambda;
                    msm_lambda2[i] = msm_state[i].add_state[1].lambda;
                    msm_lambda3[i] = msm_state[i].add_state[2].lambda;
                    msm_lambda4[i] = msm_state[i].add_state[3].lambda;
                    msm_slice1[i] = msm_state[i].add_state[0].slice;
                    msm_slice2[i] = msm_state[i].add_state[1].slice;
                    msm_slice3[i] = msm_state[i].add_state[2].slice;
                    msm_slice4[i] = msm_state[i].add_state[3].slice;
                }
            });
            transcript_mul_shift = transcript_mul.shifted();
            transcript_msm_count_shift = transcript_msm_count.shifted();
            transcript_accumulator_x_shift = transcript_accumulator_x.shifted();
            transcript_accumulator_y_shift = transcript_accumulator_y.shifted();
            precompute_scalar_sum_shift = precompute_scalar_sum.shifted();
            precompute_s1hi_shift = precompute_s1hi.shifted();
            precompute_dx_shift = precompute_dx.shifted();
            precompute_dy_shift = precompute_dy.shifted();
            precompute_tx_shift = precompute_tx.shifted();
            precompute_ty_shift = precompute_ty.shifted();
            msm_transition_shift = msm_transition.shifted();
            msm_add_shift = msm_add.shifted();
            msm_double_shift = msm_double.shifted();
            msm_skew_shift = msm_skew.shifted();
            msm_accumulator_x_shift = msm_accumulator_x.shifted();
            msm_accumulator_y_shift = msm_accumulator_y.shifted();
            msm_count_shift = msm_count.shifted();
            msm_round_shift = msm_round.shifted();
            msm_add1_shift = msm_add1.shifted();
            msm_pc_shift = msm_pc.shifted();
            precompute_pc_shift = precompute_pc.shifted();
            transcript_pc_shift = transcript_pc.shifted();
            precompute_round_shift = precompute_round.shifted();
            transcript_accumulator_empty_shift = transcript_accumulator_empty.shifted();
            precompute_select_shift = precompute_select.shifted();
        }
    };

    /**
     * @brief A container for commitment labels.
     * @note It's debatable whether this should inherit from AllEntities. since most entries are not strictly
     * needed. It has, however, been useful during debugging to have these labels available.
     *
     */
    class CommitmentLabels : public AllEntities<std::string> {
      private:
        using Base = AllEntities<std::string>;

      public:
        CommitmentLabels()
            : AllEntities<std::string>()
        {
            Base::transcript_add = "TRANSCRIPT_ADD";
            Base::transcript_mul = "TRANSCRIPT_MUL";
            Base::transcript_eq = "TRANSCRIPT_EQ";
            Base::transcript_collision_check = "TRANSCRIPT_COLLISION_CHECK";
            Base::transcript_msm_transition = "TRANSCRIPT_MSM_TRANSITION";
            Base::transcript_pc = "TRANSCRIPT_PC";
            Base::transcript_msm_count = "TRANSCRIPT_MSM_COUNT";
            Base::transcript_Px = "TRANSCRIPT_PX";
            Base::transcript_Py = "TRANSCRIPT_PY";
            Base::transcript_z1 = "TRANSCRIPT_Z1";
            Base::transcript_z2 = "TRANSCRIPT_Z2";
            Base::transcript_z1zero = "TRANSCRIPT_Z1ZERO";
            Base::transcript_z2zero = "TRANSCRIPT_Z2ZERO";
            Base::transcript_op = "TRANSCRIPT_OP";
            Base::transcript_accumulator_x = "TRANSCRIPT_ACCUMULATOR_X";
            Base::transcript_accumulator_y = "TRANSCRIPT_ACCUMULATOR_Y";
            Base::transcript_msm_x = "TRANSCRIPT_MSM_X";
            Base::transcript_msm_y = "TRANSCRIPT_MSM_Y";
            Base::precompute_pc = "PRECOMPUTE_PC";
            Base::precompute_point_transition = "PRECOMPUTE_POINT_TRANSITION";
            Base::precompute_round = "PRECOMPUTE_ROUND";
            Base::precompute_scalar_sum = "PRECOMPUTE_SCALAR_SUM";
            Base::precompute_s1hi = "PRECOMPUTE_S1HI";
            Base::precompute_s1lo = "PRECOMPUTE_S1LO";
            Base::precompute_s2hi = "PRECOMPUTE_S2HI";
            Base::precompute_s2lo = "PRECOMPUTE_S2LO";
            Base::precompute_s3hi = "PRECOMPUTE_S3HI";
            Base::precompute_s3lo = "PRECOMPUTE_S3LO";
            Base::precompute_s4hi = "PRECOMPUTE_S4HI";
            Base::precompute_s4lo = "PRECOMPUTE_S4LO";
            Base::precompute_skew = "PRECOMPUTE_SKEW";
            Base::precompute_dx = "PRECOMPUTE_DX";
            Base::precompute_dy = "PRECOMPUTE_DY";
            Base::precompute_tx = "PRECOMPUTE_TX";
            Base::precompute_ty = "PRECOMPUTE_TY";
            Base::msm_transition = "MSM_TRANSITION";
            Base::msm_add = "MSM_ADD";
            Base::msm_double = "MSM_DOUBLE";
            Base::msm_skew = "MSM_SKEW";
            Base::msm_accumulator_x = "MSM_ACCUMULATOR_X";
            Base::msm_accumulator_y = "MSM_ACCUMULATOR_Y";
            Base::msm_pc = "MSM_PC";
            Base::msm_size_of_msm = "MSM_SIZE_OF_MSM";
            Base::msm_count = "MSM_COUNT";
            Base::msm_round = "MSM_ROUND";
            Base::msm_add1 = "MSM_ADD1";
            Base::msm_add2 = "MSM_ADD2";
            Base::msm_add3 = "MSM_ADD3";
            Base::msm_add4 = "MSM_ADD4";
            Base::msm_x1 = "MSM_X1";
            Base::msm_y1 = "MSM_Y1";
            Base::msm_x2 = "MSM_X2";
            Base::msm_y2 = "MSM_Y2";
            Base::msm_x3 = "MSM_X3";
            Base::msm_y3 = "MSM_Y3";
            Base::msm_x4 = "MSM_X4";
            Base::msm_y4 = "MSM_Y4";
            Base::msm_collision_x1 = "MSM_COLLISION_X1";
            Base::msm_collision_x2 = "MSM_COLLISION_X2";
            Base::msm_collision_x3 = "MSM_COLLISION_X3";
            Base::msm_collision_x4 = "MSM_COLLISION_X4";
            Base::msm_lambda1 = "MSM_LAMBDA1";
            Base::msm_lambda2 = "MSM_LAMBDA2";
            Base::msm_lambda3 = "MSM_LAMBDA3";
            Base::msm_lambda4 = "MSM_LAMBDA4";
            Base::msm_slice1 = "MSM_SLICE1";
            Base::msm_slice2 = "MSM_SLICE2";
            Base::msm_slice3 = "MSM_SLICE3";
            Base::msm_slice4 = "MSM_SLICE4";
            Base::transcript_accumulator_empty = "TRANSCRIPT_ACCUMULATOR_EMPTY";
            Base::transcript_reset_accumulator = "TRANSCRIPT_RESET_ACCUMULATOR";
            Base::precompute_select = "PRECOMPUTE_SELECT";
            Base::lookup_read_counts_0 = "LOOKUP_READ_COUNTS_0";
            Base::lookup_read_counts_1 = "LOOKUP_READ_COUNTS_1";
            Base::z_perm = "Z_PERM";
            Base::lookup_inverses = "LOOKUP_INVERSES";
            // The ones beginning with "__" are only used for debugging
            Base::lagrange_first = "__LAGRANGE_FIRST";
            Base::lagrange_second = "__LAGRANGE_SECOND";
            Base::lagrange_last = "__LAGRANGE_LAST";
        };
    };

    class VerifierCommitments : public AllEntities<Commitment> {
      public:
        VerifierCommitments(const std::shared_ptr<VerificationKey>& verification_key)
        {
            this->lagrange_first = verification_key->lagrange_first;
            this->lagrange_second = verification_key->lagrange_second;
            this->lagrange_last = verification_key->lagrange_last;
        }
    };

    /**
     * @brief Derived class that defines proof structure for ECCVM proofs, as well as supporting functions.
     *
     */
    class Transcript : public NativeTranscript {
      public:
        uint32_t circuit_size;
        Commitment transcript_add_comm;
        Commitment transcript_mul_comm;
        Commitment transcript_eq_comm;
        Commitment transcript_collision_check_comm;
        Commitment transcript_msm_transition_comm;
        Commitment transcript_pc_comm;
        Commitment transcript_msm_count_comm;
        Commitment transcript_Px_comm;
        Commitment transcript_Py_comm;
        Commitment transcript_z1_comm;
        Commitment transcript_z2_comm;
        Commitment transcript_z1zero_comm;
        Commitment transcript_z2zero_comm;
        Commitment transcript_op_comm;
        Commitment transcript_accumulator_x_comm;
        Commitment transcript_accumulator_y_comm;
        Commitment transcript_msm_x_comm;
        Commitment transcript_msm_y_comm;
        Commitment precompute_pc_comm;
        Commitment precompute_point_transition_comm;
        Commitment precompute_round_comm;
        Commitment precompute_scalar_sum_comm;
        Commitment precompute_s1hi_comm;
        Commitment precompute_s1lo_comm;
        Commitment precompute_s2hi_comm;
        Commitment precompute_s2lo_comm;
        Commitment precompute_s3hi_comm;
        Commitment precompute_s3lo_comm;
        Commitment precompute_s4hi_comm;
        Commitment precompute_s4lo_comm;
        Commitment precompute_skew_comm;
        Commitment precompute_dx_comm;
        Commitment precompute_dy_comm;
        Commitment precompute_tx_comm;
        Commitment precompute_ty_comm;
        Commitment msm_transition_comm;
        Commitment msm_add_comm;
        Commitment msm_double_comm;
        Commitment msm_skew_comm;
        Commitment msm_accumulator_x_comm;
        Commitment msm_accumulator_y_comm;
        Commitment msm_pc_comm;
        Commitment msm_size_of_msm_comm;
        Commitment msm_count_comm;
        Commitment msm_round_comm;
        Commitment msm_add1_comm;
        Commitment msm_add2_comm;
        Commitment msm_add3_comm;
        Commitment msm_add4_comm;
        Commitment msm_x1_comm;
        Commitment msm_y1_comm;
        Commitment msm_x2_comm;
        Commitment msm_y2_comm;
        Commitment msm_x3_comm;
        Commitment msm_y3_comm;
        Commitment msm_x4_comm;
        Commitment msm_y4_comm;
        Commitment msm_collision_x1_comm;
        Commitment msm_collision_x2_comm;
        Commitment msm_collision_x3_comm;
        Commitment msm_collision_x4_comm;
        Commitment msm_lambda1_comm;
        Commitment msm_lambda2_comm;
        Commitment msm_lambda3_comm;
        Commitment msm_lambda4_comm;
        Commitment msm_slice1_comm;
        Commitment msm_slice2_comm;
        Commitment msm_slice3_comm;
        Commitment msm_slice4_comm;
        Commitment transcript_accumulator_empty_comm;
        Commitment transcript_reset_accumulator_comm;
        Commitment precompute_select_comm;
        Commitment lookup_read_counts_0_comm;
        Commitment lookup_read_counts_1_comm;
        Commitment z_perm_comm;
        Commitment lookup_inverses_comm;
        std::vector<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>> sumcheck_univariates;
        std::array<FF, NUM_ALL_ENTITIES> sumcheck_evaluations;
        std::vector<Commitment> zm_cq_comms;
        Commitment zm_cq_comm;
        uint32_t ipa_poly_degree;
        std::vector<Commitment> ipa_l_comms;
        std::vector<Commitment> ipa_r_comms;
        FF ipa_a_0_eval;
        Commitment translation_hack_comm;
        FF translation_eval_op;
        FF translation_eval_px;
        FF translation_eval_py;
        FF translation_eval_z1;
        FF translation_eval_z2;
        FF hack_eval;
        uint32_t translation_ipa_poly_degree;
        std::vector<Commitment> translation_ipa_l_comms;
        std::vector<Commitment> translation_ipa_r_comms;
        FF translation_ipa_a_0_eval;

        Transcript() = default;

        Transcript(const HonkProof& proof)
            : NativeTranscript(proof)
        {}

        void deserialize_full_transcript()
        {
            // take current proof and put them into the struct
            size_t num_frs_read = 0;
            circuit_size = NativeTranscript::template deserialize_from_buffer<uint32_t>(NativeTranscript::proof_data,
                                                                                        num_frs_read);
            size_t log_n = numeric::get_msb(circuit_size);
            transcript_add_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_mul_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_eq_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_collision_check_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_msm_transition_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_pc_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_msm_count_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_Px_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_Py_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_z1_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_z2_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_z1zero_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_z2zero_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_op_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_accumulator_x_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_accumulator_y_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_msm_x_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_msm_y_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_pc_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_point_transition_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_round_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_scalar_sum_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_s1hi_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_s1lo_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_s2hi_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_s2lo_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_s3hi_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_s3lo_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_s4hi_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_s4lo_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_skew_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_dx_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_dy_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_tx_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_ty_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_transition_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_add_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                          num_frs_read);
            msm_double_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_skew_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                           num_frs_read);
            msm_accumulator_x_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_accumulator_y_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_pc_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                         num_frs_read);
            msm_size_of_msm_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_count_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_round_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_add1_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                           num_frs_read);
            msm_add2_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                           num_frs_read);
            msm_add3_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                           num_frs_read);
            msm_add4_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                           num_frs_read);
            msm_x1_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                         num_frs_read);
            msm_y1_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                         num_frs_read);
            msm_x2_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                         num_frs_read);
            msm_y2_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                         num_frs_read);
            msm_x3_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                         num_frs_read);
            msm_y3_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                         num_frs_read);
            msm_x4_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                         num_frs_read);
            msm_y4_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                         num_frs_read);
            msm_collision_x1_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_collision_x2_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_collision_x3_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_collision_x4_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_lambda1_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_lambda2_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_lambda3_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_lambda4_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_slice1_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_slice2_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_slice3_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            msm_slice4_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_accumulator_empty_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            transcript_reset_accumulator_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            precompute_select_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            lookup_read_counts_0_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            lookup_read_counts_1_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            lookup_inverses_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            z_perm_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(NativeTranscript::proof_data,
                                                                                         num_frs_read);
            for (size_t i = 0; i < log_n; ++i) {
                sumcheck_univariates.emplace_back(NativeTranscript::template deserialize_from_buffer<
                                                  bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>>(
                    NativeTranscript::proof_data, num_frs_read));
            }
            sumcheck_evaluations = NativeTranscript::template deserialize_from_buffer<std::array<FF, NUM_ALL_ENTITIES>>(
                NativeTranscript::proof_data, num_frs_read);
            for (size_t i = 0; i < log_n; ++i) {
                zm_cq_comms.push_back(
                    NativeTranscript::template deserialize_from_buffer<Commitment>(proof_data, num_frs_read));
            }
            zm_cq_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(proof_data, num_frs_read);

            ipa_poly_degree = NativeTranscript::template deserialize_from_buffer<uint32_t>(NativeTranscript::proof_data,
                                                                                           num_frs_read);
            auto log_poly_degree = static_cast<size_t>(numeric::get_msb(ipa_poly_degree));
            for (size_t i = 0; i < log_poly_degree; ++i) {
                ipa_l_comms.emplace_back(NativeTranscript::template deserialize_from_buffer<Commitment>(
                    NativeTranscript::proof_data, num_frs_read));
                ipa_r_comms.emplace_back(NativeTranscript::template deserialize_from_buffer<Commitment>(
                    NativeTranscript::proof_data, num_frs_read));
            }
            ipa_a_0_eval =
                NativeTranscript::template deserialize_from_buffer<FF>(NativeTranscript::proof_data, num_frs_read);
            translation_hack_comm = NativeTranscript::template deserialize_from_buffer<Commitment>(
                NativeTranscript::proof_data, num_frs_read);
            translation_eval_op =
                NativeTranscript::template deserialize_from_buffer<FF>(NativeTranscript::proof_data, num_frs_read);
            translation_eval_px =
                NativeTranscript::template deserialize_from_buffer<FF>(NativeTranscript::proof_data, num_frs_read);
            translation_eval_py =
                NativeTranscript::template deserialize_from_buffer<FF>(NativeTranscript::proof_data, num_frs_read);
            translation_eval_z1 =
                NativeTranscript::template deserialize_from_buffer<FF>(NativeTranscript::proof_data, num_frs_read);
            translation_eval_z2 =
                NativeTranscript::template deserialize_from_buffer<FF>(NativeTranscript::proof_data, num_frs_read);
            hack_eval =
                NativeTranscript::template deserialize_from_buffer<FF>(NativeTranscript::proof_data, num_frs_read);

            translation_ipa_poly_degree = NativeTranscript::template deserialize_from_buffer<uint32_t>(
                NativeTranscript::proof_data, num_frs_read);

            for (size_t i = 0; i < log_poly_degree; ++i) {
                translation_ipa_l_comms.emplace_back(NativeTranscript::template deserialize_from_buffer<Commitment>(
                    NativeTranscript::proof_data, num_frs_read));
                translation_ipa_r_comms.emplace_back(NativeTranscript::template deserialize_from_buffer<Commitment>(
                    NativeTranscript::proof_data, num_frs_read));
            }

            translation_ipa_a_0_eval =
                NativeTranscript::template deserialize_from_buffer<FF>(NativeTranscript::proof_data, num_frs_read);
        }

        void serialize_full_transcript()
        {
            size_t old_proof_length = NativeTranscript::proof_data.size();
            NativeTranscript::proof_data.clear();

            NativeTranscript::template serialize_to_buffer(circuit_size, NativeTranscript::proof_data);
            size_t log_n = numeric::get_msb(circuit_size);

            NativeTranscript::template serialize_to_buffer(transcript_add_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_mul_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_eq_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_collision_check_comm,
                                                           NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_msm_transition_comm,
                                                           NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_pc_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_msm_count_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_Px_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_Py_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_z1_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_z2_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_z1zero_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_z2zero_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_op_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_accumulator_x_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_accumulator_y_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_msm_x_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_msm_y_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_pc_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_point_transition_comm,
                                                           NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_round_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_scalar_sum_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_s1hi_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_s1lo_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_s2hi_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_s2lo_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_s3hi_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_s3lo_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_s4hi_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_s4lo_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_skew_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_dx_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_dy_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_tx_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_ty_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_transition_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_add_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_double_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_skew_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_accumulator_x_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_accumulator_y_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_pc_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_size_of_msm_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_count_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_round_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_add1_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_add2_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_add3_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_add4_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_x1_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_y1_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_x2_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_y2_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_x3_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_y3_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_x4_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_y4_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_collision_x1_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_collision_x2_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_collision_x3_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_collision_x4_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_lambda1_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_lambda2_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_lambda3_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_lambda4_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_slice1_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_slice2_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_slice3_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(msm_slice4_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_accumulator_empty_comm,
                                                           NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(transcript_reset_accumulator_comm,
                                                           NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(precompute_select_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(lookup_read_counts_0_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(lookup_read_counts_1_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(lookup_inverses_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(z_perm_comm, NativeTranscript::proof_data);
            for (size_t i = 0; i < log_n; ++i) {
                NativeTranscript::template serialize_to_buffer(sumcheck_univariates[i], NativeTranscript::proof_data);
            }
            NativeTranscript::template serialize_to_buffer(sumcheck_evaluations, NativeTranscript::proof_data);
            for (size_t i = 0; i < log_n; ++i) {
                NativeTranscript::template serialize_to_buffer(zm_cq_comms[i], NativeTranscript::proof_data);
            }
            NativeTranscript::template serialize_to_buffer(zm_cq_comm, NativeTranscript::proof_data);

            NativeTranscript::template serialize_to_buffer(ipa_poly_degree, NativeTranscript::proof_data);

            auto log_poly_degree = static_cast<size_t>(numeric::get_msb(ipa_poly_degree));
            for (size_t i = 0; i < log_poly_degree; ++i) {
                NativeTranscript::template serialize_to_buffer(ipa_l_comms[i], NativeTranscript::proof_data);
                NativeTranscript::template serialize_to_buffer(ipa_r_comms[i], NativeTranscript::proof_data);
            }

            NativeTranscript::template serialize_to_buffer(ipa_a_0_eval, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(translation_hack_comm, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(translation_eval_op, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(translation_eval_px, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(translation_eval_py, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(translation_eval_z1, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(translation_eval_z2, NativeTranscript::proof_data);
            NativeTranscript::template serialize_to_buffer(hack_eval, NativeTranscript::proof_data);

            NativeTranscript::template serialize_to_buffer(translation_ipa_poly_degree, NativeTranscript::proof_data);
            log_poly_degree = static_cast<size_t>(numeric::get_msb(translation_ipa_poly_degree));
            for (size_t i = 0; i < log_poly_degree; ++i) {
                NativeTranscript::template serialize_to_buffer(translation_ipa_l_comms[i],
                                                               NativeTranscript::proof_data);
                NativeTranscript::template serialize_to_buffer(translation_ipa_r_comms[i],
                                                               NativeTranscript::proof_data);
            }

            serialize_to_buffer(translation_ipa_a_0_eval, proof_data);

            ASSERT(NativeTranscript::proof_data.size() == old_proof_length);
        }
    };
};

// NOLINTEND(cppcoreguidelines-avoid-const-or-ref-data-members)

} // namespace bb
