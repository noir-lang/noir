

#pragma once
#include "../relation_definitions.hpp"
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/polynomials/barycentric.hpp"
#include "barretenberg/polynomials/univariate.hpp"

#include "barretenberg/relations/generic_permutation/generic_permutation_relation.hpp"

#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/relations/generated/Toy/toy_avm.hpp"
#include "barretenberg/relations/generated/Toy/two_column_perm.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb::honk {
namespace flavor {

class ToyFlavor {
  public:
    using Curve = curve::BN254;
    using G1 = Curve::Group;
    using PCS = pcs::kzg::KZG<Curve>;

    using FF = G1::subgroup_field;
    using Polynomial = bb::Polynomial<FF>;
    using PolynomialHandle = std::span<FF>;
    using GroupElement = G1::element;
    using Commitment = G1::affine_element;
    using CommitmentHandle = G1::affine_element;
    using CommitmentKey = pcs::CommitmentKey<Curve>;
    using VerifierCommitmentKey = pcs::VerifierCommitmentKey<Curve>;
    using RelationSeparator = FF;

    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 1;
    static constexpr size_t NUM_WITNESS_ENTITIES = 16;
    static constexpr size_t NUM_WIRES = NUM_WITNESS_ENTITIES + NUM_PRECOMPUTED_ENTITIES;
    // We have two copies of the witness entities, so we subtract the number of fixed ones (they have no shift), one for
    // the unshifted and one for the shifted
    static constexpr size_t NUM_ALL_ENTITIES = 17;

    using Relations = std::tuple<Toy_vm::toy_avm<FF>, sumcheck::two_column_perm_relation<FF>>;

    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = compute_max_partial_relation_length<Relations>();

    // BATCHED_RELATION_PARTIAL_LENGTH = algebraic degree of sumcheck relation *after* multiplying by the `pow_zeta`
    // random polynomial e.g. For \sum(x) [A(x) * B(x) + C(x)] * PowZeta(X), relation length = 2 and random relation
    // length = 3
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = MAX_PARTIAL_RELATION_LENGTH + 1;
    static constexpr size_t NUM_RELATIONS = std::tuple_size<Relations>::value;

    template <size_t NUM_INSTANCES>
    using ProtogalaxyTupleOfTuplesOfUnivariates =
        decltype(create_protogalaxy_tuple_of_tuples_of_univariates<Relations, NUM_INSTANCES>());
    using SumcheckTupleOfTuplesOfUnivariates = decltype(create_sumcheck_tuple_of_tuples_of_univariates<Relations>());
    using TupleOfArraysOfValues = decltype(create_tuple_of_arrays_of_values<Relations>());

    static constexpr bool has_zero_row = true;

  private:
    template <typename DataType_> class PrecomputedEntities : public PrecomputedEntitiesBase {
      public:
        using DataType = DataType_;

        DEFINE_FLAVOR_MEMBERS(DataType, toy_first)

        RefVector<DataType> get_selectors() { return { toy_first }; };
        RefVector<DataType> get_sigma_polynomials() { return {}; };
        RefVector<DataType> get_id_polynomials() { return {}; };
        RefVector<DataType> get_table_polynomials() { return {}; };
    };

    template <typename DataType> class WitnessEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              toy_q_tuple_set,
                              toy_set_1_column_1,
                              toy_set_1_column_2,
                              toy_set_2_column_1,
                              toy_set_2_column_2,
                              toy_xor_a,
                              toy_xor_b,
                              toy_xor_c,
                              toy_table_xor_a,
                              toy_table_xor_b,
                              toy_table_xor_c,
                              toy_q_xor,
                              toy_q_xor_table,
                              two_column_perm,
                              lookup_xor,
                              lookup_xor_counts)

