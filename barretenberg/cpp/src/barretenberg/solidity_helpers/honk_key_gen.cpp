
#include <iostream>
#include <memory>

#include "barretenberg/honk/utils/honk_key_gen.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

#include "circuits/add_2_circuit.hpp"
#include "circuits/blake_circuit.hpp"
#include "circuits/ecdsa_circuit.hpp"

using namespace bb;

using ProverInstance = ProverInstance_<UltraKeccakFlavor>;
using VerificationKey = UltraKeccakFlavor::VerificationKey;

template <template <typename> typename Circuit>
void generate_keys_honk(std::string output_path, std::string flavour_prefix, std::string circuit_name)
{
    uint256_t public_inputs[4] = { 0, 0, 0, 0 };
    UltraCircuitBuilder builder = Circuit<UltraCircuitBuilder>::generate(public_inputs);

    auto instance = std::make_shared<ProverInstance>(builder);
    UltraKeccakProver prover(instance);
    auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);

    // Make verification key file upper case
    circuit_name.at(0) = static_cast<char>(std::toupper(static_cast<unsigned char>(circuit_name.at(0))));
    flavour_prefix.at(0) = static_cast<char>(std::toupper(static_cast<unsigned char>(flavour_prefix.at(0))));

    std::string vk_class_name = circuit_name + flavour_prefix + "VerificationKey";
    std::string base_class_name = "Base" + flavour_prefix + "Verifier";
    std::string instance_class_name = circuit_name + flavour_prefix + "Verifier";

    {
        auto vk_filename = output_path + "/keys/" + vk_class_name + ".sol";
        std::ofstream os(vk_filename);
        output_vk_sol_ultra_honk(os, verification_key, vk_class_name, true);
        info("VK contract written to: ", vk_filename);
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

    if (plonk_flavour != "honk") {
        info("honk");
        return 1;
    }

    info("Generating ", plonk_flavour, " keys for ", circuit_flavour, " circuit");

    if (plonk_flavour == "honk") {
        if (circuit_flavour == "add2") {
            generate_keys_honk<Add2Circuit>(output_path, plonk_flavour, circuit_flavour);
        } else if (circuit_flavour == "blake") {
            generate_keys_honk<BlakeCircuit>(output_path, plonk_flavour, circuit_flavour);
        } else if (circuit_flavour == "ecdsa") {
            generate_keys_honk<bb::EcdsaCircuit>(output_path, plonk_flavour, circuit_flavour);
            // TODO: recursive proofs
        } else {
            info("Unsupported circuit");
            return 1;
        }
    }
    return 0;
} // namespace bb