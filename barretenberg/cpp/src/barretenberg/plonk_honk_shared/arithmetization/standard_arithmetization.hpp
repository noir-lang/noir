#pragma once

#include "barretenberg/plonk_honk_shared/arithmetization/arithmetization.hpp"

namespace bb {

template <typename FF_> class StandardArith {
  public:
    static constexpr size_t NUM_WIRES = 3;
    static constexpr size_t NUM_SELECTORS = 5;
    using FF = FF_;

    class StandardTraceBlock : public ExecutionTraceBlock<FF, NUM_WIRES, NUM_SELECTORS> {
      public:
        void populate_wires(const uint32_t& idx_1, const uint32_t& idx_2, const uint32_t& idx_3)
        {
#ifdef CHECK_CIRCUIT_STACKTRACES
            this->stack_traces.populate();
#endif
            this->wires[0].emplace_back(idx_1);
            this->wires[1].emplace_back(idx_2);
            this->wires[2].emplace_back(idx_3);
        }

        auto& w_l() { return std::get<0>(this->wires); };
        auto& w_r() { return std::get<1>(this->wires); };
        auto& w_o() { return std::get<2>(this->wires); };
        const auto& w_l() const { return std::get<0>(this->wires); };
        const auto& w_r() const { return std::get<1>(this->wires); };
        const auto& w_o() const { return std::get<2>(this->wires); };

        auto& q_m() { return this->selectors[0]; };
        auto& q_1() { return this->selectors[1]; };
        auto& q_2() { return this->selectors[2]; };
        auto& q_3() { return this->selectors[3]; };
        auto& q_c() { return this->selectors[4]; };
        const auto& q_m() const { return this->selectors[0]; };
        const auto& q_1() const { return this->selectors[1]; };
        const auto& q_2() const { return this->selectors[2]; };
        const auto& q_3() const { return this->selectors[3]; };
        const auto& q_c() const { return this->selectors[4]; };
    };

    struct TraceBlocks {
        StandardTraceBlock pub_inputs;
        StandardTraceBlock arithmetic;

        auto get() { return RefArray{ pub_inputs, arithmetic }; }

        bool operator==(const TraceBlocks& other) const = default;
    };

    // Note: These are needed for Plonk only (for poly storage in a std::map). Must be in same order as above struct.
    inline static const std::vector<std::string> selector_names = { "q_m", "q_1", "q_2", "q_3", "q_c" };
};

} // namespace bb