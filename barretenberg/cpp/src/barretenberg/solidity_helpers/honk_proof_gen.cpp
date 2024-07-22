
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

#include "circuits/add_2_circuit.hpp"
#include "circuits/blake_circuit.hpp"
#include "circuits/ecdsa_circuit.hpp"
#include "utils/utils.hpp"

#include <iostream>
#include <sstream>

using namespace bb;
using numeric::uint256_t;

using ProverInstance = ProverInstance_<UltraKeccakFlavor>;
using VerificationKey = UltraKeccakFlavor::VerificationKey;
using Prover = UltraKeccakProver;
using Verifier = UltraKeccakVerifier;

template <template <typename> typename Circuit> void generate_proof(uint256_t inputs[])
{

    UltraCircuitBuilder builder = Circuit<UltraCircuitBuilder>::generate(inputs);

    auto instance = std::make_shared<ProverInstance>(builder);
    Prover prover(instance);
    auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);
    Verifier verifier(verification_key);

    HonkProof proof = prover.construct_proof();
    {
        if (!verifier.verify_proof(proof)) {
            throw_or_abort("Verification failed");
        }

        std::vector<uint8_t> proof_bytes = to_buffer(proof);
        std::string p = bytes_to_hex_string(proof_bytes);
        std::cout << p;
    }
}

std::string pad_left(std::string input, size_t length)
{
    return std::string(length - std::min(length, input.length()), '0') + input;
}

/**
 * @brief Main entry point for the proof generator.
 * Expected inputs:
 * 1. plonk_flavour: ultra
 * 2. circuit_flavour: blake, add2
 * 3. public_inputs: comma separated list of public inputs
 * 4. project_root_path: path to the solidity project root
 * 5. srs_path: path to the srs db
 */
int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);

    if (args.size() < 5) {
        info("usage: ", args[0], "[plonk flavour] [circuit flavour] [srs path] [public inputs]");
        return 1;
    }

    const std::string plonk_flavour = args[1];
    const std::string circuit_flavour = args[2];
    const std::string srs_path = args[3];
    const std::string string_input = args[4];

    bb::srs::init_crs_factory(srs_path);

    // @todo dynamically allocate this
    uint256_t inputs[] = { 0, 0, 0, 0, 0, 0 };

    size_t count = 0;
    std::stringstream s_stream(string_input);
    while (s_stream.good()) {
        std::string sub;
        getline(s_stream, sub, ',');
        if (sub.substr(0, 2) == "0x") {
            sub = sub.substr(2);
        }
        std::string padded = pad_left(sub, 64);
        inputs[count++] = uint256_t(padded);
    }

    if (plonk_flavour != "honk") {
        info("Only honk flavor allowed");
        return 1;
    }

    if (circuit_flavour == "blake") {
        generate_proof<BlakeCircuit>(inputs);
    } else if (circuit_flavour == "add2") {
        generate_proof<Add2Circuit>(inputs);
    } else if (circuit_flavour == "ecdsa") {
        generate_proof<EcdsaCircuit>(inputs);
    } else {
        info("Invalid circuit flavour: " + circuit_flavour);
        return 1;
    }
}