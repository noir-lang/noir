#pragma once
#include "barretenberg/sumcheck/instance/prover_instance.hpp"
#include "barretenberg/sumcheck/instance/verifier_instance.hpp"

namespace proof_system::honk {

template <typename Flavor_, size_t NUM_> struct ProverInstances_ {
  public:
    using Flavor = Flavor_;
    using FF = typename Flavor::FF;
    static constexpr size_t NUM = NUM_;
    using Instance = ProverInstance_<Flavor>;

    using ArrayType = std::array<std::shared_ptr<Instance>, NUM_>;
    // The extended length here is the length of a composition of polynomials.
    static constexpr size_t EXTENDED_LENGTH = (Flavor::MAX_TOTAL_RELATION_LENGTH - 1) * (NUM - 1) + 1;
    using RelationParameters = proof_system::RelationParameters<Univariate<FF, EXTENDED_LENGTH>>;

    ArrayType _data;
    RelationParameters relation_parameters;

    std::shared_ptr<Instance> const& operator[](size_t idx) const { return _data[idx]; }
    typename ArrayType::iterator begin() { return _data.begin(); };
    typename ArrayType::iterator end() { return _data.end(); };
    ProverInstances_() = default;
    ProverInstances_(std::vector<std::shared_ptr<Instance>> data)
    {
        ASSERT(data.size() == NUM);
        for (size_t idx = 0; idx < data.size(); idx++) {
            _data[idx] = std::move(data[idx]);
        }
    };

    /**
     * @brief  For a prover polynomial label and a fixed row index, construct a uninvariate from the corresponding value
     * from each instance.
     *
     * @example if the prover polynomia index is 1 and the row index is 2, and there are 4 instances visually we have
     *
     *           Instance 0       Instance 1       Instance 2       Instance 3
     *           q_c q_l q_r ...  q_c q_l q_r ...  q_c q_l q_r ...  q_c q_l q_r ...
     *           *   *            *   *            *   *            *   *
     *           *   *            *   *            *   *            *   *
     *           *   a            *   b            *   c            *   d
     *           *   *            *   *            *   *            *   *
     *
     * and the function returns the univariate {a, b, c, d}
     *
     * @param entity_idx A fixed column position in several execution traces.
     * @param row_idx A fixed row position in several execution
     * @return Univariate<FF, NUM> The univariate whose extensions will be used to construct the combiner.
     */
    Univariate<FF, NUM> row_to_univariate(const size_t prover_polynomial_idx, const size_t row_idx) const
    {
        Univariate<FF, NUM> result;
        size_t instance_idx = 0;
        for (auto& instance : _data) {
            result.evaluations[instance_idx] = instance->prover_polynomials._data[prover_polynomial_idx][row_idx];
            instance_idx++;
        }
        return result;
    }
};

template <typename Flavor_, size_t NUM_> struct VerifierInstances_ {
    using Flavor = Flavor_;
    using VerificationKey = typename Flavor::VerificationKey;
    using Instance = VerifierInstance_<Flavor>;
    using ArrayType = std::array<std::shared_ptr<Instance>, NUM_>;

  public:
    static constexpr size_t NUM = NUM_;
    ArrayType _data;
    std::shared_ptr<Instance> const& operator[](size_t idx) const { return _data[idx]; }
    typename ArrayType::iterator begin() { return _data.begin(); };
    typename ArrayType::iterator end() { return _data.end(); };
    VerifierInstances_(std::vector<std::shared_ptr<VerificationKey>> vks)
    {
        ASSERT(vks.size() == NUM);
        for (size_t idx = 0; idx < vks.size(); idx++) {
            Instance inst;
            inst.verification_key = std::move(vks[idx]);
            _data[idx] = std::make_unique<Instance>(inst);
        }
    };
};
} // namespace proof_system::honk