        RefVector<DataType> get_wires()
        {
            return { toy_q_tuple_set,    toy_set_1_column_1, toy_set_1_column_2, toy_set_2_column_1,
                     toy_set_2_column_2, toy_xor_a,          toy_xor_b,          toy_xor_c,
                     toy_table_xor_a,    toy_table_xor_b,    toy_table_xor_c,    toy_q_xor,
                     toy_q_xor_table,    two_column_perm,    lookup_xor,         lookup_xor_counts };
        };
        RefVector<DataType> get_sorted_polynomials() { return {}; };
    };

    template <typename DataType> class AllEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType,
                              toy_first,
                              toy_q_tuple_set,
                              toy_set_1_column_1,
                              toy_set_1_column_2,
                              toy_set_2_column_1,
                              toy_set_2_column_2,
                              toy_xor_a,
                              toy_xor_b,
                              toy_xor_c,
                              toy_table_xor_a,
                              toy_table_xor_b,
                              toy_table_xor_c,
                              toy_q_xor,
                              toy_q_xor_table,
                              two_column_perm,
                              lookup_xor,
                              lookup_xor_counts)

        RefVector<DataType> get_wires()
        {
            return { toy_first,          toy_q_tuple_set,  toy_set_1_column_1, toy_set_1_column_2, toy_set_2_column_1,
                     toy_set_2_column_2, toy_xor_a,        toy_xor_b,          toy_xor_c,          toy_table_xor_a,
                     toy_table_xor_b,    toy_table_xor_c,  toy_q_xor,          toy_q_xor_table,    two_column_perm,
                     lookup_xor,         lookup_xor_counts };
        };
        RefVector<DataType> get_unshifted()
        {
            return { toy_first,          toy_q_tuple_set,  toy_set_1_column_1, toy_set_1_column_2, toy_set_2_column_1,
                     toy_set_2_column_2, toy_xor_a,        toy_xor_b,          toy_xor_c,          toy_table_xor_a,
                     toy_table_xor_b,    toy_table_xor_c,  toy_q_xor,          toy_q_xor_table,    two_column_perm,
                     lookup_xor,         lookup_xor_counts };
        };
        RefVector<DataType> get_to_be_shifted() { return {}; };
        RefVector<DataType> get_shifted() { return {}; };
    };

  public:
    class ProvingKey : public ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>> {
      public:
        // Expose constructors on the base class
        using Base = ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>>;
        using Base::Base;

        RefVector<DataType> get_to_be_shifted() { return {}; };

        // The plookup wires that store plookup read data.
        std::array<PolynomialHandle, 0> get_table_column_wires() { return {}; };
    };

    using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment>>;

    using FoldedPolynomials = AllEntities<std::vector<FF>>;

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
        [[nodiscard]] size_t get_polynomial_size() const { return toy_q_tuple_set.size(); }
        /**
         * @brief Returns the evaluations of all prover polynomials at one point on the boolean hypercube, which
         * represents one row in the execution trace.
         */
        [[nodiscard]] AllValues get_row(size_t row_idx) const
        {
            AllValues result;
            for (auto [result_field, polynomial] : zip_view(result.get_all(), this->get_all())) {
                result_field = polynomial[row_idx];
            }
            return result;
        }
    };

    using RowPolynomials = AllEntities<FF>;

    class PartiallyEvaluatedMultivariates : public AllEntities<Polynomial> {
      public:
        PartiallyEvaluatedMultivariates() = default;
        PartiallyEvaluatedMultivariates(const size_t circuit_size)
        {
            // Storage is only needed after the first partial evaluation, hence polynomials of size (n / 2)
            for (auto& poly : get_all()) {
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

    class CommitmentLabels : public AllEntities<std::string> {
      private:
        using Base = AllEntities<std::string>;

      public:
        CommitmentLabels()
            : AllEntities<std::string>()
        {
            Base::toy_first = "TOY_FIRST";
            Base::toy_q_tuple_set = "TOY_Q_TUPLE_SET";
            Base::toy_set_1_column_1 = "TOY_SET_1_COLUMN_1";
            Base::toy_set_1_column_2 = "TOY_SET_1_COLUMN_2";
            Base::toy_set_2_column_1 = "TOY_SET_2_COLUMN_1";
            Base::toy_set_2_column_2 = "TOY_SET_2_COLUMN_2";
            Base::toy_xor_a = "TOY_XOR_A";
            Base::toy_xor_b = "TOY_XOR_B";
            Base::toy_xor_c = "TOY_XOR_C";
            Base::toy_table_xor_a = "TOY_TABLE_XOR_A";
            Base::toy_table_xor_b = "TOY_TABLE_XOR_B";
            Base::toy_table_xor_c = "TOY_TABLE_XOR_C";
            Base::toy_q_xor = "TOY_Q_XOR";
            Base::toy_q_xor_table = "TOY_Q_XOR_TABLE";
            Base::two_column_perm = "TWO_COLUMN_PERM";
            Base::lookup_xor = "LOOKUP_XOR";
            Base::lookup_xor_counts = "LOOKUP_XOR_COUNTS";
        };
    };

    class VerifierCommitments : public AllEntities<Commitment> {
      private:
        using Base = AllEntities<Commitment>;

      public:
        VerifierCommitments(const std::shared_ptr<VerificationKey>& verification_key)
        {
            toy_first = verification_key->toy_first;
        }
    };

    class Transcript : public BaseTranscript {
      public:
        uint32_t circuit_size;

        Commitment toy_q_tuple_set;
        Commitment toy_set_1_column_1;
        Commitment toy_set_1_column_2;
        Commitment toy_set_2_column_1;
        Commitment toy_set_2_column_2;
        Commitment toy_xor_a;
        Commitment toy_xor_b;
        Commitment toy_xor_c;
        Commitment toy_table_xor_a;
        Commitment toy_table_xor_b;
        Commitment toy_table_xor_c;
        Commitment toy_q_xor;
        Commitment toy_q_xor_table;
        Commitment two_column_perm;
        Commitment lookup_xor;
        Commitment lookup_xor_counts;

        std::vector<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>> sumcheck_univariates;
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
            size_t num_bytes_read = 0;
            circuit_size = deserialize_from_buffer<uint32_t>(proof_data, num_bytes_read);
            size_t log_n = numeric::get_msb(circuit_size);

            toy_q_tuple_set = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            toy_set_1_column_1 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            toy_set_1_column_2 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            toy_set_2_column_1 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            toy_set_2_column_2 = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            toy_xor_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            toy_xor_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            toy_xor_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            toy_table_xor_a = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            toy_table_xor_b = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            toy_table_xor_c = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            toy_q_xor = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            toy_q_xor_table = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            two_column_perm = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            lookup_xor = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);
            lookup_xor_counts = deserialize_from_buffer<Commitment>(Transcript::proof_data, num_bytes_read);

            for (size_t i = 0; i < log_n; ++i) {
                sumcheck_univariates.emplace_back(
                    deserialize_from_buffer<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>>(Transcript::proof_data,
                                                                                                 num_bytes_read));
            }
            sumcheck_evaluations =
                deserialize_from_buffer<std::array<FF, NUM_ALL_ENTITIES>>(Transcript::proof_data, num_bytes_read);
            for (size_t i = 0; i < log_n; ++i) {
                zm_cq_comms.push_back(deserialize_from_buffer<Commitment>(proof_data, num_bytes_read));
            }
            zm_cq_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            zm_pi_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
        }

        void serialize_full_transcript()
        {
            size_t old_proof_length = proof_data.size();
            Transcript::proof_data.clear();
            size_t log_n = numeric::get_msb(circuit_size);

            serialize_to_buffer(circuit_size, Transcript::proof_data);

            serialize_to_buffer<Commitment>(toy_q_tuple_set, Transcript::proof_data);
            serialize_to_buffer<Commitment>(toy_set_1_column_1, Transcript::proof_data);
            serialize_to_buffer<Commitment>(toy_set_1_column_2, Transcript::proof_data);
            serialize_to_buffer<Commitment>(toy_set_2_column_1, Transcript::proof_data);
            serialize_to_buffer<Commitment>(toy_set_2_column_2, Transcript::proof_data);
            serialize_to_buffer<Commitment>(toy_xor_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(toy_xor_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(toy_xor_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(toy_table_xor_a, Transcript::proof_data);
            serialize_to_buffer<Commitment>(toy_table_xor_b, Transcript::proof_data);
            serialize_to_buffer<Commitment>(toy_table_xor_c, Transcript::proof_data);
            serialize_to_buffer<Commitment>(toy_q_xor, Transcript::proof_data);
            serialize_to_buffer<Commitment>(toy_q_xor_table, Transcript::proof_data);
            serialize_to_buffer<Commitment>(two_column_perm, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_xor, Transcript::proof_data);
            serialize_to_buffer<Commitment>(lookup_xor_counts, Transcript::proof_data);

            for (size_t i = 0; i < log_n; ++i) {
                serialize_to_buffer(sumcheck_univariates[i], Transcript::proof_data);
            }
            serialize_to_buffer(sumcheck_evaluations, Transcript::proof_data);
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

} // namespace flavor
} // namespace bb::honk
