#pragma once
#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/relations/auxiliary_relation.hpp"
#include "barretenberg/relations/delta_range_constraint_relation.hpp"
#include "barretenberg/relations/ecc_op_queue_relation.hpp"
#include "barretenberg/relations/elliptic_relation.hpp"
#include "barretenberg/relations/poseidon2_external_relation.hpp"
#include "barretenberg/relations/poseidon2_internal_relation.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/ultra_arithmetic_relation.hpp"

#include <optional>

namespace bb {

class UltraCircuitChecker {
  public:
    using FF = bb::fr;
    using Arithmetic = UltraArithmeticRelation<FF>;
    using Elliptic = EllipticRelation<FF>;
    using Auxiliary = AuxiliaryRelation<FF>;
    using DeltaRangeConstraint = DeltaRangeConstraintRelation<FF>;
    using PoseidonExternal = Poseidon2ExternalRelation<FF>;
    using PoseidonInternal = Poseidon2InternalRelation<FF>;
    using Params = RelationParameters<FF>;

    /**
     * @brief Check the correctness of a circuit witness
     * @details Ensures that all relations for a given Ultra arithmetization are satisfied by the witness for each gate
     * in the circuit.
     * @note: This method does not check the permutation relation since this fundamentally depends on grand product
     * polynomials created by the prover. The lookup relation is also not checked for the same reason, however, we do
     * check the correctness of lookup gates by simply ensuring that the inputs to those gates are present in the lookup
     * tables attached to the circuit.
     *
     * @tparam Builder
     * @param builder
     */
    template <typename Builder> static bool check(const Builder& builder);

  private:
    struct TagCheckData;           // Container for data pertaining to generalized permutation tag check
    struct MemoryCheckData;        // Container for data pertaining to RAM/RAM record check
    using Key = std::array<FF, 4>; // Key type for lookup table hash table
    struct HashFunction;           // Custom hash function for lookup table hash table
    using LookupHashTable = std::unordered_set<Key, HashFunction>;

    /**
     * @brief Checks that the provided witness satisfies all gates contained in a single execution trace block
     *
     * @tparam Builder
     * @param builder
     * @param block
     * @param tag_data
     * @param memory_data
     * @param lookup_hash_table
     */
    template <typename Builder>
    static bool check_block(Builder& builder,
                            auto& block,
                            TagCheckData& tag_data,
                            MemoryCheckData& memory_data,
                            LookupHashTable& lookup_hash_table);

    /**
     * @brief Check that a given relation is satisfied for the provided inputs corresponding to a single row
     * @note Assumes the relation constraints should evaluate to zero on each row and thus does not apply to linearly
     * dependent relations like the log derivative lookup argument.
     *
     * @tparam Relation
     * @param values Values of the relation inputs at a single row
     * @param params
     */
    template <typename Relation> static bool check_relation(auto& values, auto& params);

    /**
     * @brief Check whether the values in a lookup gate are contained within a corresponding hash table
     *
     * @param values Inputs to a lookup gate
     * @param lookup_hash_table Preconstructed hash table representing entries of all tables in circuit
     */
    static bool check_lookup(auto& values, auto& lookup_hash_table);

    /**
     * @brief Check whether the left and right running tag products are equal
     * @note By construction, this is in general only true after the last gate has been processed
     *
     * @param tag_data
     */
    static bool check_tag_data(const TagCheckData& tag_data);

    /**
     * @brief Helper for initializing an empty AllValues container of the right Flavor based on Builder
     * @details We construct a Flavor::AllValues object from each row of circuit data so that we can use the Relations
     * to check correctness. UltraFlavor is used for the Ultra builder and GoblinUltraFlavor is used for the GoblinUltra
     * builder
     *
     * @tparam Builder
     */
    template <typename Builder> static auto init_empty_values();

    /**
     * @brief Populate the values required to check the correctness of a single "row" of the circuit
     * @details Populates all wire values (plus shifts) and selectors. Updates running tag product information.
     * Populates 4th wire with memory records (as needed).
     *
     * @tparam Builder
     * @param builder
     * @param values
     * @param tag_data
     * @param idx
     */
    template <typename Builder>
    static void populate_values(
        Builder& builder, auto& block, auto& values, TagCheckData& tag_data, MemoryCheckData& memory_data, size_t idx);

    /**
     * @brief Struct for managing the running tag product data for ensuring tag correctness
     */
    struct TagCheckData {
        FF left_product = FF::one();           // product of (value + γ ⋅ tag)
        FF right_product = FF::one();          // product of (value + γ ⋅ tau[tag])
        const FF gamma = FF::random_element(); // randomness for the tag check

        // We need to include each variable only once
        std::unordered_set<size_t> encountered_variables;
    };

    /**
     * @brief Struct for managing memory record data for ensuring RAM/ROM correctness
     */
    struct MemoryCheckData {
        FF eta = FF::random_element(); // randomness for constructing wire 4 mem records

        std::unordered_set<size_t> read_record_gates;  // row indices for gates containing RAM/ROM read mem record
        std::unordered_set<size_t> write_record_gates; // row indices for gates containing RAM/ROM write mem record
        // Construct hash tables for memory read/write indices to efficiently determine if row is a memory record
        MemoryCheckData(const auto& builder)
        {
            for (const auto& gate_idx : builder.memory_read_records) {
                read_record_gates.insert(gate_idx);
            }
            for (const auto& gate_idx : builder.memory_write_records) {
                write_record_gates.insert(gate_idx);
            }
        }
    };

    // Hash for lookups hash table for efficiently checking if lookups are present in set of tables used by circuit
    struct HashFunction {
        const FF mult_const = FF(uint256_t(0x1337, 0x1336, 0x1335, 0x1334));
        const FF mc_sqr = mult_const.sqr();
        const FF mc_cube = mult_const * mc_sqr;

        size_t operator()(const Key& entry) const
        {
            FF result = entry[0] + mult_const * entry[1] + mc_sqr * entry[2] + mc_cube * entry[3];
            return static_cast<size_t>(result.reduce_once().data[0]);
        }
    };
};

} // namespace bb
