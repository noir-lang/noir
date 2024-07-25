#pragma once

/**
 * Write a solidity file containing the vk params to the given stream.
 * Uses UltraHonk
 *
 * @param os
 * @param key - verification key object
 * @param class_name - the name to give the verification key class
 * @param include_types_import - include a "HonkTypes" import, only required for local tests, not with the bundled
 *contract from bb contract_honk
 **/
inline void output_vk_sol_ultra_honk(std::ostream& os,
                                     auto const& key,
                                     std::string const& class_name,
                                     bool include_types_import = false)
{

    const auto print_u256_const = [&](const auto& element, const std::string& name) {
        os << "uint256 constant " << name << " = " << element << ";" << std::endl;
    };

    const auto print_u256 = [&](const auto& element, const std::string& name) {
        os << "            " << name << ": uint256(" << element << ")," << std::endl;
    };

    const auto print_g1 = [&](const auto& element, const std::string& name, const bool last = false) {
        os << "            " << name << ": Honk.G1Point({ \n"
           << "               "
           << "x: "
           << "uint256(" << element.x << "),\n"
           << "               "
           << "y: "
           << "uint256(" << element.y << ")\n"
           << "            })";

        // only include comma if we are not the last element
        if (!last) {
            os << ",\n";
        } else {
            os << "\n";
        }
    };

    // Include the types import if working with the local test suite
    const auto print_types_import = [&]() {
        if (include_types_import) {
            os << "import { Honk } from \"../HonkTypes.sol\";\n";
        }
    };

    // clang-format off
    os <<
    //   "// Verification Key Hash: " << key->sha256_hash() << "\n"
      "// SPDX-License-Identifier: Apache-2.0\n"
      "// Copyright 2022 Aztec\n"
      "pragma solidity >=0.8.21;\n"
      "\n"
    "";
    print_types_import();
    print_u256_const(key->circuit_size, "N");
    print_u256_const(key->log_circuit_size, "LOG_N");
    print_u256_const(key->num_public_inputs, "NUMBER_OF_PUBLIC_INPUTS");
    os << ""
    "library " << class_name << " {\n"
      "    function loadVerificationKey() internal pure returns (Honk.VerificationKey memory) {\n"
      "        Honk.VerificationKey memory vk = Honk.VerificationKey({\n";
    print_u256(key->circuit_size, "circuitSize");
    print_u256(key->log_circuit_size, "logCircuitSize");
    print_u256(key->num_public_inputs, "publicInputsSize");
    print_g1(key->q_l, "ql");
    print_g1(key->q_r, "qr");
    print_g1(key->q_o, "qo");
    print_g1(key->q_4, "q4");
    print_g1(key->q_m, "qm");
    print_g1(key->q_c, "qc");
    print_g1(key->q_arith, "qArith");
    print_g1(key->q_delta_range, "qDeltaRange");
    print_g1(key->q_elliptic, "qElliptic");
    print_g1(key->q_aux, "qAux");
    print_g1(key->q_lookup, "qLookup");
    print_g1(key->sigma_1, "s1");
    print_g1(key->sigma_2, "s2");
    print_g1(key->sigma_3, "s3");
    print_g1(key->sigma_4, "s4");
    print_g1(key->table_1, "t1");
    print_g1(key->table_2, "t2");
    print_g1(key->table_3, "t3");
    print_g1(key->table_4, "t4");
    // print_g1("0x500", "0x520", key->table, "vk.TABLE_TYPE");
    print_g1(key->id_1, "id1");
    print_g1(key->id_2, "id2");
    print_g1(key->id_3, "id3");
    print_g1(key->id_4, "id4");
    print_g1(key->lagrange_first, "lagrangeFirst");
    print_g1(key->lagrange_last, "lagrangeLast", /*last=*/ true);
    os <<
        "        });\n"
        "        return vk;\n"
        "    }\n"
        "}\n";

    os << std::flush;
}