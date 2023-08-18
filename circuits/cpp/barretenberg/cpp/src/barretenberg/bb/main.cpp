#include "barretenberg/bb/get_crs.hpp"
#include "get_bytecode.hpp"
#include "get_witness.hpp"
#include <barretenberg/common/container.hpp>
#include <barretenberg/dsl/acir_format/acir_to_constraint_buf.hpp>
#include <barretenberg/dsl/acir_proofs/acir_composer.hpp>
#include <barretenberg/srs/global_crs.hpp>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

using namespace barretenberg;

uint32_t MAX_CIRCUIT_SIZE = 1 << 19;
std::string CRS_PATH = "./crs";
bool verbose = false;

void init()
{
    // Must +1!
    auto g1_data = get_g1_data(CRS_PATH, MAX_CIRCUIT_SIZE + 1);
    auto g2_data = get_g2_data(CRS_PATH);
    srs::init_crs_factory(g1_data, g2_data);
}

acir_format::WitnessVector get_witness(std::string const& witness_path)
{
    auto witness_data = get_witness_data(witness_path);
    return acir_format::witness_buf_to_witness_data(witness_data);
}

acir_format::acir_format get_constraint_system(std::string const& bytecode_path)
{
    auto bytecode = get_bytecode(bytecode_path);
    return acir_format::circuit_buf_to_acir_format(bytecode);
}

bool proveAndVerify(const std::string& bytecodePath, const std::string& witnessPath, bool recursive)
{
    auto acir_composer = new acir_proofs::AcirComposer(MAX_CIRCUIT_SIZE, verbose);
    auto constraint_system = get_constraint_system(bytecodePath);
    auto witness = get_witness(witnessPath);
    auto proof = acir_composer->create_proof(srs::get_crs_factory(), constraint_system, witness, recursive);
    auto verified = acir_composer->verify_proof(proof, recursive);
    info("verified: ", verified);
    return verified;
}

void prove(const std::string& bytecodePath,
           const std::string& witnessPath,
           bool recursive,
           const std::string& outputPath)
{
    auto acir_composer = new acir_proofs::AcirComposer(MAX_CIRCUIT_SIZE, verbose);
    auto constraint_system = get_constraint_system(bytecodePath);
    auto witness = get_witness(witnessPath);
    auto proof = acir_composer->create_proof(srs::get_crs_factory(), constraint_system, witness, recursive);
    write_file(outputPath, proof);
    info("proof written to: ", outputPath);
}

void gateCount(const std::string& bytecodePath)
{
    auto acir_composer = new acir_proofs::AcirComposer(MAX_CIRCUIT_SIZE, verbose);
    auto constraint_system = get_constraint_system(bytecodePath);
    acir_composer->create_circuit(constraint_system);
    info("gates: ", acir_composer->get_total_circuit_size());
}

bool verify(const std::string& proof_path, bool recursive, const std::string& vk_path)
{
    auto acir_composer = new acir_proofs::AcirComposer(MAX_CIRCUIT_SIZE, verbose);
    auto vk_data = from_buffer<plonk::verification_key_data>(read_file(vk_path));
    acir_composer->load_verification_key(barretenberg::srs::get_crs_factory(), std::move(vk_data));
    auto verified = acir_composer->verify_proof(read_file(proof_path), recursive);
    info("verified: ", verified);
    return verified;
}

void writeVk(const std::string& bytecodePath, const std::string& outputPath)
{
    auto acir_composer = new acir_proofs::AcirComposer(MAX_CIRCUIT_SIZE, verbose);
    auto constraint_system = get_constraint_system(bytecodePath);
    acir_composer->init_proving_key(srs::get_crs_factory(), constraint_system);
    auto vk = acir_composer->init_verification_key();
    write_file(outputPath, to_buffer(*vk));
    info("vk written to: ", outputPath);
}

void contract(const std::string& output_path, const std::string& vk_path)
{
    auto acir_composer = new acir_proofs::AcirComposer(MAX_CIRCUIT_SIZE, verbose);
    auto vk_data = from_buffer<plonk::verification_key_data>(read_file(vk_path));
    acir_composer->load_verification_key(barretenberg::srs::get_crs_factory(), std::move(vk_data));
    auto contract = acir_composer->get_solidity_verifier();
    if (output_path == "-") {
        info(contract);
    } else {
        write_file(output_path, { contract.begin(), contract.end() });
        info("contract written to: ", output_path);
    }
}

