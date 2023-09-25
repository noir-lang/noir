#pragma once
#include "barretenberg/honk/instance/prover_instance.hpp"
#include "barretenberg/honk/instance/verifier_instance.hpp"
namespace proof_system::honk {

template <typename Flavor_, size_t NUM_> struct ProverInstances_ {
    using Flavor = Flavor_;
    using Instance = ProverInstance_<Flavor>;
    using ArrayType = std::array<std::shared_ptr<Instance>, NUM_>;

  public:
    static constexpr size_t NUM = NUM_;
    ArrayType _data;
    Instance const& operator[](size_t idx) const { return _data[idx]; }
    typename ArrayType::iterator begin() { return _data.begin(); };
    typename ArrayType::iterator end() { return _data.end(); };
    ProverInstances_(std::vector<std::shared_ptr<Instance>> data)
    {
        ASSERT(data.size() == NUM);
        for (size_t idx = 0; idx < data.size(); idx++) {
            _data[idx] = std::move(data[idx]);
        }
    };
};

template <typename Flavor_, size_t NUM_> struct VerifierInstances_ {
    using Flavor = Flavor_;
    using VerificationKey = typename Flavor::VerificationKey;
    using Instance = VerifierInstance_<Flavor>;
    using ArrayType = std::array<Instance, NUM_>;

  public:
    static constexpr size_t NUM = NUM_;
    ArrayType _data;
    Instance const& operator[](size_t idx) const { return _data[idx]; }
    typename ArrayType::iterator begin() { return _data.begin(); };
    typename ArrayType::iterator end() { return _data.end(); };
    VerifierInstances_(std::vector<std::shared_ptr<VerificationKey>> vks)
    {
        ASSERT(vks.size() == NUM);
        for (size_t idx = 0; idx < vks.size(); idx++) {
            Instance inst;
            inst.verification_key = std::move(vks[idx]);
            _data[idx] = inst;
        }
    };
};
} // namespace proof_system::honk