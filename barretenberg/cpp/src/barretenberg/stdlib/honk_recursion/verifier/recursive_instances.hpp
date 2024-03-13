#pragma once
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/stdlib/honk_recursion/verifier/recursive_verifier_instance.hpp"

namespace bb::stdlib::recursion::honk {
template <IsRecursiveFlavor Flavor_, size_t NUM_> struct RecursiveVerifierInstances_ {
    using Flavor = Flavor_;
    using Builder = typename Flavor::CircuitBuilder;
    using VerificationKey = typename Flavor::VerificationKey;
    using NativeVerificationKey = typename Flavor::NativeVerificationKey;
    using Instance = RecursiveVerifierInstance_<Flavor>;
    using NativeInstance = bb::VerifierInstance_<typename Flavor::NativeFlavor>;
    using ArrayType = std::array<std::shared_ptr<Instance>, NUM_>;

  public:
    static constexpr size_t NUM = NUM_;
    static constexpr size_t BATCHED_EXTENDED_LENGTH = (Flavor::MAX_TOTAL_RELATION_LENGTH - 1 + NUM - 1) * (NUM - 1) + 1;
    ArrayType _data;
    std::shared_ptr<Instance> const& operator[](size_t idx) const { return _data[idx]; }
    typename ArrayType::iterator begin() { return _data.begin(); };
    typename ArrayType::iterator end() { return _data.end(); };
    Builder* builder;

    RecursiveVerifierInstances_(Builder* builder,
                                const std::shared_ptr<NativeInstance>& accumulator,
                                const std::vector<std::shared_ptr<NativeVerificationKey>>& vks)
        : builder(builder)
    {
        ASSERT(vks.size() == NUM - 1);
        if (accumulator->is_accumulator) {
            _data[0] = std::make_shared<Instance>(builder, accumulator);
        } else {
            _data[0] = std::make_shared<Instance>(builder, accumulator->verification_key);
        }
        size_t idx = 1;
        for (auto& vk : vks) {
            _data[idx] = std::make_shared<Instance>(builder, vk);
            idx++;
        }
    }
};
} // namespace bb::stdlib::recursion::honk
