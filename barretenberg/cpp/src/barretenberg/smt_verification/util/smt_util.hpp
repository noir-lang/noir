#pragma once
#include <fstream>

#include "barretenberg/smt_verification/circuit/circuit_base.hpp"

#define RED "\033[31m"
#define RESET "\033[0m"

void default_model(const std::vector<std::string>& special,
                   smt_circuit::CircuitBase& c1,
                   smt_circuit::CircuitBase& c2,
                   const std::string& fname = "witness.out",
                   bool pack = true);
void default_model_single(const std::vector<std::string>& special,
                          smt_circuit::CircuitBase& c,
                          const std::string& fname = "witness.out",
                          bool pack = true);

bool smt_timer(smt_solver::Solver* s);
std::pair<std::vector<bb::fr>, std::vector<bb::fr>> base4(uint32_t el);
void fix_range_lists(bb::UltraCircuitBuilder& builder);
bb::fr string_to_fr(const std::string& number, int base, size_t step = 0);
std::vector<std::vector<bb::fr>> import_witness(const std::string& fname);
std::vector<bb::fr> import_witness_single(const std::string& fname);