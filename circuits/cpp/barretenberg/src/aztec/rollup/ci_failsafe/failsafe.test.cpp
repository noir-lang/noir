#include "../constants.hpp"
#include <gtest/gtest.h>

/**
 * @brief This test detects if the circuit change expected constant is disabled. It is used so that developers can
 * safely change stuff in circuits and run tests in PRs, but there is one last failsafe that doesn't allow them to merge
 * it.
 *
 */
TEST(ci_failsafe, detect_circuit_change_disabled)
{
    EXPECT_EQ(rollup::circuit_gate_count::is_circuit_change_expected, 0);
}