#include <iostream>

#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/plonk/proof_system/verification_key/sol_gen.hpp"

#include "circuits/add_2_circuit.hpp"
#include "circuits/blake_circuit.hpp"
#include "circuits/ecdsa_circuit.hpp"
#include "circuits/recursive_circuit.hpp"

#include "utils/instance_sol_gen.hpp"
#include "utils/utils.hpp"

template <typename Composer, template <typename> typename Circuit>
void generate_keys(std::string output_path, std::string flavour_prefix, std::string circuit_name)
{
    uint256_t public_inputs[4] = { 0, 0, 0, 0 };
    auto circuit = Circuit<typename Composer::CircuitBuilder>::generate(public_inputs);

    Composer composer;
    std::shared_ptr<plonk::verification_key> vkey = composer.compute_verification_key(circuit);

    // Make verification key file upper case
    circuit_name.at(0) = static_cast<char>(std::toupper(static_cast<unsigned char>(circuit_name.at(0))));
    flavour_prefix.at(0) = static_cast<char>(std::toupper(static_cast<unsigned char>(flavour_prefix.at(0))));

    std::string vk_class_name = circuit_name + flavour_prefix + "VerificationKey";
    std::string base_class_name = "Base" + flavour_prefix + "Verifier";
    std::string instance_class_name = circuit_name + flavour_prefix + "Verifier";

    {
        auto vk_filename = output_path + "/keys/" + vk_class_name + ".sol";
        std::ofstream os(vk_filename);
        bb::output_vk_sol(os, vkey, vk_class_name);
        info("VK contract written to: ", vk_filename);
    }

    {
        auto instance_filename = output_path + "/instance/" + instance_class_name + ".sol";
        std::ofstream os(instance_filename);
        output_instance(os, vk_class_name, base_class_name, instance_class_name);
        info("Verifier instance written to: ", instance_filename);
    }
}

/*
 * @brief Main entry point for the verification key generator
 *
 * 1. project_root_path: path to the solidity project root
 * 2. srs_path: path to the srs db
 */
int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);

    if (args.size() < 5) {
        info("usage: ", args[0], "[plonk flavour] [circuit flavour] [output path] [srs path]");
        return 1;
    }

    const std::string plonk_flavour = args[1];
    const std::string circuit_flavour = args[2];
    const std::string output_path = args[3];
    const std::string srs_path = args[4];

    bb::srs::init_crs_factory(srs_path);
    // @todo - Add support for unrolled standard verifier. Needs a new solidity verifier contract.

    if (plonk_flavour != "ultra") {
        info("Flavour must be ultra");
        return 1;
    }

    info("Generating ", plonk_flavour, " keys for ", circuit_flavour, " circuit");

    if (circuit_flavour == "blake") {
        generate_keys<bb::plonk::UltraComposer, BlakeCircuit>(output_path, plonk_flavour, circuit_flavour);
    } else if (circuit_flavour == "add2") {
        generate_keys<bb::plonk::UltraComposer, Add2Circuit>(output_path, plonk_flavour, circuit_flavour);
    } else if (circuit_flavour == "recursive") {
        generate_keys<bb::plonk::UltraComposer, RecursiveCircuit>(output_path, plonk_flavour, circuit_flavour);
    } else if (circuit_flavour == "ecdsa") {
        generate_keys<bb::plonk::UltraComposer, EcdsaCircuit>(output_path, plonk_flavour, circuit_flavour);
    } else {
        info("Unsupported circuit");
        return 1;
    }
    return 0;
}