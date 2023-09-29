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
 * @tparam _NUM_WIRES
 * @tparam _num_selectors
 */
template <size_t _NUM_WIRES, size_t _num_selectors> struct Arithmetization {
    static constexpr size_t NUM_WIRES = _NUM_WIRES;
    static constexpr size_t num_selectors = _num_selectors;

    // Note: For even greater modularity, in each instantiation we could specify a list of components here, where a
    // component is a meaningful collection of functions for creating gates, as in:
    //
    // struct Component {
    //     using Arithmetic = component::Arithmetic3Wires;
    //     using RangeConstraints = component::Base4Accumulators or component::GenPerm or...
    //     using LookupTables = component::Plookup4Wire or component::CQ8Wire or...
    //     ...
    // };
    //
    // We should only do this if it becomes necessary or convenient.
};

template <typename FF, size_t num_selectors> struct SelectorsBase {
    using DataType = std::array<std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>, num_selectors>;
    DataType _data;
    size_t size() { return _data.size(); };
    typename DataType::const_iterator begin() const { return _data.begin(); };
    typename DataType::iterator begin() { return _data.begin(); };
    typename DataType::const_iterator end() const { return _data.end(); };
    typename DataType::iterator end() { return _data.end(); };
};

// These are not magic numbers and they should not be written with global constants. These parameters are not accessible
// through clearly named static class members.
template <typename _FF> class Standard : public Arithmetization</*NUM_WIRES =*/3, /*num_selectors =*/5> {
  public:
    using FF = _FF;
    struct Selectors : SelectorsBase<FF, num_selectors> {
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_m = std::get<0>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_1 = std::get<1>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_2 = std::get<2>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_3 = std::get<3>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_c = std::get<4>(this->_data);
        Selectors()
            : SelectorsBase<FF, num_selectors>(){};
        Selectors(const Selectors& other)
            : SelectorsBase<FF, num_selectors>(other)
        {}
        Selectors(Selectors&& other)
        {
            this->_data = std::move(other._data);
            this->q_m = std::get<0>(this->_data);
            this->q_1 = std::get<1>(this->_data);
            this->q_2 = std::get<2>(this->_data);
            this->q_3 = std::get<3>(this->_data);
            this->q_c = std::get<4>(this->_data);
        };
        Selectors& operator=(Selectors&& other)
        {
            SelectorsBase<FF, num_selectors>::operator=(other);
            return *this;
        }
        ~Selectors() = default;
    };
};

template <typename _FF> class Turbo : public Arithmetization</*NUM_WIRES =*/4, /*num_selectors =*/11> {
  public:
    using FF = _FF;
    struct Selectors : SelectorsBase<FF, num_selectors> {
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_m = std::get<0>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_c = std::get<1>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_1 = std::get<2>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_2 = std::get<3>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_3 = std::get<4>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_4 = std::get<5>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_5 = std::get<6>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_arith = std::get<7>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_fixed_base = std::get<8>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_range = std::get<9>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_logic = std::get<10>(this->_data);
        Selectors()
            : SelectorsBase<FF, num_selectors>(){};
        Selectors(const Selectors& other)
            : SelectorsBase<FF, num_selectors>(other)
        {}
        Selectors(Selectors&& other)
        {
            this->_data = std::move(other._data);
            this->q_m = std::get<0>(this->_data);
            this->q_c = std::get<1>(this->_data);
            this->q_1 = std::get<2>(this->_data);
            this->q_2 = std::get<3>(this->_data);
            this->q_3 = std::get<4>(this->_data);
            this->q_4 = std::get<5>(this->_data);
            this->q_5 = std::get<6>(this->_data);
            this->q_arith = std::get<7>(this->_data);
            this->q_fixed_base = std::get<8>(this->_data);
            this->q_range = std::get<9>(this->_data);
            this->q_logic = std::get<10>(this->_data);
        };
        Selectors& operator=(Selectors&& other)
        {
            SelectorsBase<FF, num_selectors>::operator=(other);
            return *this;
        }
        ~Selectors() = default;
    };
};

template <typename _FF> class Ultra : public Arithmetization</*NUM_WIRES =*/4, /*num_selectors =*/12> {
  public:
    using FF = _FF;
    struct Selectors : SelectorsBase<FF, num_selectors> {
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_m = std::get<0>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_c = std::get<1>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_1 = std::get<2>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_2 = std::get<3>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_3 = std::get<4>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_4 = std::get<5>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_arith = std::get<6>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_sort = std::get<7>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_elliptic = std::get<8>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_aux = std::get<9>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_lookup_type = std::get<10>(this->_data);
        std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>& q_elliptic_double = std::get<11>(this->_data);
        Selectors()
            : SelectorsBase<FF, num_selectors>(){};
        Selectors(const Selectors& other)
            : SelectorsBase<FF, num_selectors>(other)
        {}
        Selectors(Selectors&& other)
        {
            this->_data = std::move(other._data);
            this->q_m = std::get<0>(this->_data);
            this->q_c = std::get<1>(this->_data);
            this->q_1 = std::get<2>(this->_data);
            this->q_2 = std::get<3>(this->_data);
            this->q_3 = std::get<4>(this->_data);
            this->q_4 = std::get<5>(this->_data);
            this->q_arith = std::get<6>(this->_data);
            this->q_sort = std::get<7>(this->_data);
            this->q_elliptic = std::get<8>(this->_data);
            this->q_aux = std::get<9>(this->_data);
            this->q_lookup_type = std::get<10>(this->_data);
            this->q_elliptic_double = std::get<11>(this->_data);
        };
        Selectors& operator=(Selectors&& other)
        {
            SelectorsBase<FF, num_selectors>::operator=(other);
            return *this;
        }
        ~Selectors() = default;
        // Selectors() = default;
        // Selectors(const Selectors& other) = default;
        // Selectors(Selectors&& other) = default;
        // Selectors& operator=(Selectors const& other) = default;
        // Selectors& operator=(Selectors&& other) = default;
        // ~Selectors() = default;
    };
};
class GoblinTranslator : public Arithmetization</*NUM_WIRES =*/78, /*num_selectors =*/0> {
  public:
    // Dirty hack
    using Selectors = bool;
    using FF = curve::BN254::ScalarField;
};
} // namespace arithmetization