#include "circuit_schema.hpp"

namespace smt_circuit_schema {

/**
 * @brief Get the CircuitSchema object
 * @details Initialize the CircuitSchema from the binary file
 * that contains an msgpack compatible buffer.
 *
 * @param filename
 * @return CircuitSchema
 */
CircuitSchema unpack_from_file(const std::string& filename)
{
    std::ifstream fin;
    fin.open(filename, std::ios::ate | std::ios::binary);
    if (!fin.is_open()) {
        throw std::invalid_argument("file not found");
    }
    if (fin.tellg() == -1) {
        throw std::invalid_argument("something went wrong");
    }

    uint64_t fsize = static_cast<uint64_t>(fin.tellg());
    fin.seekg(0, std::ios_base::beg);

    CircuitSchema cir;
    char* encoded_data = new char[fsize];
    fin.read(encoded_data, static_cast<std::streamsize>(fsize));
    msgpack::unpack(encoded_data, fsize).get().convert(cir);
    return cir;
}

/**
 * @brief Get the CircuitSchema object
 * @details Initialize the CircuitSchema from the msgpack compatible buffer.
 *
 * @param buf
 * @return CircuitSchema
 */
CircuitSchema unpack_from_buffer(const msgpack::sbuffer& buf)
{
    CircuitSchema cir;
    msgpack::unpack(buf.data(), buf.size()).get().convert(cir);
    return cir;
}

/**
 * @brief Translates the schema to python format
 * @details Returns the contents of the .py file
 * that can be further imported by python script
 *
 * @example output.py
 * variables = ["zero", "one", "my_input1", "var3", ..., "var_n"] - variable names
 * public = [[0, 0x000...0], [1, 0x000...1], [2, 0x000..abcd]] - (index, value)
 * gates = [
 *  [[0x000...0, 0x000...1, 0x000...0, 0x000...0, 0x000...0], [0, 0, 0]], ...
 * ]
 * @todo UltraCircuitSchema output
 */
void print_schema_for_use_in_python(CircuitSchema& cir)
{
    info("variable_names = [");
    for (uint32_t i = 0; i < static_cast<uint32_t>(cir.variables.size()); i++) {
        if (cir.vars_of_interest.contains(i)) {
            info('"', cir.vars_of_interest[i], "\",");
        } else {
            info("\"v", i, "\",");
        }
    }
    info("]");
    info("public = [");
    for (auto i : cir.public_inps) {
        info("[", i, ", ", cir.variables[i], "],");
    }
    info("]");
    info("gates = [");

    for (size_t i = 0; i < cir.selectors.size(); i++) {
        info("[",
             "[",
             cir.selectors[0][i][0],
             ", ",
             cir.selectors[0][i][1],
             ", ",
             cir.selectors[0][i][2],
             ", ",
             cir.selectors[0][i][3],
             ", ",
             cir.selectors[0][i][4],
             "], [",
             cir.wires[0][i][0],
             ", ",
             cir.wires[0][i][1],
             ", ",
             cir.wires[0][i][2],
             "]],");
    }
    info("]");
}
} // namespace smt_circuit_schema