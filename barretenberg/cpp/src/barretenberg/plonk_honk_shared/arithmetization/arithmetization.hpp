#pragma once
#include "barretenberg/common/ref_array.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/plonk_honk_shared/types/circuit_type.hpp"
#include <array>
#include <barretenberg/common/slab_allocator.hpp>
#include <cstddef>
#include <vector>

namespace bb {

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
 *     using RangeConstraints = component::Base4Accumulators or component::DeltaRangeConstraint or...
 *     using LookupTables = component::Plookup4Wire or component::CQ8Wire or...
 *     ...
 * };
 *
 * We should only do this if it becomes necessary or convenient.
 */

/**
 * @brief Basic structure for storing gate data in a builder
 *
 * @tparam FF
 * @tparam NUM_WIRES
 * @tparam NUM_SELECTORS
 */
template <typename FF, size_t NUM_WIRES, size_t NUM_SELECTORS> class ExecutionTraceBlock {
  public:
    using SelectorType = std::vector<FF, bb::ContainerSlabAllocator<FF>>;
    using WireType = std::vector<uint32_t, bb::ContainerSlabAllocator<uint32_t>>;
    using Selectors = std::array<SelectorType, NUM_SELECTORS>;
    using Wires = std::array<WireType, NUM_WIRES>;

    Wires wires; // vectors of indices into a witness variables array
    Selectors selectors;
    bool has_ram_rom = false;   // does the block contain RAM/ROM gates
    bool is_pub_inputs = false; // is this the public inputs block

    bool operator==(const ExecutionTraceBlock& other) const = default;

    size_t size() const { return std::get<0>(this->wires).size(); }

    void reserve(size_t size_hint)
    {
        for (auto& w : wires) {
            w.reserve(size_hint);
        }
        for (auto& p : selectors) {
            p.reserve(size_hint);
        }
    }
};

