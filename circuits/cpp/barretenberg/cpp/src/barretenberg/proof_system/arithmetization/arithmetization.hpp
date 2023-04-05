#pragma once
#include <cstddef>

namespace arithmetization {

/**
 * @brief Specify the structure of a CircuitConstructor
 *
 * @details This is typically passed as a template argument specifying the structure of a circuit constructor. It
 * should only ever contain circuit constructor data--it should not contain data that is particular to any
 * proving system.
 *
 * @remark It may make sense to say this is only partial arithmetization data, with the full data being
 * contained in the circuit constructor. We could change the name of this class if it conflicts with common usage.
 *
 * @tparam _num_wires
 * @tparam _num_selectors
 */
template <size_t _num_wires, size_t _num_selectors> struct Arithmetization {
    static constexpr size_t num_wires = _num_wires;
    static constexpr size_t num_selectors = _num_selectors;
    // Note: For even greater modularity, in each instantiation we could specify a list of components here, where a
    // component is a meaningful collection of functions for creating gates, as in:
    //
    // struct Component {
    //     using Arithmetic = component::Arithmetic3Wires;
    //     using RangeConstraints = component::Base4Accumulators or component::GenPerm or...
    //     using LooupTables = component::Plookup4Wire or component::CQ8Wire or...
    //     ...
    // };
    //
    // We should only do this if it becomes necessary or convenient.
};

// These are not magic numbers and they should not be written with global constants. These paraters are not accessible
// through clearly named static class members.
using Standard = Arithmetization</*num_wires =*/3, /*num_selectors =*/5>;
using Turbo = Arithmetization</*num_wires =*/4, /*num_selectors =*/11>;
using Ultra = Arithmetization</*num_wires =*/4, /*num_selectors =*/11>;

} // namespace arithmetization