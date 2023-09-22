#pragma once
#include "../sumcheck/relation_definitions_fwd.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/honk/pcs/ipa/ipa.hpp"
#include "barretenberg/honk/pcs/kzg/kzg.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"
#include "barretenberg/proof_system/relations/ecc_vm/ecc_lookup_relation.hpp"
#include "barretenberg/proof_system/relations/ecc_vm/ecc_msm_relation.hpp"
#include "barretenberg/proof_system/relations/ecc_vm/ecc_point_table_relation.hpp"
#include "barretenberg/proof_system/relations/ecc_vm/ecc_set_relation.hpp"
#include "barretenberg/proof_system/relations/ecc_vm/ecc_transcript_relation.hpp"
#include "barretenberg/proof_system/relations/ecc_vm/ecc_wnaf_relation.hpp"
#include "barretenberg/proof_system/relations/relation_types.hpp"
#include <array>
#include <concepts>
#include <span>
#include <string>
#include <type_traits>
#include <vector>

// NOLINTBEGIN(cppcoreguidelines-avoid-const-or-ref-data-members)

namespace proof_system::honk {
namespace flavor {

template <typename CycleGroup_T, typename Curve_T, typename PCS_T> class ECCVMBase {
  public:
    // forward template params into the ECCVMBase namespace
    using CycleGroup = CycleGroup_T;
    using Curve = Curve_T;
    using G1 = typename Curve::Group;
    using PCS = PCS_T;

    using FF = typename G1::subgroup_field;
    using Polynomial = barretenberg::Polynomial<FF>;
    using PolynomialHandle = std::span<FF>;
    using GroupElement = typename G1::element;
    using Commitment = typename G1::affine_element;
    using CommitmentHandle = typename G1::affine_element;
    using CommitmentKey = pcs::CommitmentKey<Curve>;
    using VerifierCommitmentKey = pcs::VerifierCommitmentKey<Curve>;

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

    using GrandProductRelations = std::tuple<sumcheck::ECCVMSetRelation<FF>>;
    // define the tuple of Relations that comprise the Sumcheck relation
    using Relations = std::tuple<sumcheck::ECCVMTranscriptRelation<FF>,
                                 sumcheck::ECCVMPointTableRelation<FF>,
                                 sumcheck::ECCVMWnafRelation<FF>,
                                 sumcheck::ECCVMMSMRelation<FF>,
                                 sumcheck::ECCVMSetRelation<FF>,
                                 sumcheck::ECCVMLookupRelation<FF>>;

    using LookupRelation = sumcheck::ECCVMLookupRelation<FF>;
    static constexpr size_t MAX_RELATION_LENGTH = get_max_relation_length<Relations>();

    // MAX_RANDOM_RELATION_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta` random
    // polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation length = 3
    static constexpr size_t MAX_RANDOM_RELATION_LENGTH = MAX_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size<Relations>::value;

    // Instantiate the BarycentricData needed to extend each Relation Univariate
    // static_assert(instantiate_barycentric_utils<FF, MAX_RANDOM_RELATION_LENGTH>());

    // define the containers for storing the contributions from each relation in Sumcheck
    using RelationUnivariates = decltype(create_relation_univariates_container<FF, Relations>());
    using RelationValues = decltype(create_relation_values_container<FF, Relations>());

  private:
    /**
     * @brief A base class labelling precomputed entities and (ordered) subsets of interest.
     * @details Used to build the proving key and verification key.
     */
    template <typename DataType, typename HandleType>
    class PrecomputedEntities : public PrecomputedEntities_<DataType, HandleType, NUM_PRECOMPUTED_ENTITIES> {
      public:
        DataType& lagrange_first = std::get<0>(this->_data);
        DataType& lagrange_second = std::get<1>(this->_data);
        DataType& lagrange_last = std::get<2>(this->_data);

        std::vector<HandleType> get_selectors() override { return { lagrange_first, lagrange_second, lagrange_last }; };
        std::vector<HandleType> get_sigma_polynomials() override { return {}; };
        std::vector<HandleType> get_id_polynomials() override { return {}; };
        std::vector<HandleType> get_table_polynomials() { return {}; };
    };

    /**
     * @brief Container for all witness polynomials used/constructed by the prover.
     * @details Shifts are not included here since they do not occupy their own memory.
     */
    template <typename DataType, typename HandleType>
    class WitnessEntities : public WitnessEntities_<DataType, HandleType, NUM_WITNESS_ENTITIES> {
      public:
        // clang-format off
        DataType& transcript_add                  = std::get<0>(this->_data);
        DataType& transcript_mul                  = std::get<1>(this->_data);
        DataType& transcript_eq                   = std::get<2>(this->_data);
        DataType& transcript_collision_check           = std::get<3>(this->_data);
        DataType& transcript_msm_transition       = std::get<4>(this->_data);
        DataType& transcript_pc                     = std::get<5>(this->_data);
        DataType& transcript_msm_count              = std::get<6>(this->_data);
        DataType& transcript_x                      = std::get<7>(this->_data);
        DataType& transcript_y                      = std::get<8>(this->_data);
        DataType& transcript_z1                     = std::get<9>(this->_data);
        DataType& transcript_z2                     = std::get<10>(this->_data);
        DataType& transcript_z1zero                 = std::get<11>(this->_data); 
        DataType& transcript_z2zero                 = std::get<12>(this->_data);
        DataType& transcript_op                     = std::get<13>(this->_data);
        DataType& transcript_accumulator_x          = std::get<14>(this->_data);
        DataType& transcript_accumulator_y          = std::get<15>(this->_data);
        DataType& transcript_msm_x                  = std::get<16>(this->_data);
        DataType& transcript_msm_y                  = std::get<17>(this->_data);
        DataType& precompute_pc                          = std::get<18>(this->_data);
        DataType& precompute_point_transition            = std::get<19>(this->_data);
        DataType& precompute_round                       = std::get<20>(this->_data);
        DataType& precompute_scalar_sum                  = std::get<21>(this->_data);
        DataType& precompute_s1hi                          = std::get<22>(this->_data);
        DataType& precompute_s1lo                          = std::get<23>(this->_data);
        DataType& precompute_s2hi                          = std::get<24>(this->_data);
        DataType& precompute_s2lo                          = std::get<25>(this->_data);
        DataType& precompute_s3hi                          = std::get<26>(this->_data);
        DataType& precompute_s3lo                          = std::get<27>(this->_data);
        DataType& precompute_s4hi                          = std::get<28>(this->_data);
        DataType& precompute_s4lo                          = std::get<29>(this->_data);
        DataType& precompute_skew                        = std::get<30>(this->_data);
        DataType& precompute_dx                          = std::get<31>(this->_data);
        DataType& precompute_dy                          = std::get<32>(this->_data);
        DataType& precompute_tx                          = std::get<33>(this->_data);
        DataType& precompute_ty                          = std::get<34>(this->_data);
        DataType& msm_transition                  = std::get<35>(this->_data);
        DataType& msm_add                         = std::get<36>(this->_data);
        DataType& msm_double                      = std::get<37>(this->_data);
        DataType& msm_skew                        = std::get<38>(this->_data);
        DataType& msm_accumulator_x                 = std::get<39>(this->_data);
        DataType& msm_accumulator_y                 = std::get<40>(this->_data);
        DataType& msm_pc                            = std::get<41>(this->_data);
        DataType& msm_size_of_msm                   = std::get<42>(this->_data);
        DataType& msm_count                         = std::get<43>(this->_data);
        DataType& msm_round                         = std::get<44>(this->_data);
        DataType& msm_add1                        = std::get<45>(this->_data);
        DataType& msm_add2                        = std::get<46>(this->_data);
        DataType& msm_add3                        = std::get<47>(this->_data);
        DataType& msm_add4                        = std::get<48>(this->_data);
        DataType& msm_x1                            = std::get<49>(this->_data);
        DataType& msm_y1                            = std::get<50>(this->_data);
        DataType& msm_x2                            = std::get<51>(this->_data);
        DataType& msm_y2                            = std::get<52>(this->_data);
        DataType& msm_x3                            = std::get<53>(this->_data);
        DataType& msm_y3                            = std::get<54>(this->_data);
        DataType& msm_x4                            = std::get<55>(this->_data);
        DataType& msm_y4                            = std::get<56>(this->_data);
        DataType& msm_collision_x1                  = std::get<57>(this->_data);
        DataType& msm_collision_x2                  = std::get<58>(this->_data);
        DataType& msm_collision_x3                  = std::get<59>(this->_data);
        DataType& msm_collision_x4                  = std::get<60>(this->_data);
        DataType& msm_lambda1                       = std::get<61>(this->_data);
        DataType& msm_lambda2                       = std::get<62>(this->_data);
        DataType& msm_lambda3                       = std::get<63>(this->_data);
        DataType& msm_lambda4                       = std::get<64>(this->_data);
        DataType& msm_slice1                        = std::get<65>(this->_data);
        DataType& msm_slice2                        = std::get<66>(this->_data);
        DataType& msm_slice3                        = std::get<67>(this->_data);
        DataType& msm_slice4                        = std::get<68>(this->_data);
        DataType& transcript_accumulator_empty      = std::get<69>(this->_data);
        DataType& transcript_reset_accumulator    = std::get<70>(this->_data);
        DataType& precompute_select                            = std::get<71>(this->_data);
        DataType& lookup_read_counts_0              = std::get<72>(this->_data);
        DataType& lookup_read_counts_1              = std::get<73>(this->_data);
        DataType& z_perm                            = std::get<74>(this->_data);
        DataType& lookup_inverses                   = std::get<75>(this->_data);

        // clang-format on
        std::vector<HandleType> get_wires() override
        {
            return {
                transcript_add,
                transcript_mul,
                transcript_eq,
                transcript_collision_check,
                transcript_msm_transition,
                transcript_pc,
                transcript_msm_count,
                transcript_x,
                transcript_y,
                transcript_z1,
                transcript_z2,
                transcript_z1zero,
                transcript_z2zero,
                transcript_op,
                transcript_accumulator_x,
                transcript_accumulator_y,
                transcript_msm_x,
                transcript_msm_y,
                precompute_pc,
                precompute_point_transition,
                precompute_round,
                precompute_scalar_sum,
                precompute_s1hi,
                precompute_s1lo,
                precompute_s2hi,
                precompute_s2lo,
                precompute_s3hi,
                precompute_s3lo,
                precompute_s4hi,
                precompute_s4lo,
                precompute_skew,
                precompute_dx,
                precompute_dy,
                precompute_tx,
                precompute_ty,
                msm_transition,
                msm_add,
                msm_double,
                msm_skew,
                msm_accumulator_x,
                msm_accumulator_y,
                msm_pc,
                msm_size_of_msm,
                msm_count,
                msm_round,
                msm_add1,
                msm_add2,
                msm_add3,
                msm_add4,
                msm_x1,
                msm_y1,
                msm_x2,
                msm_y2,
                msm_x3,
                msm_y3,
                msm_x4,
                msm_y4,
                msm_collision_x1,
                msm_collision_x2,
                msm_collision_x3,
                msm_collision_x4,
                msm_lambda1,
                msm_lambda2,
                msm_lambda3,
                msm_lambda4,
                msm_slice1,
                msm_slice2,
                msm_slice3,
                msm_slice4,
                transcript_accumulator_empty,
                transcript_reset_accumulator,
                precompute_select,
                lookup_read_counts_0,
                lookup_read_counts_1,
            };
        };
        // The sorted concatenations of table and witness data needed for plookup.
        std::vector<HandleType> get_sorted_polynomials() { return {}; };
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
        // clang-format off
        DataType& lagrange_first                    = std::get<0>(this->_data);
        DataType& lagrange_second                   = std::get<1>(this->_data);
        DataType& lagrange_last                     = std::get<2>(this->_data);
        DataType& transcript_add                    = std::get<3>(this->_data);
        DataType& transcript_mul                    = std::get<4>(this->_data);
        DataType& transcript_eq                     = std::get<5>(this->_data);
        DataType& transcript_collision_check        = std::get<6>(this->_data);
        DataType& transcript_msm_transition         = std::get<7>(this->_data);
        DataType& transcript_pc                     = std::get<8>(this->_data);
        DataType& transcript_msm_count              = std::get<9>(this->_data);
        DataType& transcript_x                      = std::get<10>(this->_data);
        DataType& transcript_y                      = std::get<11>(this->_data);
        DataType& transcript_z1                     = std::get<12>(this->_data);
        DataType& transcript_z2                     = std::get<13>(this->_data);
        DataType& transcript_z1zero                 = std::get<14>(this->_data); 
        DataType& transcript_z2zero                 = std::get<15>(this->_data);
        DataType& transcript_op                     = std::get<16>(this->_data);
        DataType& transcript_accumulator_x          = std::get<17>(this->_data);
        DataType& transcript_accumulator_y          = std::get<18>(this->_data);
        DataType& transcript_msm_x                  = std::get<19>(this->_data);
        DataType& transcript_msm_y                  = std::get<20>(this->_data);
        DataType& precompute_pc                     = std::get<21>(this->_data);
        DataType& precompute_point_transition       = std::get<22>(this->_data);
        DataType& precompute_round                  = std::get<23>(this->_data);
        DataType& precompute_scalar_sum             = std::get<24>(this->_data);
        DataType& precompute_s1hi                   = std::get<25>(this->_data);
        DataType& precompute_s1lo                   = std::get<26>(this->_data);
        DataType& precompute_s2hi                   = std::get<27>(this->_data);
        DataType& precompute_s2lo                   = std::get<28>(this->_data);
        DataType& precompute_s3hi                   = std::get<29>(this->_data);
        DataType& precompute_s3lo                   = std::get<30>(this->_data);
        DataType& precompute_s4hi                   = std::get<31>(this->_data);
        DataType& precompute_s4lo                   = std::get<32>(this->_data);
        DataType& precompute_skew                   = std::get<33>(this->_data);
        DataType& precompute_dx                     = std::get<34>(this->_data);
        DataType& precompute_dy                     = std::get<35>(this->_data);
        DataType& precompute_tx                     = std::get<36>(this->_data);
        DataType& precompute_ty                     = std::get<37>(this->_data);
        DataType& msm_transition                    = std::get<38>(this->_data);
        DataType& msm_add                           = std::get<39>(this->_data);
        DataType& msm_double                        = std::get<40>(this->_data);
        DataType& msm_skew                          = std::get<41>(this->_data);
        DataType& msm_accumulator_x                 = std::get<42>(this->_data);
        DataType& msm_accumulator_y                 = std::get<43>(this->_data);
        DataType& msm_pc                            = std::get<44>(this->_data);
        DataType& msm_size_of_msm                   = std::get<45>(this->_data);
        DataType& msm_count                         = std::get<46>(this->_data);
        DataType& msm_round                         = std::get<47>(this->_data);
        DataType& msm_add1                          = std::get<48>(this->_data);
        DataType& msm_add2                          = std::get<49>(this->_data);
        DataType& msm_add3                          = std::get<50>(this->_data);
        DataType& msm_add4                          = std::get<51>(this->_data);
        DataType& msm_x1                            = std::get<52>(this->_data);
        DataType& msm_y1                            = std::get<53>(this->_data);
        DataType& msm_x2                            = std::get<54>(this->_data);
        DataType& msm_y2                            = std::get<55>(this->_data);
        DataType& msm_x3                            = std::get<56>(this->_data);
        DataType& msm_y3                            = std::get<57>(this->_data);
        DataType& msm_x4                            = std::get<58>(this->_data);
        DataType& msm_y4                            = std::get<59>(this->_data);
        DataType& msm_collision_x1                  = std::get<60>(this->_data);
        DataType& msm_collision_x2                  = std::get<61>(this->_data);
        DataType& msm_collision_x3                  = std::get<62>(this->_data);
        DataType& msm_collision_x4                  = std::get<63>(this->_data);
        DataType& msm_lambda1                       = std::get<64>(this->_data);
        DataType& msm_lambda2                       = std::get<65>(this->_data);
        DataType& msm_lambda3                       = std::get<66>(this->_data);
        DataType& msm_lambda4                       = std::get<67>(this->_data);
        DataType& msm_slice1                        = std::get<68>(this->_data);
        DataType& msm_slice2                        = std::get<69>(this->_data);
        DataType& msm_slice3                        = std::get<70>(this->_data);
        DataType& msm_slice4                        = std::get<71>(this->_data);
        DataType& transcript_accumulator_empty      = std::get<72>(this->_data);
        DataType& transcript_reset_accumulator      = std::get<73>(this->_data);
        DataType& precompute_select                 = std::get<74>(this->_data);
        DataType& lookup_read_counts_0              = std::get<75>(this->_data);
        DataType& lookup_read_counts_1              = std::get<76>(this->_data);
        DataType& z_perm                            = std::get<77>(this->_data);
        DataType& lookup_inverses                   = std::get<78>(this->_data);
        DataType& transcript_mul_shift              = std::get<79>(this->_data);
        DataType& transcript_msm_count_shift        = std::get<80>(this->_data);
        DataType& transcript_accumulator_x_shift    = std::get<81>(this->_data);
        DataType& transcript_accumulator_y_shift    = std::get<82>(this->_data);
        DataType& precompute_scalar_sum_shift       = std::get<83>(this->_data);
        DataType& precompute_s1hi_shift             = std::get<84>(this->_data);
        DataType& precompute_dx_shift               = std::get<85>(this->_data);
        DataType& precompute_dy_shift               = std::get<86>(this->_data);
        DataType& precompute_tx_shift               = std::get<87>(this->_data);
        DataType& precompute_ty_shift               = std::get<88>(this->_data);
        DataType& msm_transition_shift              = std::get<89>(this->_data);
        DataType& msm_add_shift                     = std::get<90>(this->_data);
        DataType& msm_double_shift                  = std::get<91>(this->_data);
        DataType& msm_skew_shift                    = std::get<92>(this->_data);
        DataType& msm_accumulator_x_shift           = std::get<93>(this->_data);
        DataType& msm_accumulator_y_shift           = std::get<94>(this->_data);
        DataType& msm_count_shift                   = std::get<95>(this->_data);
        DataType& msm_round_shift                   = std::get<96>(this->_data);
        DataType& msm_add1_shift                    = std::get<97>(this->_data);
        DataType& msm_pc_shift                      = std::get<98>(this->_data);
        DataType& precompute_pc_shift               = std::get<99>(this->_data);
        DataType& transcript_pc_shift               = std::get<100>(this->_data);
        DataType& precompute_round_shift            = std::get<101>(this->_data);
        DataType& transcript_accumulator_empty_shift= std::get<102>(this->_data);
        DataType& precompute_select_shift           = std::get<103>(this->_data);
        DataType& z_perm_shift                      = std::get<104>(this->_data);

        template <size_t index>
        [[nodiscard]] const DataType& lookup_read_counts() const
        {
            static_assert(index == 0 || index == 1);
            return std::get<75 + index>(this->_data);
        }
        // clang-format on

        std::vector<HandleType> get_wires() override
        {
            return {
                transcript_add,
                transcript_mul,
                transcript_eq,
                transcript_collision_check,
                transcript_msm_transition,
                transcript_pc,
                transcript_msm_count,
                transcript_x,
                transcript_y,
                transcript_z1,
                transcript_z2,
                transcript_z1zero,
                transcript_z2zero,
                transcript_op,
                transcript_accumulator_x,
                transcript_accumulator_y,
                transcript_msm_x,
                transcript_msm_y,
                precompute_pc,
                precompute_point_transition,
                precompute_round,
                precompute_scalar_sum,
                precompute_s1hi,
                precompute_s1lo,
                precompute_s2hi,
                precompute_s2lo,
                precompute_s3hi,
                precompute_s3lo,
                precompute_s4hi,
                precompute_s4lo,
                precompute_skew,
                precompute_dx,
                precompute_dy,
                precompute_tx,
                precompute_ty,
                msm_transition,
                msm_add,
                msm_double,
                msm_skew,
                msm_accumulator_x,
                msm_accumulator_y,
                msm_pc,
                msm_size_of_msm,
                msm_count,
                msm_round,
                msm_add1,
                msm_add2,
                msm_add3,
                msm_add4,
                msm_x1,
                msm_y1,
                msm_x2,
                msm_y2,
                msm_x3,
                msm_y3,
                msm_x4,
                msm_y4,
                msm_collision_x1,
                msm_collision_x2,
                msm_collision_x3,
                msm_collision_x4,
                msm_lambda1,
                msm_lambda2,
                msm_lambda3,
                msm_lambda4,
                msm_slice1,
                msm_slice2,
                msm_slice3,
                msm_slice4,
                transcript_accumulator_empty,
                transcript_reset_accumulator,
                precompute_select,
                lookup_read_counts_0,
                lookup_read_counts_1,
            };
        };
        // Gemini-specific getters.
        std::vector<HandleType> get_unshifted() override
        {
            return {
                lagrange_first,
                lagrange_second,
                lagrange_last,
                transcript_add,
                transcript_eq,
                transcript_collision_check,
                transcript_msm_transition,
                transcript_x,
                transcript_y,
                transcript_z1,
                transcript_z2,
                transcript_z1zero,
                transcript_z2zero,
                transcript_op,
                transcript_msm_x,
                transcript_msm_y,
                precompute_point_transition,
                precompute_s1hi,
                precompute_s2hi,
                precompute_s2lo,
                precompute_s3hi,
                precompute_s3lo,
                precompute_s4hi,
                precompute_s4lo,
                precompute_skew,
                msm_size_of_msm,
                msm_add2,
                msm_add3,
                msm_add4,
                msm_x1,
                msm_y1,
                msm_x2,
                msm_y2,
                msm_x3,
                msm_y3,
                msm_x4,
                msm_y4,
                msm_collision_x1,
                msm_collision_x2,
                msm_collision_x3,
                msm_collision_x4,
                msm_lambda1,
                msm_lambda2,
                msm_lambda3,
                msm_lambda4,
                msm_slice1,
                msm_slice2,
                msm_slice3,
                msm_slice4,
                transcript_reset_accumulator,
                lookup_read_counts_0,
                lookup_read_counts_1,
                lookup_inverses,
            };
        };

        std::vector<HandleType> get_to_be_shifted() override
        {
            return {
                transcript_mul,
                transcript_msm_count,
                transcript_accumulator_x,
                transcript_accumulator_y,
                precompute_scalar_sum,
                precompute_s1hi,
                precompute_dx,
                precompute_dy,
                precompute_tx,
                precompute_ty,
                msm_transition,
                msm_add,
                msm_double,
                msm_skew,
                msm_accumulator_x,
                msm_accumulator_y,
                msm_count,
                msm_round,
                msm_add1,
                msm_pc,
                precompute_pc,
                transcript_pc,
                precompute_round,
                transcript_accumulator_empty,
                precompute_select,
                z_perm,
            };
        };
        std::vector<HandleType> get_shifted() override
        {
            return {
                transcript_mul_shift,
                transcript_msm_count_shift,
                transcript_accumulator_x_shift,
                transcript_accumulator_y_shift,
                precompute_scalar_sum_shift,
                precompute_s1hi_shift,
                precompute_dx_shift,
                precompute_dy_shift,
                precompute_tx_shift,
                precompute_ty_shift,
                msm_transition_shift,
                msm_add_shift,
                msm_double_shift,
                msm_skew_shift,
                msm_accumulator_x_shift,
                msm_accumulator_y_shift,
                msm_count_shift,
                msm_round_shift,
                msm_add1_shift,
                msm_pc_shift,
                precompute_pc_shift,
                transcript_pc_shift,
                precompute_round_shift,
                transcript_accumulator_empty_shift,
                precompute_select_shift,
                z_perm_shift,
            };
        };

        AllEntities() = default;

        AllEntities(const AllEntities& other)
            : AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES>(other){};

        AllEntities(AllEntities&& other) noexcept
            : AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES>(other){};

        AllEntities& operator=(const AllEntities& other)
        {
            if (this == &other) {
                return *this;
            }
            AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES>::operator=(other);
            return *this;
        }

        AllEntities& operator=(AllEntities&& other) noexcept
        {
            AllEntities_<DataType, HandleType, NUM_ALL_ENTITIES>::operator=(other);
            return *this;
        }

        ~AllEntities() override = default;
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
    using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment, CommitmentHandle>>;

    /**
     * @brief A container for polynomials handles; only stores spans.
     */
    using ProverPolynomials = AllEntities<PolynomialHandle, PolynomialHandle>;

    /**
     * @brief A container for polynomials produced after the first round of sumcheck.
     * @todo TODO(#394) Use polynomial classes for guaranteed memory alignment.
     */
    using FoldedPolynomials = AllEntities<std::vector<FF>, PolynomialHandle>;

    using RawPolynomials = AllEntities<Polynomial, PolynomialHandle>;

    /**
     * @brief A container for polynomials produced after the first round of sumcheck.
     * @todo TODO(#394) Use polynomial classes for guaranteed memory alignment.
     */
    using RowPolynomials = AllEntities<FF, FF>;

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
      private:
        using Base = AllEntities<std::string, std::string>;

      public:
        CommitmentLabels()
            : AllEntities<std::string, std::string>()
        {
            Base::transcript_add = "TRANSCRIPT_ADD";
            Base::transcript_mul = "TRANSCRIPT_MUL";
            Base::transcript_eq = "TRANSCRIPT_EQ";
            Base::transcript_collision_check = "TRANSCRIPT_COLLISION_CHECK";
            Base::transcript_msm_transition = "TRANSCRIPT_MSM_TRANSITION";
            Base::transcript_pc = "TRANSCRIPT_PC";
            Base::transcript_msm_count = "TRANSCRIPT_MSM_COUNT";
            Base::transcript_x = "TRANSCRIPT_X";
            Base::transcript_y = "TRANSCRIPT_Y";
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

    class VerifierCommitments : public AllEntities<Commitment, CommitmentHandle> {
      private:
        using Base = AllEntities<Commitment, CommitmentHandle>;

      public:
        VerifierCommitments(const std::shared_ptr<VerificationKey>& verification_key,
                            const VerifierTranscript<FF>& transcript)
        {
            static_cast<void>(transcript);
            Base::lagrange_first = verification_key->lagrange_first;
            Base::lagrange_second = verification_key->lagrange_second;
            Base::lagrange_last = verification_key->lagrange_last;
        }
    };
};

class ECCVM : public ECCVMBase<grumpkin::g1, curve::BN254, pcs::kzg::KZG<curve::BN254>> {};
class ECCVMGrumpkin : public ECCVMBase<barretenberg::g1, curve::Grumpkin, pcs::ipa::IPA<curve::Grumpkin>> {};

// NOLINTEND(cppcoreguidelines-avoid-const-or-ref-data-members)

} // namespace flavor
namespace sumcheck {

extern template class ECCVMTranscriptRelationBase<barretenberg::fr>;
extern template class ECCVMWnafRelationBase<barretenberg::fr>;
extern template class ECCVMPointTableRelationBase<barretenberg::fr>;
extern template class ECCVMMSMRelationBase<barretenberg::fr>;
extern template class ECCVMSetRelationBase<barretenberg::fr>;
extern template class ECCVMLookupRelationBase<barretenberg::fr>;

DECLARE_SUMCHECK_RELATION_CLASS(ECCVMTranscriptRelationBase, flavor::ECCVM);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMWnafRelationBase, flavor::ECCVM);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMPointTableRelationBase, flavor::ECCVM);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMMSMRelationBase, flavor::ECCVM);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMSetRelationBase, flavor::ECCVM);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMLookupRelationBase, flavor::ECCVM);

DECLARE_SUMCHECK_RELATION_CLASS(ECCVMTranscriptRelationBase, flavor::ECCVMGrumpkin);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMWnafRelationBase, flavor::ECCVMGrumpkin);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMPointTableRelationBase, flavor::ECCVMGrumpkin);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMMSMRelationBase, flavor::ECCVMGrumpkin);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMSetRelationBase, flavor::ECCVMGrumpkin);
DECLARE_SUMCHECK_RELATION_CLASS(ECCVMLookupRelationBase, flavor::ECCVMGrumpkin);

DECLARE_SUMCHECK_PERMUTATION_CLASS(ECCVMSetRelationBase, flavor::ECCVM);
DECLARE_SUMCHECK_PERMUTATION_CLASS(ECCVMSetRelationBase, flavor::ECCVMGrumpkin);
} // namespace sumcheck
} // namespace proof_system::honk