// These are not magic numbers and they should not be written with global constants. These parameters are not
// accessible through clearly named static class members.
template <typename FF_> class StandardArith {
  public:
    static constexpr size_t NUM_WIRES = 3;
    static constexpr size_t NUM_SELECTORS = 5;
    using FF = FF_;

    class StandardTraceBlock : public ExecutionTraceBlock<FF, NUM_WIRES, NUM_SELECTORS> {
      public:
        void populate_wires(const uint32_t& idx_1, const uint32_t& idx_2, const uint32_t& idx_3)
        {
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

template <typename FF_> class UltraArith {
  public:
    static constexpr size_t NUM_WIRES = 4;
    static constexpr size_t NUM_SELECTORS = 11;
    using FF = FF_;

    class UltraTraceBlock : public ExecutionTraceBlock<FF, NUM_WIRES, NUM_SELECTORS> {
      public:
        void populate_wires(const uint32_t& idx_1, const uint32_t& idx_2, const uint32_t& idx_3, const uint32_t& idx_4)
        {
            this->wires[0].emplace_back(idx_1);
            this->wires[1].emplace_back(idx_2);
            this->wires[2].emplace_back(idx_3);
            this->wires[3].emplace_back(idx_4);
        }

        auto& w_l() { return std::get<0>(this->wires); };
        auto& w_r() { return std::get<1>(this->wires); };
        auto& w_o() { return std::get<2>(this->wires); };
        auto& w_4() { return std::get<3>(this->wires); };

        auto& q_m() { return this->selectors[0]; };
        auto& q_c() { return this->selectors[1]; };
        auto& q_1() { return this->selectors[2]; };
        auto& q_2() { return this->selectors[3]; };
        auto& q_3() { return this->selectors[4]; };
        auto& q_4() { return this->selectors[5]; };
        auto& q_arith() { return this->selectors[6]; };
        auto& q_delta_range() { return this->selectors[7]; };
        auto& q_elliptic() { return this->selectors[8]; };
        auto& q_aux() { return this->selectors[9]; };
        auto& q_lookup_type() { return this->selectors[10]; };
    };

    struct TraceBlocks {
        UltraTraceBlock pub_inputs;
        UltraTraceBlock arithmetic;
        UltraTraceBlock delta_range;
        UltraTraceBlock elliptic;
        UltraTraceBlock aux;
        UltraTraceBlock lookup;

        TraceBlocks()
        {
            aux.has_ram_rom = true;
            pub_inputs.is_pub_inputs = true;
        }

        auto get() { return RefArray{ pub_inputs, arithmetic, delta_range, elliptic, aux, lookup }; }

        void summarize()
        {
            info("Gate blocks summary:");
            info("pub inputs:\t", pub_inputs.size());
            info("arithmetic:\t", arithmetic.size());
            info("delta range:\t", delta_range.size());
            info("elliptic:\t", elliptic.size());
            info("auxiliary:\t", aux.size());
            info("lookups:\t", lookup.size());
            info("");
        }

        bool operator==(const TraceBlocks& other) const = default;
    };

    // Note: These are needed for Plonk only (for poly storage in a std::map). Must be in same order as above struct.
    inline static const std::vector<std::string> selector_names = { "q_m",        "q_c",   "q_1",       "q_2",
                                                                    "q_3",        "q_4",   "q_arith",   "q_sort",
                                                                    "q_elliptic", "q_aux", "table_type" };
};

/**
 * @brief Ultra Honk arithmetization
 * @details Extends the conventional Ultra arithmetization with a new selector related to databus lookups
 *
 * @tparam FF_
 */
template <typename FF_> class UltraHonkArith {
  public:
    static constexpr size_t NUM_WIRES = 4;
    static constexpr size_t NUM_SELECTORS = 14;
    using FF = FF_;

    class UltraHonkTraceBlock : public ExecutionTraceBlock<FF, NUM_WIRES, NUM_SELECTORS> {
      public:
        void populate_wires(const uint32_t& idx_1, const uint32_t& idx_2, const uint32_t& idx_3, const uint32_t& idx_4)
        {
            this->wires[0].emplace_back(idx_1);
            this->wires[1].emplace_back(idx_2);
            this->wires[2].emplace_back(idx_3);
            this->wires[3].emplace_back(idx_4);
        }

        auto& w_l() { return std::get<0>(this->wires); };
        auto& w_r() { return std::get<1>(this->wires); };
        auto& w_o() { return std::get<2>(this->wires); };
        auto& w_4() { return std::get<3>(this->wires); };

        auto& q_m() { return this->selectors[0]; };
        auto& q_c() { return this->selectors[1]; };
        auto& q_1() { return this->selectors[2]; };
        auto& q_2() { return this->selectors[3]; };
        auto& q_3() { return this->selectors[4]; };
        auto& q_4() { return this->selectors[5]; };
        auto& q_arith() { return this->selectors[6]; };
        auto& q_delta_range() { return this->selectors[7]; };
        auto& q_elliptic() { return this->selectors[8]; };
        auto& q_aux() { return this->selectors[9]; };
        auto& q_lookup_type() { return this->selectors[10]; };
        auto& q_busread() { return this->selectors[11]; };
        auto& q_poseidon2_external() { return this->selectors[12]; };
        auto& q_poseidon2_internal() { return this->selectors[13]; };

        /**
         * @brief Add zeros to all selectors which are not part of the conventional Ultra arithmetization
         * @details Facilitates reuse of Ultra gate construction functions in arithmetizations which extend the
         * conventional Ultra arithmetization
         *
         */
        void pad_additional()
        {
            q_busread().emplace_back(0);
            q_poseidon2_external().emplace_back(0);
            q_poseidon2_internal().emplace_back(0);
        };

        /**
         * @brief Resizes all selectors which are not part of the conventional Ultra arithmetization
         * @details Facilitates reuse of Ultra gate construction functions in arithmetizations which extend the
         * conventional Ultra arithmetization
         * @param new_size
         */
        void resize_additional(size_t new_size)
        {
            q_busread().resize(new_size);
            q_poseidon2_external().resize(new_size);
            q_poseidon2_internal().resize(new_size);
        };
    };

    struct TraceBlocks {
        UltraHonkTraceBlock ecc_op;
        UltraHonkTraceBlock pub_inputs;
        UltraHonkTraceBlock arithmetic;
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/919): Change: DeltaRangeConstraint -->
        // DeltaRangeConstraint
        UltraHonkTraceBlock delta_range;
        UltraHonkTraceBlock elliptic;
        UltraHonkTraceBlock aux;
        UltraHonkTraceBlock lookup;
        UltraHonkTraceBlock busread;
        UltraHonkTraceBlock poseidon_external;
        UltraHonkTraceBlock poseidon_internal;

        TraceBlocks()
        {
            aux.has_ram_rom = true;
            pub_inputs.is_pub_inputs = true;
        }

        auto get()
        {
            return RefArray{ ecc_op, pub_inputs, arithmetic, delta_range,       elliptic,
                             aux,    lookup,     busread,    poseidon_external, poseidon_internal };
        }

        void summarize()
        {
            info("Gate blocks summary:");
            info("goblin ecc op:\t", ecc_op.size());
            info("pub inputs:\t", pub_inputs.size());
            info("arithmetic:\t", arithmetic.size());
            info("delta range:\t", delta_range.size());
            info("elliptic:\t", elliptic.size());
            info("auxiliary:\t", aux.size());
            info("lookups:\t", lookup.size());
            info("busread:\t", busread.size());
            info("poseidon ext:\t", poseidon_external.size());
            info("poseidon int:\t", poseidon_internal.size());
            info("");
        }

        bool operator==(const TraceBlocks& other) const = default;
    };

    // Note: Unused. Needed only for consistency with Ultra arith (which is used by Plonk)
    inline static const std::vector<std::string> selector_names = {};
};

class GoblinTranslatorArith {
  public:
    static constexpr size_t NUM_WIRES = 81;
    static constexpr size_t NUM_SELECTORS = 0;
};

template <typename T>
concept HasAdditionalSelectors = IsAnyOf<T, UltraHonkArith<bb::fr>>;
} // namespace bb