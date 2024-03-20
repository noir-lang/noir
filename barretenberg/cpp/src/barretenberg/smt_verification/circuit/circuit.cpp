#include "circuit.hpp"

namespace smt_circuit {

/**
 * @brief Construct a new Circuit::Circuit object
 *
 * @param circuit_info CircuitShema object
 * @param solver pointer to the global solver
 * @param tag tag of the circuit. Empty by default.
 */
Circuit::Circuit(CircuitSchema& circuit_info, Solver* solver, TermType type, const std::string& tag, bool optimizations)
    : variables(circuit_info.variables)
    , public_inps(circuit_info.public_inps)
    , variable_names(circuit_info.vars_of_interest)
    , selectors(circuit_info.selectors)
    , wires_idxs(circuit_info.wires)
    , real_variable_index(circuit_info.real_variable_index)
    , optimizations(optimizations)
    , solver(solver)
    , type(type)
    , tag(tag)
{
    if (!this->tag.empty()) {
        if (this->tag[0] != '_') {
            this->tag = "_" + this->tag;
        }
    }

    for (auto& x : variable_names) {
        variable_names_inverse.insert({ x.second, x.first });
    }

    variable_names.insert({ 0, "zero" });
    variable_names.insert({ 1, "one" });
    variable_names_inverse.insert({ "zero", 0 });
    variable_names_inverse.insert({ "one", 1 });
    optimized.insert({ 0, false });
    optimized.insert({ 1, false });

    this->init();

    // Perform all relaxation for gates or
    // add gate in its normal state to solver
    size_t i = 0;
    while (i < this->get_num_gates()) {
        i = this->prepare_gates(i);
    }

    for (const auto& i : this->public_inps) {
        this->symbolic_vars[this->real_variable_index[i]] == this->variables[i];
    }
}

/**
 * Creates all the needed symbolic variables and constants
 * which are used in circuit.
 *
 */
void Circuit::init()
{
    size_t num_vars = variables.size();
    symbolic_vars.insert({ 0, STerm::Var("zero" + this->tag, this->solver, this->type) });
    symbolic_vars.insert({ 1, STerm::Var("one" + this->tag, this->solver, this->type) });

    for (uint32_t i = 2; i < num_vars; i++) {
        uint32_t real_idx = this->real_variable_index[i];
        if (this->symbolic_vars.contains(real_idx)) {
            continue;
        }

        std::string name = variable_names.contains(real_idx) ? variable_names[real_idx] : "var_" + std::to_string(i);
        name += this->tag;
        symbolic_vars.insert({ real_idx, STerm::Var(name, this->solver, this->type) });

        optimized.insert({ real_idx, true });
    }

    symbolic_vars[0] == bb::fr(0);
    symbolic_vars[1] == bb::fr(1);
}

/**
 * @brief Relaxes univariate polynomial constraints.
 * TODO(alex): probably won't be necessary in the nearest future
 * because of new solver
 *
 * @param q_m multiplication selector
 * @param q_1 l selector
 * @param q_2 r selector
 * @param q_3 o selector
 * @param q_c constant
 * @param w   witness index
 */
void Circuit::handle_univariate_constraint(bb::fr q_m, bb::fr q_1, bb::fr q_2, bb::fr q_3, bb::fr q_c, uint32_t w)
{
    bb::fr b = q_1 + q_2 + q_3;

    if (q_m == 0) {
        symbolic_vars[w] == -q_c / b;
        return;
    }

    std::pair<bool, bb::fr> d = (b * b - bb::fr(4) * q_m * q_c).sqrt();
    if (!d.first) {
        throw std::invalid_argument("There're no roots of quadratic polynomial");
    }
    bb::fr x1 = (-b + d.second) / (bb::fr(2) * q_m);
    bb::fr x2 = (-b - d.second) / (bb::fr(2) * q_m);

    if (d.second == 0) {
        symbolic_vars[w] == STerm(x1, this->solver, type);
    } else {
        ((Bool(symbolic_vars[w]) == Bool(STerm(x1, this->solver, this->type))) |
         (Bool(symbolic_vars[w]) == Bool(STerm(x2, this->solver, this->type))))
            .assert_term();
    }
}

/**
 * @brief Relaxes logic constraints(AND/XOR).
 * @details This function is needed when we use bitwise compatible
 * symbolic terms.
 * It compares the chunk of selectors of the current circuit
 * with pure create_logic_constraint from circuit_builder.
 * It uses binary search to find a bit length of the constraint,
 * since we don't know it in general.
 * After a match is found, it updates the cursor to skip all the
 * redundant constraints and adds a pure a ^ b = c or a & b = c
 * constraint to solver.
 * If there's no match, it will return -1
 *
 * @param cursor current position
 * @return next position or -1
 */
size_t Circuit::handle_logic_constraint(size_t cursor)
{
    // Initialize binary search. Logic gate can only accept even bit lengths
    // So we need to find a match among [1, 127] and then multiply the result by 2
    size_t beg = 1;
    size_t end = 127;
    size_t mid = 0;
    auto res = static_cast<size_t>(-1);

    // Indicates that current bit length is a match for XOR
    bool xor_flag = true;
    // Indicates that current bit length is a match for AND
    bool and_flag = true;
    // Indicates the logic operation(true - XOR, false - AND) if the match is found.
    bool logic_flag = true;
    CircuitProps xor_props;
    CircuitProps and_props;

    bool stop_flag = false;

    while (beg <= end) {
        mid = (end + beg) / 2;

        // Take a pure logic circuit for the current bit length(2 * mid)
        // and compare it's selectors to selectors of the global circuit
        // at current position(cursor).
        // If they are equal, we can apply an optimization
        // However, if we have a match at bit length 2, it is possible
        // to have a match at higher bit lengths. That's why we store
        // the current match as `res` and proceed with ordinary binary search.
        // `stop_flag` simply indicates that the first selector doesn't match
        // and we can skip this whole section.

        if (!this->cached_subcircuits[SubcircuitType::XOR].contains(mid * 2)) {
            this->cached_subcircuits[SubcircuitType::XOR].insert(
                { mid * 2, get_standard_logic_circuit(mid * 2, true) });
        }
        xor_props = this->cached_subcircuits[SubcircuitType::XOR][mid * 2];

        if (!this->cached_subcircuits[SubcircuitType::AND].contains(mid * 2)) {
            this->cached_subcircuits[SubcircuitType::AND].insert(
                { mid * 2, get_standard_logic_circuit(mid * 2, false) });
        }
        and_props = this->cached_subcircuits[SubcircuitType::AND][mid * 2];

        CircuitSchema xor_circuit = xor_props.circuit;
        CircuitSchema and_circuit = and_props.circuit;

        xor_flag = cursor + xor_props.num_gates <= this->selectors.size();
        and_flag = cursor + xor_props.num_gates <= this->selectors.size();
        if (xor_flag || and_flag) {
            for (size_t j = 0; j < xor_props.num_gates; j++) {
                // It is possible for gates to be equal but wires to be not, but I think it's very
                // unlikely to happen
                xor_flag &= xor_circuit.selectors[j + xor_props.start_gate] == this->selectors[cursor + j];
                and_flag &= and_circuit.selectors[j + and_props.start_gate] == this->selectors[cursor + j];

                if (!xor_flag && !and_flag) {
                    // Won't match at any bit length
                    if (j == 0) {
                        stop_flag = true;
                    }
                    break;
                }
            }
        }
        if (stop_flag) {
            break;
        }

        if (!xor_flag && !and_flag) {
            end = mid - 1;
        } else {
            res = 2 * mid;
            logic_flag = xor_flag;

            beg = mid + 1;
        }
    }

    // TODO(alex): Figure out if I need to create range constraint here too or it'll be
    // created anyway in any circuit
    if (res != static_cast<size_t>(-1)) {
        xor_props = get_standard_logic_circuit(res, true);
        and_props = get_standard_logic_circuit(res, false);

        info("Logic constraint optimization: ", std::to_string(res), " bits. is_xor: ", logic_flag);
        size_t left_gate = xor_props.gate_idxs[0];
        uint32_t left_gate_idx = xor_props.idxs[0];
        size_t right_gate = xor_props.gate_idxs[1];
        uint32_t right_gate_idx = xor_props.idxs[1];
        size_t out_gate = xor_props.gate_idxs[2];
        uint32_t out_gate_idx = xor_props.idxs[2];

        uint32_t left_idx = this->real_variable_index[this->wires_idxs[cursor + left_gate][left_gate_idx]];
        uint32_t right_idx = this->real_variable_index[this->wires_idxs[cursor + right_gate][right_gate_idx]];
        uint32_t out_idx = this->real_variable_index[this->wires_idxs[cursor + out_gate][out_gate_idx]];

        STerm left = this->symbolic_vars[left_idx];
        STerm right = this->symbolic_vars[right_idx];
        STerm out = this->symbolic_vars[out_idx];

        if (logic_flag) {
            (left ^ right) == out;
        } else {
            (left & right) == out;
        }

        // You have to mark these arguments so they won't be optimized out
        optimized[left_idx] = false;
        optimized[right_idx] = false;
        optimized[out_idx] = false;
        return cursor + xor_props.num_gates;
    }
    return res;
}

/**
 * @brief Relaxes range constraints.
 * @details This function is needed when we use range compatible
 * symbolic terms.
 * It compares the chunk of selectors of the current circuit
 * with pure create_range_constraint from circuit_builder.
 * It uses binary search to find a bit length of the constraint,
 * since we don't know it in general.
 * After match is found, it updates the cursor to skip all the
 * redundant constraints and adds a pure a < 2^bit_length
 * constraint to solver.
 * If there's no match, it will return -1
 *
 * @param cursor current position
 * @return next position or -1
 */
size_t Circuit::handle_range_constraint(size_t cursor)
{
    // Indicates that current bit length is a match
    bool range_flag = true;
    size_t mid = 0;
    auto res = static_cast<size_t>(-1);

    CircuitProps range_props;
    // Range constraints differ depending on oddness of bit_length
    // That's why we need to handle these cases separately
    for (size_t odd = 0; odd < 2; odd++) {
        // Initialize binary search.
        // We need to find a match among [1, 127] and then set the result to 2 * mid, or 2 * mid + 1
        size_t beg = 1;
        size_t end = 127;

        bool stop_flag = false;
        while (beg <= end) {
            mid = (end + beg) / 2;

            // Take a pure logic circuit for the current bit length(2 * mid + odd)
            // and compare it's selectors to selectors of the global circuit
            // at current positin(cursor).
            // If they are equal, we can apply an optimization
            // However, if we have a match at bit length 2, it is possible
            // to have a match at higher bit lengths. That's why we store
            // the current match as `res` and proceed with ordinary binary search.
            // `stop_flag` simply indicates that the first selector doesn't match
            // and we can skip this whole section.

            if (!this->cached_subcircuits[SubcircuitType::RANGE].contains(2 * mid + odd)) {
                this->cached_subcircuits[SubcircuitType::RANGE].insert(
                    { 2 * mid + odd, get_standard_range_constraint_circuit(2 * mid + odd) });
            }
            range_props = this->cached_subcircuits[SubcircuitType::RANGE][2 * mid + odd];
            CircuitSchema range_circuit = range_props.circuit;

            range_flag = cursor + range_props.num_gates <= this->get_num_gates();
            if (range_flag) {
                for (size_t j = 0; j < range_props.num_gates; j++) {
                    // It is possible for gates to be equal but wires to be not, but I think it's very
                    // unlikely to happen
                    range_flag &= range_circuit.selectors[j + range_props.start_gate] == this->selectors[cursor + j];

                    if (!range_flag) {
                        // Won't match at any bit length
                        if (j <= 2) {
                            stop_flag = true;
                        }
                        break;
                    }
                }
            }
            if (stop_flag) {
                break;
            }

            if (!range_flag) {
                end = mid - 1;
            } else {
                res = 2 * mid + odd;
                beg = mid + 1;
            }
        }

        if (res != static_cast<size_t>(-1)) {
            range_flag = true;
            break;
        }
    }

    if (range_flag) {
        info("Range constraint optimization: ", std::to_string(res), " bits");
        range_props = get_standard_range_constraint_circuit(res);

        size_t left_gate = range_props.gate_idxs[0];
        uint32_t left_gate_idx = range_props.idxs[0];
        uint32_t left_idx = this->real_variable_index[this->wires_idxs[cursor + left_gate][left_gate_idx]];

        STerm left = this->symbolic_vars[left_idx];
        left <= (bb::fr(2).pow(res) - 1);

        // You have to mark these arguments so they won't be optimized out
        optimized[left_idx] = false;
        return cursor + range_props.num_gates;
    }
    return res;
}

/**
 * @brief Adds all the gate constraints to the solver.
 * Relaxes constraint system for non-ff solver engines
 * via removing subcircuits that were already proved being correct.
 *
 */
size_t Circuit::prepare_gates(size_t cursor)
{
    // TODO(alex): implement bitvector class and compute offsets
    if (this->type == TermType::BVTerm && this->optimizations) {
        size_t res = handle_logic_constraint(cursor);
        if (res != static_cast<size_t>(-1)) {
            return res;
        }
    }

    if ((this->type == TermType::BVTerm || this->type == TermType::FFITerm) && this->optimizations) {
        size_t res = handle_range_constraint(cursor);
        if (res != static_cast<size_t>(-1)) {
            return res;
        }
    }

    bb::fr q_m = this->selectors[cursor][0];
    bb::fr q_1 = this->selectors[cursor][1];
    bb::fr q_2 = this->selectors[cursor][2];
    bb::fr q_3 = this->selectors[cursor][3];
    bb::fr q_c = this->selectors[cursor][4];

    uint32_t w_l = this->wires_idxs[cursor][0];
    uint32_t w_r = this->wires_idxs[cursor][1];
    uint32_t w_o = this->wires_idxs[cursor][2];
    optimized[w_l] = false;
    optimized[w_r] = false;
    optimized[w_o] = false;

    // Handles the case when we have univariate polynomial as constraint
    // by simply finding the roots via quadratic formula(or linear)
    // There're 7 possibilities of that, which are present below
    bool univariate_flag = false;
    univariate_flag |= (w_l == w_r) && (w_r == w_o);
    univariate_flag |= (w_l == w_r) && (q_3 == 0);
    univariate_flag |= (w_l == w_o) && (q_2 == 0) && (q_m == 0);
    univariate_flag |= (w_r == w_o) && (q_1 == 0) && (q_m == 0);
    univariate_flag |= (q_m == 0) && (q_1 == 0) && (q_3 == 0);
    univariate_flag |= (q_m == 0) && (q_2 == 0) && (q_3 == 0);
    univariate_flag |= (q_m == 0) && (q_1 == 0) && (q_2 == 0);

    // Univariate gate. Relaxes the solver. Or is it?
    // TODO(alex): Test the effect of this relaxation after the tests are merged.
    if (univariate_flag) {
        if ((q_m == 1) && (q_1 == 0) && (q_2 == 0) && (q_3 == -1) && (q_c == 0)) {
            (Bool(symbolic_vars[w_l]) == Bool(symbolic_vars[0]) | Bool(symbolic_vars[w_l]) == Bool(symbolic_vars[1]))
                .assert_term();
        } else {
            this->handle_univariate_constraint(q_m, q_1, q_2, q_3, q_c, w_l);
        }
    } else {
        STerm eq = symbolic_vars[0];

        // mul selector
        if (q_m != 0) {
            eq += symbolic_vars[w_l] * symbolic_vars[w_r] * q_m;
        }
        // left selector
        if (q_1 != 0) {
            eq += symbolic_vars[w_l] * q_1;
        }
        // right selector
        if (q_2 != 0) {
            eq += symbolic_vars[w_r] * q_2;
        }
        // out selector
        if (q_3 != 0) {
            eq += symbolic_vars[w_o] * q_3;
        }
        // constant selector
        if (q_c != 0) {
            eq += q_c;
        }
        eq == symbolic_vars[0];
    }
    return cursor + 1;
}

/**
 * @brief Returns a previously named symbolic variable.
 *
 * @param name
 * @return STerm
 */
STerm Circuit::operator[](const std::string& name)
{
    if (!this->variable_names_inverse.contains(name)) {
        throw std::invalid_argument("No such an item `" + name + "` in vars or it vas not declared as interesting");
    }
    uint32_t idx = this->variable_names_inverse[name];
    return this->symbolic_vars[idx];
}

/**
 * @brief Similar functionality to old .check_circuit() method
 * in standard circuit builder.
 *
 * @param witness
 * @return true
 * @return false
 */
bool Circuit::simulate_circuit_eval(std::vector<bb::fr>& witness) const
{
    if (witness.size() != this->get_num_vars()) {
        throw std::invalid_argument("Witness size should be " + std::to_string(this->get_num_vars()) + ", not " +
                                    std::to_string(witness.size()));
    }
    for (size_t i = 0; i < this->selectors.size(); i++) {
        bb::fr res = 0;
        bb::fr x = witness[this->wires_idxs[i][0]];
        bb::fr y = witness[this->wires_idxs[i][1]];
        bb::fr o = witness[this->wires_idxs[i][2]];
        res += this->selectors[i][0] * x * y;
        res += this->selectors[i][1] * x;
        res += this->selectors[i][2] * y;
        res += this->selectors[i][3] * o;
        res += this->selectors[i][4];
        if (res != 0) {
            return false;
        }
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
std::pair<Circuit, Circuit> unique_witness_ext(CircuitSchema& circuit_info,
                                               Solver* s,
                                               TermType type,
                                               const std::vector<std::string>& equal,
                                               const std::vector<std::string>& not_equal,
                                               const std::vector<std::string>& equal_at_the_same_time,
                                               const std::vector<std::string>& not_equal_at_the_same_time)
{
    // TODO(alex): set optimizations to be true once they are confirmed
    Circuit c1(circuit_info, s, type, "circuit1", false);
    Circuit c2(circuit_info, s, type, "circuit2", false);

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
std::pair<Circuit, Circuit> unique_witness(CircuitSchema& circuit_info,
                                           Solver* s,
                                           TermType type,
                                           const std::vector<std::string>& equal)
{
    // TODO(alex): set optimizations to be true once they are confirmed
    Circuit c1(circuit_info, s, type, "circuit1", false);
    Circuit c2(circuit_info, s, type, "circuit2", false);

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