#pragma once
#include <fstream>

#include "barretenberg/smt_verification/circuit/circuit_base.hpp"

#define RED "\033[31m"
#define RESET "\033[0m"

void default_model(const std::vector<std::string>& special,
                   smt_circuit::CircuitBase& c1,
                   smt_circuit::CircuitBase& c2,
                   const std::string& fname = "witness.out");
void default_model_single(const std::vector<std::string>& special,
                          smt_circuit::CircuitBase& c,
                          const std::string& fname = "witness.out");

bool smt_timer(smt_solver::Solver* s);
std::pair<std::vector<bb::fr>, std::vector<bb::fr>> base4(uint32_t el);
void fix_range_lists(bb::UltraCircuitBuilder& builder);