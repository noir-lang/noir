#pragma once
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include <array>
#include <barretenberg/common/slab_allocator.hpp>
#include <cstddef>
#include <vector>

namespace arithmetization {

/**
 * @brief Specify the structure of a CircuitBuilder
 *
 * @details This is typically passed as a template argument specifying the structure of a circuit constructor. It
 * should only ever contain circuit constructor data--it should not contain data that is particular to any
 * proving system.
 *
 * @remark It may make sense to say this is only partial arithmetization data, with the full data being
 * contained in the circuit constructor. We could change the name of this class if it conflicts with common usage.
 *
 * @note For even greater modularity, in each instantiation we could specify a list of components here, where a
 * component is a meaningful collection of functions for creating gates, as in:
 *
 * struct Component {
 *     using Arithmetic = component::Arithmetic3Wires;
 *     using RangeConstraints = component::Base4Accumulators or component::GenPerm or...
 *     using LookupTables = component::Plookup4Wire or component::CQ8Wire or...
 *     ...
 * };
 *
 * We should only do this if it becomes necessary or convenient.
 */

// These are not magic numbers and they should not be written with global constants. These parameters are not accessible
// through clearly named static class members.
template <typename FF_> class Standard {
  public:
    static constexpr size_t NUM_WIRES = 3;
    static constexpr size_t num_selectors = 5;
    using FF = FF_;
    using SelectorType = std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>;

    std::vector<SelectorType> selectors;

    SelectorType& q_m() { return selectors[0]; };
    SelectorType& q_1() { return selectors[1]; };
    SelectorType& q_2() { return selectors[2]; };
    SelectorType& q_3() { return selectors[3]; };
    SelectorType& q_c() { return selectors[4]; };

    Standard()
        : selectors(num_selectors)
    {}

    const auto& get() const { return selectors; };

    void reserve(size_t size_hint)
    {
        for (auto& p : selectors) {
            p.reserve(size_hint);
        }
    }

    // Note: These are needed for Plonk only (for poly storage in a std::map). Must be in same order as above struct.
    inline static const std::vector<std::string> selector_names = { "q_m", "q_1", "q_2", "q_3", "q_c" };
};

template <typename FF_> class Ultra {
  public:
    static constexpr size_t NUM_WIRES = 4;
    static constexpr size_t num_selectors = 11;
    using FF = FF_;
    using SelectorType = std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>;

    std::vector<SelectorType> selectors;

    SelectorType& q_m() { return selectors[0]; };
    SelectorType& q_c() { return selectors[1]; };
    SelectorType& q_1() { return selectors[2]; };
    SelectorType& q_2() { return selectors[3]; };
    SelectorType& q_3() { return selectors[4]; };
    SelectorType& q_4() { return selectors[5]; };
    SelectorType& q_arith() { return selectors[6]; };
    SelectorType& q_sort() { return selectors[7]; };
    SelectorType& q_elliptic() { return selectors[8]; };
    SelectorType& q_aux() { return selectors[9]; };
    SelectorType& q_lookup_type() { return selectors[10]; };

    Ultra()
        : selectors(num_selectors)
    {}

    const auto& get() const { return selectors; };

    void reserve(size_t size_hint)
    {
        for (auto& p : selectors) {
            p.reserve(size_hint);
        }
    }

    // Note: These are needed for Plonk only (for poly storage in a std::map). Must be in same order as above struct.
    inline static const std::vector<std::string> selector_names = { "q_m",        "q_c",   "q_1",       "q_2",
                                                                    "q_3",        "q_4",   "q_arith",   "q_sort",
                                                                    "q_elliptic", "q_aux", "table_type" };
};

class GoblinTranslator {
  public:
    static constexpr size_t NUM_WIRES = 81;
    static constexpr size_t num_selectors = 0;
};
} // namespace arithmetization