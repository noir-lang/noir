#include "smt_util.hpp"

/**
 * @brief Converts a string of an arbitrary base to fr.
 * Note: there should be no prefix
 *
 * @param number string to be converted
 * @param base base representation of the string
 * @param step power n such that base^n <= 2^64. If base = 2, 10, 16. May remain undeclared.
 * @return bb::fr
 */
bb::fr string_to_fr(const std::string& number, int base, size_t step)
{
    bb::fr res = 0;
    char* ptr = nullptr;
    if (base == 2) {
        step = 64;
    } else if (base == 16) {
        step = 4;
    } else if (base == 10) {
        step = 19;
    } else if (step == 0) {
        info("Step should be non zero");
        return 0;
    }

    size_t i = number[0] == '-' ? 1 : 0;
    bb::fr step_power = bb::fr(base).pow(step);
    for (; i < number.size(); i += step) {
        std::string slice = number.substr(i, step);
        bb::fr cur_power = i + step > number.size() ? bb::fr(base).pow(number.size() - i) : step_power;
        res *= cur_power;
        res += std::strtoull(slice.data(), &ptr, base);
    }
    res = number[0] == '-' ? -res : res;
    return res;
}

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
 * @param pack flags out to pack the resulting witness using msgpack
 */
void default_model(const std::vector<std::string>& special,
                   smt_circuit::CircuitBase& c1,
                   smt_circuit::CircuitBase& c2,
                   const std::string& fname,
                   bool pack)
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

    std::vector<std::vector<bb::fr>> packed_witness;
    packed_witness.reserve(c1.get_num_vars());
    int base = c1.type == smt_terms::TermType::BVTerm ? 2 : 10;

    for (uint32_t i = 0; i < c1.get_num_vars(); i++) {
        std::string vname1 = vterms1[i].toString();
        std::string vname2 = vterms2[i].toString();
        std::string new_line = "{" + mmap1[vname1] + ", " + mmap2[vname2] + "},           // " + vname1 + ", " + vname2;

        if (c1.real_variable_index[i] != i) {
            new_line += " -> " + std::to_string(c1.real_variable_index[i]);
        }

        if (mmap1[vname1] != mmap2[vname2]) {
            info(RED, new_line, RESET);
        }
        myfile << new_line << std::endl;
        ;

        packed_witness.push_back({ string_to_fr(mmap1[vname1], base), string_to_fr(mmap2[vname2], base) });
    }
    myfile << "};";
    myfile.close();

    if (pack) {
        msgpack::sbuffer buffer;
        msgpack::pack(buffer, packed_witness);

        std::fstream myfile;
        myfile.open(fname + ".pack", std::ios::out | std::ios::trunc | std::ios::binary);

        myfile.write(buffer.data(), static_cast<int64_t>(buffer.size()));
        myfile.close();
    }

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
 * @param pack flags out to pack the resulting witness using msgpack
 */
void default_model_single(const std::vector<std::string>& special,
                          smt_circuit::CircuitBase& c,
                          const std::string& fname,
                          bool pack)
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

    std::vector<bb::fr> packed_witness;
    packed_witness.reserve(c.get_num_vars());
    int base = c.type == smt_terms::TermType::BVTerm ? 2 : 10;

    for (size_t i = 0; i < c.get_num_vars(); i++) {
        std::string vname = vterms[i].toString();
        std::string new_line = mmap[vname] + ",              // " + vname;
        if (c.real_variable_index[i] != i) {
            new_line += " -> " + std::to_string(c.real_variable_index[i]);
        }
        myfile << new_line << std::endl;
        packed_witness.push_back(string_to_fr(mmap[vname], base));
    }
    myfile << "};";
    myfile.close();

    if (pack) {
        msgpack::sbuffer buffer;
        msgpack::pack(buffer, packed_witness);

        std::fstream myfile;
        myfile.open(fname + ".pack", std::ios::out | std::ios::trunc | std::ios::binary);

        myfile.write(buffer.data(), static_cast<int64_t>(buffer.size()));
        myfile.close();
    }

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
 * @brief Import witness, obtained by solver, from file.
 * @details Imports the witness, that was packed by default_model function
 *
 * @param fname
 * @return std::vector<std::vector<bb::fr>>
 */
std::vector<std::vector<bb::fr>> import_witness(const std::string& fname)
{
    std::ifstream fin;
    fin.open(fname, std::ios::ate | std::ios::binary);
    if (!fin.is_open()) {
        throw std::invalid_argument("file not found");
    }
    if (fin.tellg() == -1) {
        throw std::invalid_argument("something went wrong");
    }

    uint64_t fsize = static_cast<uint64_t>(fin.tellg());
    fin.seekg(0, std::ios_base::beg);

    std::vector<std::vector<bb::fr>> res;
    char* encoded_data = new char[fsize];
    fin.read(encoded_data, static_cast<std::streamsize>(fsize));
    msgpack::unpack(encoded_data, fsize).get().convert(res);
    return res;
}

/**
 * @brief Import witness, obtained by solver, from file.
 * @details Imports the witness, that was packed by default_model_single function
 *
 * @param fname
 * @return std::vector<std::vector<bb::fr>>
 */
std::vector<bb::fr> import_witness_single(const std::string& fname)
{
    std::ifstream fin;
    fin.open(fname, std::ios::ate | std::ios::binary);
    if (!fin.is_open()) {
        throw std::invalid_argument("file not found");
    }
    if (fin.tellg() == -1) {
        throw std::invalid_argument("something went wrong");
    }

    uint64_t fsize = static_cast<uint64_t>(fin.tellg());
    fin.seekg(0, std::ios_base::beg);

    std::vector<bb::fr> res;
    char* encoded_data = new char[fsize];
    fin.read(encoded_data, static_cast<std::streamsize>(fsize));
    msgpack::unpack(encoded_data, fsize).get().convert(res);
    return res;
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