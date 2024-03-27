#pragma once
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_circuit_builder.hpp"

namespace bb {

class MockCircuits {
  public:
    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using Point = Curve::AffineElement;

    /**
     * @brief Populate a builder with a specified number of arithmetic gates; includes a PI
     *
     * @param builder
     * @param num_gates
     */
    template <typename Builder>
    static void construct_arithmetic_circuit(Builder& builder, const size_t target_log2_dyadic_size = 4)
    {
        const size_t target_dyadic_size = 1 << target_log2_dyadic_size;
        const size_t num_preamble_gates = builder.num_gates;
        ASSERT(target_dyadic_size >= num_preamble_gates);

        // For good measure, include a gate with some public inputs
        if (target_dyadic_size > num_preamble_gates) {
            FF a = FF::random_element();
            FF b = FF::random_element();
            FF c = FF::random_element();
            FF d = a + b + c;
            uint32_t a_idx = builder.add_public_variable(a);
            uint32_t b_idx = builder.add_variable(b);
            uint32_t c_idx = builder.add_variable(c);
            uint32_t d_idx = builder.add_variable(d);

            builder.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, FF(1), FF(1), FF(1), FF(-1), FF(0) });
        }

        // A proper treatment of this would dynamically calculate how many gates to add given static information about
        // Builder, but a major overhaul of the execution trace is underway, so we just elect to use a hack. Namely, for
        // all of builders for which we instantiate this template and for all circuit sizes we care about, to achieve a
        // desired dyadic circuit size after boilerplate gates, it is sufficient to fill up to OFFSET_HACK-many gates
        // short of the desired dyadic circuit size.
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/902)
        static constexpr size_t OFFSET_HACK = 10;

        // to prevent underflow of the loop upper limit; target size >= 16 should suffice
        ASSERT(target_dyadic_size > OFFSET_HACK + num_preamble_gates);
        // Add arbitrary arithmetic gates to obtain a total of num_gates-many gates
        FF a = FF::random_element();
        FF b = FF::random_element();
        FF c = FF::random_element();
        FF d = a + b + c;
        uint32_t a_idx = builder.add_variable(a);
        uint32_t b_idx = builder.add_variable(b);
        uint32_t c_idx = builder.add_variable(c);
        uint32_t d_idx = builder.add_variable(d);

        for (size_t i = 0; i < target_dyadic_size - OFFSET_HACK - 1 - num_preamble_gates; ++i) {
            builder.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, FF(1), FF(1), FF(1), FF(-1), FF(0) });
        }
    }

    /**
     * @brief Populate a builder with some arbitrary goblinized ECC ops, one of each type
     *
     * @param builder
     */
    static void construct_goblin_ecc_op_circuit(GoblinUltraCircuitBuilder& builder)
    {
        // Add a mul accum op, an add accum op and an equality op
        builder.queue_ecc_add_accum(Point::one() * FF::random_element());
        builder.queue_ecc_mul_accum(Point::one() * FF::random_element(), FF::random_element());
        builder.queue_ecc_eq();
    }
};
} // namespace bb