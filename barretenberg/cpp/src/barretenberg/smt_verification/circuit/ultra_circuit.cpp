#include "ultra_circuit.hpp"
#include "barretenberg/common/log.hpp"

namespace smt_circuit {

/**
 * @brief Construct a new UltraCircuit object
 *
 * @param circuit_info CircuitShema object
 * @param solver pointer to the global solver
 * @param tag tag of the circuit. Empty by default.
 */
UltraCircuit::UltraCircuit(
    CircuitSchema& circuit_info, Solver* solver, TermType type, const std::string& tag, bool optimizations)
    : CircuitBase(circuit_info.vars_of_interest,
                  circuit_info.variables,
                  circuit_info.public_inps,
                  circuit_info.real_variable_index,
                  circuit_info.real_variable_tags,
                  circuit_info.range_tags,
                  solver,
                  type,
                  tag,
                  optimizations)
    , selectors(circuit_info.selectors)
    , wires_idxs(circuit_info.wires)
    , lookup_tables(circuit_info.lookup_tables)
{
    // Perform all relaxations for gates or
    // add gate in its normal state to solver

    size_t arith_cursor = 0;
    while (arith_cursor < this->selectors[1].size()) {
        arith_cursor = this->handle_arithmetic_relation(arith_cursor, 1);
    }

    size_t lookup_cursor = 0;
    while (lookup_cursor < this->selectors[5].size()) {
        lookup_cursor = this->handle_lookup_relation(lookup_cursor, 5);
    }

    size_t elliptic_cursor = 0;
    while (elliptic_cursor < this->selectors[3].size()) {
        elliptic_cursor = this->handle_elliptic_relation(elliptic_cursor, 3);
    }

    // size_t delta_range_cursor = 0;
    // while(delta_range_cursor < this->selectors[2].size()){
    //     delta_range_cursor = this->handle_delta_range_relation(delta_range_cursor, 2);
    // }
    handle_range_constraints();

    // TODO(alex): aux
}

/**
 * @brief Adds all the arithmetic gate constraints to the solver.
 * Relaxes constraint system for non-ff solver engines
 * via removing subcircuits that were already proved being correct.
 *
 * @param cusor current selector
 * @param idx arithmthetic selectors position in all selectors
 * @return new cursor value
 */
size_t UltraCircuit::handle_arithmetic_relation(size_t cursor, size_t idx)
{
    bb::fr q_m = this->selectors[idx][cursor][0];
    bb::fr q_l = this->selectors[idx][cursor][1];
    bb::fr q_r = this->selectors[idx][cursor][2];
    bb::fr q_o = this->selectors[idx][cursor][3];
    bb::fr q_4 = this->selectors[idx][cursor][4];
    bb::fr q_c = this->selectors[idx][cursor][5];
    bb::fr q_arith = this->selectors[idx][cursor][6];

    uint32_t w_l_idx = this->wires_idxs[idx][cursor][0];
    uint32_t w_r_idx = this->wires_idxs[idx][cursor][1];
    uint32_t w_o_idx = this->wires_idxs[idx][cursor][2];
    uint32_t w_4_idx = this->wires_idxs[idx][cursor][3];
    uint32_t w_l_shift_idx = this->wires_idxs[idx][cursor][4];
    uint32_t w_4_shift_idx = this->wires_idxs[idx][cursor][7];

    STerm w_l = this->symbolic_vars[w_l_idx];
    STerm w_r = this->symbolic_vars[w_r_idx];
    STerm w_o = this->symbolic_vars[w_o_idx];
    STerm w_4 = this->symbolic_vars[w_4_idx];
    STerm w_4_shift = this->symbolic_vars[w_4_shift_idx];
    STerm w_l_shift = this->symbolic_vars[w_l_shift_idx];

    std::vector<bb::fr> boolean_gate = { 1, -1, 0, 0, 0, 0, 1, 0, 0, 0, 0 };
    bool boolean_gate_flag =
        (boolean_gate == selectors[1][cursor]) && (w_l_idx == w_r_idx) && (w_o_idx == 0) && (w_4_idx == 0);
    if (boolean_gate_flag) {
        (Bool(w_l) == Bool(STerm(0, this->solver, this->type)) | Bool(w_l) == Bool(STerm(1, this->solver, this->type)))
            .assert_term();
        return cursor + 1;
    }

    STerm res = this->symbolic_vars[0];
    static const bb::fr neg_half = bb::fr(-2).invert();

    if (!q_arith.is_zero()) {
        if (q_m != 0) {
            res += ((q_arith - 3) * q_m * neg_half) * w_r * w_l;
        }
        if (q_l != 0) {
            res += (q_l * w_l);
        }
        if (q_r != 0) {
            res += (q_r * w_r);
        }
        if (q_o != 0) {
            res += (q_o * w_o);
        }
        if (q_4 != 0) {
            res += (q_4 * w_4);
        }
        if (q_c != 0) {
            res += q_c;
        }
        if (q_arith != 1) {
            res += (q_arith - 1) * w_4_shift;
        }
        // res *= q_arith;
        res == bb::fr::zero();

        optimized[w_l_idx] = false;
        optimized[w_r_idx] = false;
        optimized[w_o_idx] = false;
        optimized[w_4_idx] = false;
    }

    if (q_arith * (q_arith - 1) * (q_arith - 2) != 0) {
        res = w_l + w_4 - w_l_shift + q_m;
        res == bb::fr::zero();
        optimized[w_l_shift_idx] = false;
    }

    return cursor + 1;
}

void UltraCircuit::process_new_table(uint32_t table_idx)
{
    std::vector<std::vector<cvc5::Term>> new_table;
    bool is_xor = true;
    bool is_and = true;

    for (auto table_entry : this->lookup_tables[table_idx]) {
        std::vector<cvc5::Term> tmp_entry = {
            STerm(table_entry[0], this->solver, this->type),
            STerm(table_entry[1], this->solver, this->type),
            STerm(table_entry[2], this->solver, this->type),
        };
        new_table.push_back(tmp_entry);

        is_xor &= (static_cast<uint256_t>(table_entry[0]) ^ static_cast<uint256_t>(table_entry[1])) ==
                  static_cast<uint256_t>(table_entry[2]);
        is_and &= (static_cast<uint256_t>(table_entry[0]) & static_cast<uint256_t>(table_entry[1])) ==
                  static_cast<uint256_t>(table_entry[2]);
    }
    this->cached_symbolic_tables.insert({ table_idx, this->solver->create_lookup_table(new_table) });
    if (is_xor) {
        this->tables_types.insert({ table_idx, TableType::XOR });
        info("Encountered a XOR table");
    } else if (is_and) {
        this->tables_types.insert({ table_idx, TableType::AND });
        info("Encountered an AND table");
    } else {
        this->tables_types.insert({ table_idx, TableType::UNKNOWN });
        info("Encountered an UNKNOWN table");
    }
}

/**
 * @brief Adds all the lookup gate constraints to the solver.
 * Relaxes constraint system for non-ff solver engines
 * via removing subcircuits that were already proved being correct.
 *
 * @param cusor current selector
 * @param idx lookup selectors position in all selectors
 * @return new cursor value
 */
size_t UltraCircuit::handle_lookup_relation(size_t cursor, size_t idx)
{
    bb::fr q_m = this->selectors[idx][cursor][0];
    bb::fr q_r = this->selectors[idx][cursor][2];
    bb::fr q_o = this->selectors[idx][cursor][3];
    bb::fr q_c = this->selectors[idx][cursor][5];
    bb::fr q_lookup = this->selectors[idx][cursor][10];

    if (q_lookup.is_zero()) {
        return cursor + 1;
    }

    uint32_t w_l_idx = this->wires_idxs[idx][cursor][0];
    uint32_t w_r_idx = this->wires_idxs[idx][cursor][1];
    uint32_t w_o_idx = this->wires_idxs[idx][cursor][2];
    uint32_t w_l_shift_idx = this->wires_idxs[idx][cursor][4];
    uint32_t w_r_shift_idx = this->wires_idxs[idx][cursor][5];
    uint32_t w_o_shift_idx = this->wires_idxs[idx][cursor][6];

    optimized[w_l_idx] = false;
    optimized[w_r_idx] = false;
    optimized[w_o_idx] = false;
    optimized[w_l_shift_idx] = false;
    optimized[w_r_shift_idx] = false;
    optimized[w_o_shift_idx] = false;

    auto table_idx = static_cast<uint32_t>(q_o);
    if (!this->cached_symbolic_tables.contains(table_idx)) {
        this->process_new_table(table_idx);
    }

    STerm first_entry = this->symbolic_vars[w_l_idx] + q_r * this->symbolic_vars[w_l_shift_idx];
    STerm second_entry = this->symbolic_vars[w_r_idx] + q_m * this->symbolic_vars[w_r_shift_idx];
    STerm third_entry = this->symbolic_vars[w_o_idx] + q_c * this->symbolic_vars[w_o_shift_idx];
    std::vector<STerm> entries = { first_entry, second_entry, third_entry };

    if (this->type == TermType::BVTerm && this->enable_optimizations) {
        // Sort of an optimization.
        // However if we don't do this, solver will find a unique witness that corresponds to overflowed value.
        if (q_r == -64 && q_m == -64 && q_c == -64) {
            this->symbolic_vars[w_l_shift_idx] = this->symbolic_vars[w_l_idx] >> 6;
            this->symbolic_vars[w_r_shift_idx] = this->symbolic_vars[w_r_idx] >> 6;
            this->symbolic_vars[w_o_shift_idx] = this->symbolic_vars[w_o_idx] >> 6;
        }
        switch (this->tables_types[table_idx]) {
        case TableType::XOR:
            info("XOR optimization");
            (first_entry ^ second_entry) == third_entry;
            return cursor + 1;
        case TableType::AND:
            info("AND optimization");
            (first_entry & second_entry) == third_entry;
            return cursor + 1;
        case TableType::UNKNOWN:
            break;
        }
    }
    info("Unknown Table");
    STerm::in_table(entries, this->cached_symbolic_tables[table_idx]);
    return cursor + 1;
}

/**
 * @brief Adds all the elliptic gate constraints to the solver.
 *
 * @param cusor current selector
 * @param idx elliptic selectors position in all selectors
 * @return new cursor value
 */
size_t UltraCircuit::handle_elliptic_relation(size_t cursor, size_t idx)
{
    bb::fr q_is_double = this->selectors[idx][cursor][0];
    bb::fr q_sign = this->selectors[idx][cursor][1];
    bb::fr q_elliptic = this->selectors[idx][cursor][8];
    if (q_elliptic.is_zero()) {
        return cursor + 1;
    }

    uint32_t w_r_idx = this->wires_idxs[idx][cursor][1];
    uint32_t w_o_idx = this->wires_idxs[idx][cursor][2];
    uint32_t w_l_shift_idx = this->wires_idxs[idx][cursor][4];
    uint32_t w_r_shift_idx = this->wires_idxs[idx][cursor][5];
    uint32_t w_o_shift_idx = this->wires_idxs[idx][cursor][6];
    uint32_t w_4_shift_idx = this->wires_idxs[idx][cursor][7];
    optimized[w_r_idx] = false;
    optimized[w_o_idx] = false;
    optimized[w_l_shift_idx] = false;
    optimized[w_r_shift_idx] = false;
    optimized[w_o_shift_idx] = false;
    optimized[w_4_shift_idx] = false;

    STerm x_1 = this->symbolic_vars[w_r_idx];
    STerm y_1 = this->symbolic_vars[w_o_idx];
    STerm x_2 = this->symbolic_vars[w_l_shift_idx];
    STerm y_2 = this->symbolic_vars[w_4_shift_idx];
    STerm x_3 = this->symbolic_vars[w_r_shift_idx];
    STerm y_3 = this->symbolic_vars[w_o_shift_idx];

    auto x_diff = (x_2 - x_1);
    auto y2_sqr = (y_2 * y_2);
    auto y1_sqr = (y_1 * y_1);
    auto y1y2 = y_1 * y_2 * q_sign;
    auto x_add_identity = (x_3 + x_2 + x_1) * x_diff * x_diff - y2_sqr - y1_sqr + y1y2 + y1y2;

    auto y1_plus_y3 = y_1 + y_3;
    auto y_diff = y_2 * q_sign - y_1;
    auto y_add_identity = y1_plus_y3 * x_diff + (x_3 - x_1) * y_diff;

    if (q_is_double.is_zero()) {
        x_add_identity == 0; // scaling_factor = 1
        y_add_identity == 0; // scaling_factor = 1
    }

    bb::fr curve_b = this->selectors[3][cursor][11];
    auto x_pow_4 = (y1_sqr - curve_b) * x_1;
    auto y1_sqr_mul_4 = y1_sqr + y1_sqr;
    y1_sqr_mul_4 += y1_sqr_mul_4;
    auto x1_pow_4_mul_9 = x_pow_4 * 9;
    auto x_double_identity = (x_3 + x_1 + x_1) * y1_sqr_mul_4 - x1_pow_4_mul_9;

    auto x1_sqr_mul_3 = (x_1 + x_1 + x_1) * x_1;
    auto y_double_identity = x1_sqr_mul_3 * (x_1 - x_3) - (y_1 + y_1) * (y_1 + y_3);

    if (!q_is_double.is_zero()) {
        x_double_identity == 0; // scaling_factor = 1
        y_double_identity == 0; // scaling_factor = 1
    }

    return cursor + 1;
}

/**
 * @brief Adds all the delta_range gate constraints to the solver.
 *
 * @param cusor current selector
 * @param idx delta_range selectors position in all selectors
 * @return new cursor value
 * @todo Useless?
 */
size_t UltraCircuit::handle_delta_range_relation(size_t cursor, size_t idx)
{
    bb::fr q_delta_range = this->selectors[idx][cursor][7];
    if (q_delta_range == 0) {
        return cursor + 1;
    }

    uint32_t w_l_idx = this->wires_idxs[idx][cursor][0];
    uint32_t w_r_idx = this->wires_idxs[idx][cursor][1];
    uint32_t w_o_idx = this->wires_idxs[idx][cursor][2];
    uint32_t w_4_idx = this->wires_idxs[idx][cursor][3];
    uint32_t w_l_shift_idx = this->wires_idxs[idx][cursor][4];

    STerm w_1 = this->symbolic_vars[w_l_idx];
    STerm w_2 = this->symbolic_vars[w_r_idx];
    STerm w_3 = this->symbolic_vars[w_o_idx];
    STerm w_4 = this->symbolic_vars[w_4_idx];
    STerm w_1_shift = this->symbolic_vars[w_l_shift_idx];

    STerm delta_1 = w_2 - w_1;
    STerm delta_2 = w_3 - w_2;
    STerm delta_3 = w_4 - w_3;
    STerm delta_4 = w_1_shift - w_4;

    STerm tmp = (delta_1 - 1) * (delta_1 - 1) - 1;
    tmp *= (delta_1 - 2) * (delta_1 - 2) - 1;
    tmp == 0;

    tmp = (delta_2 - 1) * (delta_2 - 1) - 1;
    tmp *= (delta_2 - 2) * (delta_2 - 2) - 1;
    tmp == 0;

    tmp = (delta_3 - 1) * (delta_3 - 1) - 1;
    tmp *= (delta_3 - 2) * (delta_3 - 2) - 1;
    tmp == 0;

    tmp = (delta_4 - 1) * (delta_4 - 1) - 1;
    tmp *= (delta_4 - 2) * (delta_4 - 2) - 1;
    tmp == 0;

    return cursor + 1;
}

/**
 * @brief Adds all the range constraints to the solver.
 *
 * @param cusor current selector
 * @param idx delta_range selectors position in all selectors
 * @return new cursor value
 */
void UltraCircuit::handle_range_constraints()
{
    for (uint32_t i = 0; i < this->get_num_vars(); i++) {
        if (i != this->real_variable_index[i] || optimized[i]) {
            continue;
        }

        uint32_t tag = this->real_variable_tags[this->real_variable_index[i]];
        if (tag != 0 && this->range_tags.contains(tag)) {
            uint64_t range = this->range_tags[tag];
            if (this->type == TermType::FFTerm || !this->enable_optimizations) {
                if (!this->cached_range_tables.contains(range)) {
                    std::vector<cvc5::Term> new_range_table;
                    for (size_t entry = 0; entry < range; entry++) {
                        new_range_table.push_back(STerm(entry, this->solver, this->type));
                    }
                    this->cached_range_tables.insert({ range, this->solver->create_table(new_range_table) });
                }

                this->symbolic_vars[i].in(this->cached_range_tables[range]);
            } else {
                this->symbolic_vars[i] <= range;
            }
            optimized[i] = false;
        }
    }
}
/**
 * @brief Similar functionality to old .check_circuit() method
 * in standard circuit builder.
 *
 * @param witness
 * @return true
 * @return false
 *
 * @todo Do we actually need this here?
 */
bool UltraCircuit::simulate_circuit_eval(std::vector<bb::fr>& witness) const
{
    if (witness.size() != this->get_num_vars()) {
        throw std::invalid_argument("Witness size should be " + std::to_string(this->get_num_vars()) +

                                    std::to_string(witness.size()));
    }
    return true;
}

/**
 * @brief Check your circuit for witness uniqueness
 *
 * @details Creates two Circuit objects that represent the same
 * circuit, however you can choose which variables should be (not) equal in both cases,
 * and also the variables that should (not) be equal at the same time.
 *
 * @param circuit_info
 * @param s pointer to the global solver
 * @param equal The list of names of variables which should be equal in both circuits(each is equal)
 * @param not_equal The list of names of variables which should not be equal in both circuits(each is not equal)
 * @param equal_at_the_same_time The list of variables, where at least one pair has to be equal
 * @param not_equal_at_the_same_time The list of variables, where at least one pair has to be distinct
 * @return std::pair<Circuit, Circuit>
 */
std::pair<UltraCircuit, UltraCircuit> UltraCircuit::unique_witness_ext(
    CircuitSchema& circuit_info,
    Solver* s,
    TermType type,
    const std::vector<std::string>& equal,
    const std::vector<std::string>& not_equal,
    const std::vector<std::string>& equal_at_the_same_time,
    const std::vector<std::string>& not_equal_at_the_same_time,
    bool enable_optimizations)
{
    UltraCircuit c1(circuit_info, s, type, "circuit1", enable_optimizations);
    UltraCircuit c2(circuit_info, s, type, "circuit2", enable_optimizations);

    for (const auto& term : equal) {
        c1[term] == c2[term];
    }
    for (const auto& term : not_equal) {
        c1[term] != c2[term];
    }

    std::vector<Bool> eqs;
    for (const auto& term : equal_at_the_same_time) {
        Bool tmp = Bool(c1[term]) == Bool(c2[term]);
        eqs.push_back(tmp);
    }

    if (eqs.size() > 1) {
        batch_or(eqs).assert_term();
    } else if (eqs.size() == 1) {
        eqs[0].assert_term();
    }

    std::vector<Bool> neqs;
    for (const auto& term : not_equal_at_the_same_time) {
        Bool tmp = Bool(c1[term]) != Bool(c2[term]);
        neqs.push_back(tmp);
    }

    if (neqs.size() > 1) {
        batch_or(neqs).assert_term();
    } else if (neqs.size() == 1) {
        neqs[0].assert_term();
    }
    return { c1, c2 };
}

/**
 * @brief Check your circuit for witness uniqueness
 *
 * @details Creates two Circuit objects that represent the same
 * circuit, however you can choose which variables should be equal in both cases,
 * other witness members will be marked as not equal at the same time
 * or basically they will have to differ by at least one element.
 *
 * @param circuit_info
 * @param s pointer to the global solver
 * @param equal The list of names of variables which should be equal in both circuits(each is equal)
 * @return std::pair<Circuit, Circuit>
 */
std::pair<UltraCircuit, UltraCircuit> UltraCircuit::unique_witness(CircuitSchema& circuit_info,
                                                                   Solver* s,
                                                                   TermType type,
                                                                   const std::vector<std::string>& equal,
                                                                   bool enable_optimizations)
{
    UltraCircuit c1(circuit_info, s, type, "circuit1", enable_optimizations);
    UltraCircuit c2(circuit_info, s, type, "circuit2", enable_optimizations);

    for (const auto& term : equal) {
        c1[term] == c2[term];
    }

    std::vector<Bool> neqs;
    for (const auto& node : c1.symbolic_vars) {
        uint32_t i = node.first;
        if (std::find(equal.begin(), equal.end(), std::string(c1.variable_names[i])) != equal.end()) {
            continue;
        }
        if (c1.optimized[i]) {
            continue;
        }
        Bool tmp = Bool(c1[i]) != Bool(c2[i]);
        neqs.push_back(tmp);
    }

    if (neqs.size() > 1) {
        batch_or(neqs).assert_term();
    } else if (neqs.size() == 1) {
        neqs[0].assert_term();
    }
    return { c1, c2 };
}
}; // namespace smt_circuit