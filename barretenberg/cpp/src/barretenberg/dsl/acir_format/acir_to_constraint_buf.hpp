#pragma once
#include "acir_format.hpp"
#include "serde/index.hpp"

namespace acir_format {

AcirFormat circuit_buf_to_acir_format(std::vector<uint8_t> const& buf, bool honk_recursion);

/**
 * @brief Converts from the ACIR-native `WitnessMap` format to Barretenberg's internal `WitnessVector` format.
 *
 * @param buf Serialized representation of a `WitnessMap`.
 * @return A `WitnessVector` equivalent to the passed `WitnessMap`.
 * @note This transformation results in all unassigned witnesses within the `WitnessMap` being assigned the value 0.
 *       Converting the `WitnessVector` back to a `WitnessMap` is unlikely to return the exact same `WitnessMap`.
 */
WitnessVector witness_buf_to_witness_data(std::vector<uint8_t> const& buf);

std::vector<AcirFormat> program_buf_to_acir_format(std::vector<uint8_t> const& buf, bool honk_recursion);

WitnessVectorStack witness_buf_to_witness_stack(std::vector<uint8_t> const& buf);

#ifndef __wasm__
AcirProgramStack get_acir_program_stack(std::string const& bytecode_path,
                                        std::string const& witness_path,
                                        bool honk_recursion);
#endif
} // namespace acir_format