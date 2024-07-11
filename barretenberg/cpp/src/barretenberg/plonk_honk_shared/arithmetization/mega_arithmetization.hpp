#pragma once

#include "barretenberg/plonk_honk_shared/arithmetization/arithmetization.hpp"

namespace bb {

/**
 * @brief Mega arithmetization
 *
 * @tparam FF_
 */
template <typename FF_> class MegaArith {

    /**
     * @brief Defines the circuit block types for the Mega arithmetization
     * @note Its useful to define this as a template since it is used to actually store gate data (T = MegaTraceBlock)
     * but also to store corresponding block sizes (T = uint32_t) for the structured trace or dynamic block size
     * tracking in ClientIvc.
     *
     * @tparam T
     */
    template <typename T> struct MegaTraceBlocks {
        T ecc_op;
        T pub_inputs;
        T arithmetic;
        T delta_range;
        T elliptic;
        T aux;
        T lookup;
        T busread;
        T poseidon_external;
        T poseidon_internal;

        auto get()
        {
            return RefArray{ ecc_op, pub_inputs, arithmetic, delta_range,       elliptic,
                             aux,    lookup,     busread,    poseidon_external, poseidon_internal };
        }

        bool operator==(const MegaTraceBlocks& other) const = default;
    };

    // An arbitrary but small-ish structuring that can be used for generic unit testing with non-trivial circuits
    struct SmallTestStructuredBlockSizes : public MegaTraceBlocks<uint32_t> {
        SmallTestStructuredBlockSizes()
        {
            const uint32_t FIXED_SIZE = 1 << 14;
            this->ecc_op = FIXED_SIZE;
            this->pub_inputs = FIXED_SIZE;
            this->arithmetic = FIXED_SIZE;
            this->delta_range = FIXED_SIZE;
            this->elliptic = FIXED_SIZE;
            this->aux = FIXED_SIZE;
            this->lookup = FIXED_SIZE;
            this->busread = FIXED_SIZE;
            this->poseidon_external = FIXED_SIZE;
            this->poseidon_internal = FIXED_SIZE;
        }
    };

    // A minimal structuring specifically tailored to the medium complexity transaction for the ClientIvc benchmark
    struct ClientIvcBenchStructuredBlockSizes : public MegaTraceBlocks<uint32_t> {
        ClientIvcBenchStructuredBlockSizes()
        {
            this->ecc_op = 1 << 10;
            this->pub_inputs = 1 << 7;
            this->arithmetic = 1 << 16;
            this->delta_range = 1 << 15;
            this->elliptic = 1 << 14;
            this->aux = 1 << 16;
            this->lookup = 1 << 15;
            this->busread = 1 << 7;
            this->poseidon_external = 1 << 11;
            this->poseidon_internal = 1 << 14;
        }
    };

    // Structuring tailored to the full e2e TS test (TO BE UPDATED ACCORDINGLY)
    struct E2eStructuredBlockSizes : public MegaTraceBlocks<uint32_t> {
        E2eStructuredBlockSizes()
        {
            this->ecc_op = 1 << 10;
            this->pub_inputs = 30000;
            this->arithmetic = 600000;
            this->delta_range = 140000;
            this->elliptic = 600000;
            this->aux = 1400000;
            this->lookup = 460000;
            this->busread = 1 << 7;
            this->poseidon_external = 15000;
            this->poseidon_internal = 85000;
        }
    };

  public:
    static constexpr size_t NUM_WIRES = 4;
    static constexpr size_t NUM_SELECTORS = 14;

    using FF = FF_;

    class MegaTraceBlock : public ExecutionTraceBlock<FF, NUM_WIRES, NUM_SELECTORS> {
      public:
        void populate_wires(const uint32_t& idx_1, const uint32_t& idx_2, const uint32_t& idx_3, const uint32_t& idx_4)
        {
#ifdef CHECK_CIRCUIT_STACKTRACES
            this->stack_traces.populate();
#endif
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

    struct TraceBlocks : public MegaTraceBlocks<MegaTraceBlock> {

        E2eStructuredBlockSizes fixed_block_sizes;

        // Set fixed block sizes for use in structured trace
        void set_fixed_block_sizes(TraceStructure setting)
        {
            MegaTraceBlocks<uint32_t> fixed_block_sizes{}; // zero initialized

            switch (setting) {
            case TraceStructure::NONE:
                break;
            case TraceStructure::SMALL_TEST:
                fixed_block_sizes = SmallTestStructuredBlockSizes();
                break;
            case TraceStructure::CLIENT_IVC_BENCH:
                fixed_block_sizes = ClientIvcBenchStructuredBlockSizes();
                break;
            case TraceStructure::E2E_FULL_TEST:
                fixed_block_sizes = E2eStructuredBlockSizes();
                break;
            }
            for (auto [block, size] : zip_view(this->get(), fixed_block_sizes.get())) {
                block.set_fixed_size(size);
            }
        }

        TraceBlocks()
        {
            this->aux.has_ram_rom = true;
            this->pub_inputs.is_pub_inputs = true;
        }

        void summarize() const
        {
            info("Gate blocks summary: (actual gates / fixed capacity)");
            info("goblin ecc op :\t", this->ecc_op.size(), "/", this->ecc_op.get_fixed_size());
            info("pub inputs    :\t", this->pub_inputs.size(), "/", this->pub_inputs.get_fixed_size());
            info("arithmetic    :\t", this->arithmetic.size(), "/", this->arithmetic.get_fixed_size());
            info("delta range   :\t", this->delta_range.size(), "/", this->delta_range.get_fixed_size());
            info("elliptic      :\t", this->elliptic.size(), "/", this->elliptic.get_fixed_size());
            info("auxiliary     :\t", this->aux.size(), "/", this->aux.get_fixed_size());
            info("lookups       :\t", this->lookup.size(), "/", this->lookup.get_fixed_size());
            info("busread       :\t", this->busread.size(), "/", this->busread.get_fixed_size());
            info("poseidon ext  :\t", this->poseidon_external.size(), "/", this->poseidon_external.get_fixed_size());
            info("poseidon int  :\t", this->poseidon_internal.size(), "/", this->poseidon_internal.get_fixed_size());
            info("");
        }

        size_t get_total_structured_size()
        {
            size_t total_size = 0;
            for (auto block : this->get()) {
                total_size += block.get_fixed_size();
            }
            return total_size;
        }

        void check_within_fixed_sizes()
        {
            for (auto block : this->get()) {
                if (block.size() > block.get_fixed_size()) {
                    info("WARNING: Num gates in circuit block exceeds the specified fixed size - execution trace will "
                         "not be constructed correctly!");
                    summarize();
                    ASSERT(false);
                }
            }
        }

        bool operator==(const TraceBlocks& other) const = default;
    };

    // Note: Unused. Needed only for consistency with Ultra arith (which is used by Plonk)
    inline static const std::vector<std::string> selector_names = {};
};

template <typename T>
concept HasAdditionalSelectors = IsAnyOf<T, MegaArith<bb::fr>>;
} // namespace bb