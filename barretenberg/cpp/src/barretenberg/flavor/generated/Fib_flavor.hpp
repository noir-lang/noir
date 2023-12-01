

#pragma once
#include "../relation_definitions_fwd.hpp"
#include "barretenberg/commitment_schemes/kzg/kzg.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/polynomials/barycentric.hpp"
#include "barretenberg/polynomials/univariate.hpp"

#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/flavor/flavor_macros.hpp"
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/relations/generated/Fib.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace proof_system::honk {
namespace flavor {

class FibFlavor {
  public:
    using Curve = curve::BN254;
    using G1 = Curve::Group;
    using PCS = pcs::kzg::KZG<Curve>;

    using FF = G1::subgroup_field;
    using Polynomial = barretenberg::Polynomial<FF>;
    using PolynomialHandle = std::span<FF>;
    using GroupElement = G1::element;
    using Commitment = G1::affine_element;
    using CommitmentHandle = G1::affine_element;
    using CommitmentKey = pcs::CommitmentKey<Curve>;
    using VerifierCommitmentKey = pcs::VerifierCommitmentKey<Curve>;

    static constexpr size_t NUM_PRECOMPUTED_ENTITIES = 2;
    static constexpr size_t NUM_WITNESS_ENTITIES = 2;
    static constexpr size_t NUM_WIRES = NUM_WITNESS_ENTITIES + NUM_PRECOMPUTED_ENTITIES;
    // We have two copies of the witness entities, so we subtract the number of fixed ones (they have no shift), one for
    // the unshifted and one for the shifted
    static constexpr size_t NUM_ALL_ENTITIES = 6;

    using Relations = std::tuple<Fib_vm::Fib<FF>>;

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
        DEFINE_FLAVOR_MEMBERS(DataType, Fibonacci_LAST, Fibonacci_FIRST)

        RefVector<DataType> get_selectors()
        {
            return {
                Fibonacci_LAST,
                Fibonacci_FIRST,
            };
        };

        RefVector<DataType> get_sigma_polynomials() { return {}; };
        RefVector<DataType> get_id_polynomials() { return {}; };
        RefVector<DataType> get_table_polynomials() { return {}; };
    };

    template <typename DataType> class WitnessEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(DataType, Fibonacci_x, Fibonacci_y)

        RefVector<DataType> get_wires()
        {
            return {
                Fibonacci_x,
                Fibonacci_y,

            };
        };

        RefVector<DataType> get_sorted_polynomials() { return {}; };
    };

    template <typename DataType> class AllEntities {
      public:
        DEFINE_FLAVOR_MEMBERS(
            DataType, Fibonacci_LAST, Fibonacci_FIRST, Fibonacci_x, Fibonacci_y, Fibonacci_x_shift, Fibonacci_y_shift)

        RefVector<DataType> get_wires()
        {
            return {
                Fibonacci_LAST, Fibonacci_FIRST, Fibonacci_x, Fibonacci_y, Fibonacci_x_shift, Fibonacci_y_shift,

            };
        };

        RefVector<DataType> get_unshifted()
        {
            return {
                Fibonacci_LAST,
                Fibonacci_FIRST,
                Fibonacci_x,
                Fibonacci_y,

            };
        };

        RefVector<DataType> get_to_be_shifted()
        {
            return {
                Fibonacci_x,
                Fibonacci_y,

            };
        };

        RefVector<DataType> get_shifted()
        {
            return {
                Fibonacci_x_shift,
                Fibonacci_y_shift,

            };
        };
    };

  public:
    class ProvingKey : public ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>> {
      public:
        // Expose constructors on the base class
        using Base = ProvingKey_<PrecomputedEntities<Polynomial>, WitnessEntities<Polynomial>>;
        using Base::Base;

        // The plookup wires that store plookup read data.
        std::array<PolynomialHandle, 0> get_table_column_wires() { return {}; };
    };

    using VerificationKey = VerificationKey_<PrecomputedEntities<Commitment>>;

    using ProverPolynomials = AllEntities<PolynomialHandle>;

    using FoldedPolynomials = AllEntities<std::vector<FF>>;

    class AllValues : public AllEntities<FF> {
      public:
        using Base = AllEntities<FF>;
        using Base::Base;
    };

    class AllPolynomials : public AllEntities<Polynomial> {
      public:
        [[nodiscard]] size_t get_polynomial_size() const { return this->Fibonacci_LAST.size(); }
        [[nodiscard]] AllValues get_row(const size_t row_idx) const
        {
            AllValues result;
            for (auto [result_field, polynomial] : zip_view(result.get_all(), get_all())) {
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
    template <size_t LENGTH> using ProverUnivariates = AllEntities<barretenberg::Univariate<FF, LENGTH>>;

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
            Base::Fibonacci_LAST = "Fibonacci_LAST";
            Base::Fibonacci_FIRST = "Fibonacci_FIRST";
            Base::Fibonacci_x = "Fibonacci_x";
            Base::Fibonacci_y = "Fibonacci_y";
        };
    };

    class VerifierCommitments : public AllEntities<Commitment> {
      private:
        using Base = AllEntities<Commitment>;

      public:
        VerifierCommitments(const std::shared_ptr<VerificationKey>& verification_key)
        {
            Fibonacci_LAST = verification_key->Fibonacci_LAST;
            Fibonacci_FIRST = verification_key->Fibonacci_FIRST;
        }
    };

    class Transcript : public BaseTranscript {
      public:
        uint32_t circuit_size;

        Commitment Fibonacci_x;
        Commitment Fibonacci_y;

        std::vector<barretenberg::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>> sumcheck_univariates;
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

            Fibonacci_x = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);
            Fibonacci_y = deserialize_from_buffer<Commitment>(BaseTranscript::proof_data, num_bytes_read);

            for (size_t i = 0; i < log_n; ++i) {
                sumcheck_univariates.emplace_back(
                    deserialize_from_buffer<barretenberg::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>>(
                        BaseTranscript::proof_data, num_bytes_read));
            }
            sumcheck_evaluations =
                deserialize_from_buffer<std::array<FF, NUM_ALL_ENTITIES>>(BaseTranscript::proof_data, num_bytes_read);
            for (size_t i = 0; i < log_n; ++i) {
                zm_cq_comms.push_back(deserialize_from_buffer<Commitment>(proof_data, num_bytes_read));
            }
            zm_cq_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
            zm_pi_comm = deserialize_from_buffer<Commitment>(proof_data, num_bytes_read);
        }

        void serialize_full_transcript()
        {
            size_t old_proof_length = proof_data.size();
            BaseTranscript::proof_data.clear();
            size_t log_n = numeric::get_msb(circuit_size);

            serialize_to_buffer(circuit_size, BaseTranscript::proof_data);

            serialize_to_buffer<Commitment>(Fibonacci_x, BaseTranscript::proof_data);
            serialize_to_buffer<Commitment>(Fibonacci_y, BaseTranscript::proof_data);

            for (size_t i = 0; i < log_n; ++i) {
                serialize_to_buffer(sumcheck_univariates[i], BaseTranscript::proof_data);
            }
            serialize_to_buffer(sumcheck_evaluations, BaseTranscript::proof_data);
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
} // namespace proof_system::honk
