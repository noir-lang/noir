#pragma once
#include <fstream>

#include "barretenberg/serialize/cbind.hpp"
#include "barretenberg/serialize/msgpack.hpp"

namespace smt_circuit_schema {

/**
 * @brief Serialized state of a circuit
 *
 * @details Used to transfer the state of the circuit
 * to Symbolic Circuit class.
 * Symbolic circuit is then used to produce SMT statements
 * that describe needed properties of the circuit.
 *
 * @param modulus Modulus of the field we are working with
 * @param public_inps Public inputs to the current circuit
 * @param vars_of_interest Map wires indices to their given names
 * @param variables List of wires values in the current circuit
 * @param selectors List of selectors in the current circuit
 * @param wires List of wires indices for each selector
 * @param real_variable_index Encoded copy constraints
 */
struct CircuitSchema {
    std::string modulus;
    std::vector<uint32_t> public_inps;
    std::unordered_map<uint32_t, std::string> vars_of_interest;
    std::vector<bb::fr> variables;
    std::vector<std::vector<bb::fr>> selectors;
    std::vector<std::vector<uint32_t>> wires;
    std::vector<uint32_t> real_variable_index;
    MSGPACK_FIELDS(modulus, public_inps, vars_of_interest, variables, selectors, wires, real_variable_index);
};

CircuitSchema unpack_from_buffer(const msgpack::sbuffer& buf);
CircuitSchema unpack_from_file(const std::string& filename);
void print_schema_for_use_in_python(CircuitSchema& cir);
} // namespace smt_circuit_schema