void proofAsFields(const std::string& proof_path, std::string const& vk_path, const std::string& output_path)
{
    auto acir_composer = new acir_proofs::AcirComposer(MAX_CIRCUIT_SIZE, verbose);
    auto vk_data = from_buffer<plonk::verification_key_data>(read_file(vk_path));
    auto data = acir_composer->serialize_proof_into_fields(read_file(proof_path), vk_data.num_public_inputs);
    auto json = format("[", join(map(data, [](auto fr) { return format("\"", fr, "\""); })), "]");
    write_file(output_path, { json.begin(), json.end() });
    info("proof as fields written to: ", output_path);
}

void vkAsFields(const std::string& vk_path, const std::string& output_path)
{
    auto acir_composer = new acir_proofs::AcirComposer(MAX_CIRCUIT_SIZE, verbose);
    auto vk_data = from_buffer<plonk::verification_key_data>(read_file(vk_path));
    acir_composer->load_verification_key(barretenberg::srs::get_crs_factory(), std::move(vk_data));
    auto data = acir_composer->serialize_verification_key_into_fields();

    // We need to move vk_hash to the front...
    std::rotate(data.begin(), data.end() - 1, data.end());

    auto json = format("[", join(map(data, [](auto fr) { return format("\"", fr, "\""); })), "]");
    write_file(output_path, { json.begin(), json.end() });
    info("vk as fields written to: ", output_path);
}

bool flagPresent(std::vector<std::string>& args, const std::string& flag)
{
    return std::find(args.begin(), args.end(), flag) != args.end();
}

std::string getOption(std::vector<std::string>& args, const std::string& option, const std::string& defaultValue)
{
    auto itr = std::find(args.begin(), args.end(), option);
    return (itr != args.end() && std::next(itr) != args.end()) ? *(std::next(itr)) : defaultValue;
}

int main(int argc, char* argv[])
{
    try {
        std::vector<std::string> args(argv + 1, argv + argc);
        verbose = flagPresent(args, "-v") || flagPresent(args, "--verbose");

        if (args.empty()) {
            std::cerr << "No command provided.\n";
            return 1;
        }

        std::string command = args[0];

        std::string bytecode_path = getOption(args, "-b", "./target/main.bytecode");
        std::string witness_path = getOption(args, "-w", "./target/witness.tr");
        std::string proof_path = getOption(args, "-p", "./proofs/proof");
        std::string vk_path = getOption(args, "-k", "./target/vk");
        CRS_PATH = getOption(args, "-c", "./crs");
        bool recursive = flagPresent(args, "-r") || flagPresent(args, "--recursive");
        init();

        if (command == "prove_and_verify") {
            return proveAndVerify(bytecode_path, witness_path, recursive) ? 0 : 1;
        } else if (command == "prove") {
            std::string output_path = getOption(args, "-o", "./proofs/proof");
            prove(bytecode_path, witness_path, recursive, output_path);
        } else if (command == "gates") {
            gateCount(bytecode_path);
        } else if (command == "verify") {
            verify(proof_path, recursive, vk_path);
        } else if (command == "contract") {
            std::string output_path = getOption(args, "-o", "./target/contract.sol");
            contract(output_path, vk_path);
        } else if (command == "write_vk") {
            std::string output_path = getOption(args, "-o", "./target/vk");
            writeVk(bytecode_path, output_path);
        } else if (command == "proof_as_fields") {
            std::string output_path = getOption(args, "-o", proof_path + "_fields.json");
            proofAsFields(proof_path, vk_path, output_path);
        } else if (command == "vk_as_fields") {
            std::string output_path = getOption(args, "-o", vk_path + "_fields.json");
            vkAsFields(vk_path, output_path);
        } else {
            std::cerr << "Unknown command: " << command << "\n";
            return 1;
        }
    } catch (std::runtime_error const& err) {
        std::cerr << err.what() << std::endl;
        return 1;
    }
}