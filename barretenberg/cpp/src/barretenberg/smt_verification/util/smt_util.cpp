#include "smt_util.hpp"

/**
 * @brief Get pretty formatted result of the solver work
 *
 * @details Having two circuits and defined constraint system
 * inside the solver get the pretty formatted output.
 * The whole witness will be saved in c-like array format.
 * Special variables will be printed to stdout. e.g. `a_1, a_2 = val_a_1, val_a_2;`
 *
 * @param special The list of variables that you need to see in stdout
 * @param c1 first circuit
 * @param c2 the copy of the first circuit with changed tag
 * @param s  solver
 * @param fname file to store the resulting witness if succeded
 */
void default_model(const std::vector<std::string>& special,
                   smt_circuit::CircuitBase& c1,
                   smt_circuit::CircuitBase& c2,
                   const std::string& fname)
{
    std::vector<cvc5::Term> vterms1;
    std::vector<cvc5::Term> vterms2;
    vterms1.reserve(c1.get_num_vars());
    vterms2.reserve(c1.get_num_vars());

    for (uint32_t i = 0; i < c1.get_num_vars(); i++) {
        vterms1.push_back(c1.symbolic_vars[c1.real_variable_index[i]]);
        vterms2.push_back(c2.symbolic_vars[c2.real_variable_index[i]]);
    }

    std::unordered_map<std::string, std::string> mmap1 = c1.solver->model(vterms1);
    std::unordered_map<std::string, std::string> mmap2 = c1.solver->model(vterms2);

    std::fstream myfile;
    myfile.open(fname, std::ios::out | std::ios::trunc | std::ios::binary);
    myfile << "w12 = {" << std::endl;
    for (uint32_t i = 0; i < c1.get_num_vars(); i++) {
        std::string vname1 = vterms1[i].toString();
        std::string vname2 = vterms2[i].toString();
        if (c1.real_variable_index[i] == i) {
            myfile << "{" << mmap1[vname1] << ", " << mmap2[vname2] << "}";
            myfile << ",           // " << vname1 << ", " << vname2 << std::endl;
            if (mmap1[vname1] != mmap2[vname2]) {
                info(RED, "{", mmap1[vname1], ", ", mmap2[vname2], "}", ",           // ", vname1, ", ", vname2, RESET);
            }
        } else {
            myfile << "{" << mmap1[vname1] << ", " + mmap2[vname2] << "}";
            myfile << ",           // " << vname1 << " ," << vname2 << " -> " << c1.real_variable_index[i] << std::endl;
            if (mmap1[vname1] != mmap2[vname2]) {
                info(RED,
                     "{",
                     mmap1[vname1],
                     ", ",
                     mmap2[vname2],
                     "}",
                     ",           // ",
                     vname1,
                     ", ",
                     vname2,
                     " -> ",
                     c1.real_variable_index[i],
                     RESET);
            }
        }
    }
    myfile << "};";
    myfile.close();

    std::unordered_map<std::string, cvc5::Term> vterms;
    for (const auto& vname : special) {
        vterms.insert({ vname + "_1", c1[vname] });
        vterms.insert({ vname + "_2", c2[vname] });
    }

    std::unordered_map<std::string, std::string> mmap = c1.solver->model(vterms);
    for (const auto& vname : special) {
        info(vname, "_1, ", vname, "_2 = ", mmap[vname + "_1"], ", ", mmap[vname + "_2"]);
    }
}

/**
 * @brief Get pretty formatted result of the solver work
 *
 * @details Having a circuit and defined constraint system
 * inside the solver get the pretty formatted output.
 * The whole witness will be saved in c-like array format.
 * Special variables will be printed to stdout. e.g. `a = val_a;`
 *
 * @param special The list of variables that you need to see in stdout
 * @param c first circuit
 * @param s  solver
 * @param fname file to store the resulting witness if succeded
 */
void default_model_single(const std::vector<std::string>& special,
                          smt_circuit::CircuitBase& c,
                          const std::string& fname)
{
    std::vector<cvc5::Term> vterms;
    vterms.reserve(c.get_num_vars());

    for (uint32_t i = 0; i < c.get_num_vars(); i++) {
        vterms.push_back(c.symbolic_vars[c.real_variable_index[i]]);
    }

    std::unordered_map<std::string, std::string> mmap = c.solver->model(vterms);

    std::fstream myfile;
    myfile.open(fname, std::ios::out | std::ios::trunc | std::ios::binary);
    myfile << "w = {" << std::endl;
    for (size_t i = 0; i < c.get_num_vars(); i++) {
        std::string vname = vterms[i].toString();
        if (c.real_variable_index[i] == i) {
            myfile << mmap[vname] << ",              // " << vname << std::endl;
        } else {
            myfile << mmap[vname] << ",              // " << vname << " -> " << c.real_variable_index[i] << std::endl;
        }
    }
    myfile << "};";
    myfile.close();

    std::unordered_map<std::string, cvc5::Term> vterms1;
    for (const auto& vname : special) {
        vterms1.insert({ vname, c[vname] });
    }

    std::unordered_map<std::string, std::string> mmap1 = c.solver->model(vterms1);
    for (const auto& vname : special) {
        info(vname, " = ", mmap1[vname]);
    }
}

/**
 * @brief Get the solver result and amount of time
 * that it took to solve.
 *
 * @param s
 * @return bool is system satisfiable?
 */
bool smt_timer(smt_solver::Solver* s)
{
    auto start = std::chrono::high_resolution_clock::now();
    bool res = s->check();
    auto stop = std::chrono::high_resolution_clock::now();
    uint32_t duration_secs = static_cast<uint32_t>(duration_cast<std::chrono::seconds>(stop - start).count());
    info("Time elapsed: ", duration_secs / 60, " min ", duration_secs % 60, " sec");

    info(s->cvc_result);
    return res;
}

/**
 * @brief base4 decomposition with accumulators
 *
 * @param el
 * @return base decomposition, accumulators
 */
std::pair<std::vector<bb::fr>, std::vector<bb::fr>> base4(uint32_t el)
{
    std::vector<bb::fr> limbs;
    limbs.reserve(16);
    for (size_t i = 0; i < 16; i++) {
        limbs.emplace_back(el % 4);
        el /= 4;
    }
    std::reverse(limbs.begin(), limbs.end());
    std::vector<bb::fr> accumulators;
    accumulators.reserve(16);
    bb::fr accumulator = 0;
    for (size_t i = 0; i < 16; i++) {
        accumulator = accumulator * 4 + limbs[i];
        accumulators.emplace_back(accumulator);
    }
    std::reverse(limbs.begin(), limbs.end());
    std::reverse(accumulators.begin(), accumulators.end());
    return { limbs, accumulators };
}

/**
 * @brief Fix the triples from range_lists in the witness
 * @details Since we are not using the part of the witness, that
 * contains range lists, they are set to 0 by the solver. We need to
 * overwrite them to check the witness obtained by the solver.
 *
 * @param builder
 */
void fix_range_lists(bb::UltraCircuitBuilder& builder)
{
    for (auto list : builder.range_lists) {
        uint64_t num_multiples_of_three = (list.first / bb::UltraCircuitBuilder::DEFAULT_PLOOKUP_RANGE_STEP_SIZE);
        for (uint64_t i = 0; i <= num_multiples_of_three; i++) {
            builder.variables[list.second.variable_indices[i]] =
                i * bb::UltraCircuitBuilder::DEFAULT_PLOOKUP_RANGE_STEP_SIZE;
        }
        builder.variables[list.second.variable_indices[num_multiples_of_three + 1]] = list.first;
    }
}