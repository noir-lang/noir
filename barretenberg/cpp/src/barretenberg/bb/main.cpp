#include "barretenberg/bb/file_io.hpp"
#include "barretenberg/client_ivc/client_ivc.hpp"
#include "barretenberg/common/map.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/dsl/acir_format/acir_format.hpp"
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/proving_key/serialize.hpp"
#include "barretenberg/serialize/cbind.hpp"
#include "barretenberg/stdlib/honk_recursion/verifier/client_ivc_recursive_verifier.hpp"
#include <cstddef>
#ifndef DISABLE_AZTEC_VM
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_execution.hpp"
#include "barretenberg/vm/avm_trace/stats.hpp"
#endif
#include "config.hpp"
#include "get_bn254_crs.hpp"
#include "get_bytecode.hpp"
#include "get_grumpkin_crs.hpp"
#include "libdeflate.h"
#include "log.hpp"
#include <barretenberg/common/benchmark.hpp>
#include <barretenberg/common/container.hpp>
#include <barretenberg/common/log.hpp>
#include <barretenberg/common/timer.hpp>
#include <barretenberg/dsl/acir_format/acir_to_constraint_buf.hpp>
#include <barretenberg/dsl/acir_proofs/acir_composer.hpp>
#include <barretenberg/srs/global_crs.hpp>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

using namespace bb;

std::string getHomeDir()
{
    char* home = std::getenv("HOME");
    return home != nullptr ? std::string(home) : "./";
}

std::string CRS_PATH = getHomeDir() + "/.bb-crs";

const std::filesystem::path current_path = std::filesystem::current_path();
const auto current_dir = current_path.filename().string();

/**
 * @brief Initialize the global crs_factory for bn254 based on a known dyadic circuit size
 *
 * @param dyadic_circuit_size power-of-2 circuit size
 */
void init_bn254_crs(size_t dyadic_circuit_size)
{
    // Must +1 for Plonk only!
    auto bn254_g1_data = get_bn254_g1_data(CRS_PATH, dyadic_circuit_size + 1);
    auto bn254_g2_data = get_bn254_g2_data(CRS_PATH);
    srs::init_crs_factory(bn254_g1_data, bn254_g2_data);
}

/**
 * @brief Initialize the global crs_factory for grumpkin based on a known dyadic circuit size
 * @details Grumpkin crs is required only for the ECCVM
 *
 * @param dyadic_circuit_size power-of-2 circuit size
 */
void init_grumpkin_crs(size_t eccvm_dyadic_circuit_size)
{
    auto grumpkin_g1_data = get_grumpkin_g1_data(CRS_PATH, eccvm_dyadic_circuit_size);
    srs::init_grumpkin_crs_factory(grumpkin_g1_data);
}

// Initializes without loading G1
// TODO(https://github.com/AztecProtocol/barretenberg/issues/811) adapt for grumpkin
acir_proofs::AcirComposer verifier_init()
{
    acir_proofs::AcirComposer acir_composer(0, verbose_logging);
    auto g2_data = get_bn254_g2_data(CRS_PATH);
    srs::init_crs_factory({}, g2_data);
    return acir_composer;
}

acir_format::WitnessVector get_witness(std::string const& witness_path)
{
    auto witness_data = get_bytecode(witness_path);
    return acir_format::witness_buf_to_witness_data(witness_data);
}

acir_format::AcirFormat get_constraint_system(std::string const& bytecode_path, bool honk_recursion)
{
    auto bytecode = get_bytecode(bytecode_path);
    return acir_format::circuit_buf_to_acir_format(bytecode, honk_recursion);
}

acir_format::WitnessVectorStack get_witness_stack(std::string const& witness_path)
{
    auto witness_data = get_bytecode(witness_path);
    return acir_format::witness_buf_to_witness_stack(witness_data);
}

std::vector<acir_format::AcirFormat> get_constraint_systems(std::string const& bytecode_path, bool honk_recursion)
{
    auto bytecode = get_bytecode(bytecode_path);
    return acir_format::program_buf_to_acir_format(bytecode, honk_recursion);
}

std::string to_json(std::vector<bb::fr>& data)
{
    return format("[", join(map(data, [](auto fr) { return format("\"", fr, "\""); })), "]");
}

std::string vk_to_json(std::vector<bb::fr>& data)
{
    // We need to move vk_hash to the front...
    std::rotate(data.begin(), data.end() - 1, data.end());

    return format("[", join(map(data, [](auto fr) { return format("\"", fr, "\""); })), "]");
}

std::string honk_vk_to_json(std::vector<bb::fr>& data)
{
    return format("[", join(map(data, [](auto fr) { return format("\"", fr, "\""); })), "]");
}

/**
 * @brief Proves and Verifies an ACIR circuit
 *
 * Communication:
 * - proc_exit: A boolean value is returned indicating whether the proof is valid.
 *   an exit code of 0 will be returned for success and 1 for failure.
 *
 * @param bytecodePath Path to the file containing the serialized circuit
 * @param witnessPath Path to the file containing the serialized witness
 * @param recursive Whether to use recursive proof generation of non-recursive
 * @return true if the proof is valid
 * @return false if the proof is invalid
 */
bool proveAndVerify(const std::string& bytecodePath, const std::string& witnessPath)
{
    auto constraint_system = get_constraint_system(bytecodePath, /*honk_recursion=*/false);
    auto witness = get_witness(witnessPath);

    acir_proofs::AcirComposer acir_composer{ 0, verbose_logging };
    acir_composer.create_circuit(constraint_system, witness);

    init_bn254_crs(acir_composer.get_dyadic_circuit_size());

    Timer pk_timer;
    acir_composer.init_proving_key();
    write_benchmark("pk_construction_time", pk_timer.milliseconds(), "acir_test", current_dir);

    write_benchmark("gate_count", acir_composer.get_total_circuit_size(), "acir_test", current_dir);
    write_benchmark("subgroup_size", acir_composer.get_dyadic_circuit_size(), "acir_test", current_dir);

    Timer proof_timer;
    auto proof = acir_composer.create_proof();
    write_benchmark("proof_construction_time", proof_timer.milliseconds(), "acir_test", current_dir);

    Timer vk_timer;
    acir_composer.init_verification_key();
    write_benchmark("vk_construction_time", vk_timer.milliseconds(), "acir_test", current_dir);

    auto verified = acir_composer.verify_proof(proof);

    vinfo("verified: ", verified);
    return verified;
}

template <IsUltraFlavor Flavor>
bool proveAndVerifyHonkAcirFormat(acir_format::AcirFormat constraint_system, acir_format::WitnessVector witness)
{
    using Builder = Flavor::CircuitBuilder;
    using Prover = UltraProver_<Flavor>;
    using Verifier = UltraVerifier_<Flavor>;
    using VerificationKey = Flavor::VerificationKey;

    bool honk_recursion = false;
    if constexpr (IsAnyOf<Flavor, UltraFlavor>) {
        honk_recursion = true;
    }
    // Construct a bberg circuit from the acir representation
    auto builder = acir_format::create_circuit<Builder>(constraint_system, 0, witness, honk_recursion);

    auto num_extra_gates = builder.get_num_gates_added_to_ensure_nonzero_polynomials();
    size_t srs_size = builder.get_circuit_subgroup_size(builder.get_total_circuit_size() + num_extra_gates);
    init_bn254_crs(srs_size);

    // Construct Honk proof
    Prover prover{ builder };
    auto proof = prover.construct_proof();

    // Verify Honk proof
    auto verification_key = std::make_shared<VerificationKey>(prover.instance->proving_key);
    Verifier verifier{ verification_key };

    return verifier.verify_proof(proof);
}

/**
 * @brief Constructs and verifies a Honk proof for an acir-generated circuit
 *
 * @tparam Flavor
 * @param bytecodePath Path to serialized acir circuit data
 * @param witnessPath Path to serialized acir witness data
 */
template <IsUltraFlavor Flavor> bool proveAndVerifyHonk(const std::string& bytecodePath, const std::string& witnessPath)
{
    bool honk_recursion = false;
    if constexpr (IsAnyOf<Flavor, UltraFlavor>) {
        honk_recursion = true;
    }
    // Populate the acir constraint system and witness from gzipped data
    auto constraint_system = get_constraint_system(bytecodePath, honk_recursion);
    auto witness = get_witness(witnessPath);

    return proveAndVerifyHonkAcirFormat<Flavor>(constraint_system, witness);
}

/**
 * @brief Constructs and verifies multiple Honk proofs for an ACIR-generated program.
 *
 * @tparam Flavor
 * @param bytecodePath Path to serialized acir program data. An ACIR program contains a list of circuits.
 * @param witnessPath Path to serialized acir witness stack data. This dictates the execution trace the backend should
 * follow.
 */
template <IsUltraFlavor Flavor>
bool proveAndVerifyHonkProgram(const std::string& bytecodePath, const std::string& witnessPath)
{
    bool honk_recursion = false;
    if constexpr (IsAnyOf<Flavor, UltraFlavor>) {
        honk_recursion = true;
    }
    auto program_stack = acir_format::get_acir_program_stack(bytecodePath, witnessPath, honk_recursion);

    while (!program_stack.empty()) {
        auto stack_item = program_stack.back();

        if (!proveAndVerifyHonkAcirFormat<Flavor>(stack_item.constraints, stack_item.witness)) {
            return false;
        }
        program_stack.pop_back();
    }
    return true;
}

// TODO(#7371): this could probably be more idiomatic
template <typename T> T unpack_from_file(const std::string& filename)
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

    T result;
    char* encoded_data = new char[fsize];
    fin.read(encoded_data, static_cast<std::streamsize>(fsize));
    msgpack::unpack(encoded_data, fsize).get().convert(result);
    return result;
}

// TODO(#7371) find a home for this
acir_format::WitnessVector witness_map_to_witness_vector(std::map<std::string, std::string> const& witness_map)
{
    acir_format::WitnessVector wv;
    size_t index = 0;
    for (auto& e : witness_map) {
        uint64_t value = std::stoull(e.first);
        // ACIR uses a sparse format for WitnessMap where unused witness indices may be left unassigned.
        // To ensure that witnesses sit at the correct indices in the `WitnessVector`, we fill any indices
        // which do not exist within the `WitnessMap` with the dummy value of zero.
        while (index < value) {
            wv.push_back(fr(0));
            index++;
        }
        wv.push_back(fr(uint256_t(e.second)));
        index++;
    }
    return wv;
}

std::vector<uint8_t> decompressedBuffer(uint8_t* bytes, size_t size)
{
    std::vector<uint8_t> content;
    // initial size guess
    content.resize(1024ULL * 128ULL);
    for (;;) {
        auto decompressor = std::unique_ptr<libdeflate_decompressor, void (*)(libdeflate_decompressor*)>{
            libdeflate_alloc_decompressor(), libdeflate_free_decompressor
        };
        size_t actual_size = 0;
        libdeflate_result decompress_result = libdeflate_gzip_decompress(
            decompressor.get(), bytes, size, std::data(content), std::size(content), &actual_size);
        if (decompress_result == LIBDEFLATE_INSUFFICIENT_SPACE) {
            // need a bigger buffer
            content.resize(content.size() * 2);
            continue;
        }
        if (decompress_result == LIBDEFLATE_BAD_DATA) {
            throw std::invalid_argument("bad gzip data in bb main");
        }
        content.resize(actual_size);
        break;
    }
    return content;
}

void client_ivc_prove_output_all_msgpack(const std::string& bytecodePath,
                                         const std::string& witnessPath,
                                         const std::string& outputDir)
{
    using Flavor = MegaFlavor; // This is the only option
    using Builder = Flavor::CircuitBuilder;
    using Program = acir_format::AcirProgram;
    using ECCVMVK = ECCVMFlavor::VerificationKey;
    using TranslatorVK = TranslatorFlavor::VerificationKey;

    init_bn254_crs(1 << 24);
    init_grumpkin_crs(1 << 14);

    auto gzippedBincodes = unpack_from_file<std::vector<std::string>>(bytecodePath);
    auto witnessMaps = unpack_from_file<std::vector<std::string>>(witnessPath);
    std::vector<Program> folding_stack;
    for (size_t i = 0; i < gzippedBincodes.size(); i++) {
        // TODO(#7371) there is a lot of copying going on in bincode, we should make sure this writes as a buffer in
        // the future
        std::vector<uint8_t> buffer =
            decompressedBuffer(reinterpret_cast<uint8_t*>(&gzippedBincodes[i][0]), gzippedBincodes[i].size()); // NOLINT

        std::vector<acir_format::AcirFormat> constraint_systems = acir_format::program_buf_to_acir_format(
            buffer,
            false); // TODO(https://github.com/AztecProtocol/barretenberg/issues/1013):
                    // this assumes that folding is never done with ultrahonk.
        std::vector<uint8_t> witnessBuffer =
            decompressedBuffer(reinterpret_cast<uint8_t*>(&witnessMaps[i][0]), witnessMaps[i].size()); // NOLINT
        acir_format::WitnessVectorStack witness_stack = acir_format::witness_buf_to_witness_stack(witnessBuffer);
        acir_format::AcirProgramStack program_stack{ constraint_systems, witness_stack };
        folding_stack.push_back(program_stack.back());
    }
    // TODO(#7371) dedupe this with the rest of the similar code
    ClientIVC ivc;
    ivc.trace_structure = TraceStructure::E2E_FULL_TEST;

    // Accumulate the entire program stack into the IVC
    for (Program& program : folding_stack) {
        // auto& stack_item = program_stack.witness_stack[i];

        // Construct a bberg circuit from the acir representation
        auto circuit =
            acir_format::create_circuit<Builder>(program.constraints, 0, program.witness, false, ivc.goblin.op_queue);
        ivc.accumulate(circuit);
    }

    // Write the proof and verification keys into the working directory in  'binary' format (in practice it seems this
    // directory is passed by bb.js)
    std::string vkPath = outputDir + "/inst_vk"; // the vk of the last instance
    std::string accPath = outputDir + "/pg_acc";
    std::string proofPath = outputDir + "/client_ivc_proof";
    std::string translatorVkPath = outputDir + "/translator_vk";
    std::string eccVkPath = outputDir + "/ecc_vk";

    auto proof = ivc.prove();
    auto eccvm_vk = std::make_shared<ECCVMVK>(ivc.goblin.get_eccvm_proving_key());
    auto translator_vk = std::make_shared<TranslatorVK>(ivc.goblin.get_translator_proving_key());

    auto last_instance = std::make_shared<ClientIVC::VerifierInstance>(ivc.instance_vk);
    vinfo("ensure valid proof: ", ivc.verify(proof, { ivc.verifier_accumulator, last_instance }));

    vinfo("write proof and vk data to files..");
    write_file(proofPath, to_buffer(proof));
    write_file(vkPath, to_buffer(ivc.instance_vk));
    write_file(accPath, to_buffer(ivc.verifier_accumulator));
    write_file(translatorVkPath, to_buffer(translator_vk));
    write_file(eccVkPath, to_buffer(eccvm_vk));
}

template <typename T> std::shared_ptr<T> read_to_shared_ptr(const std::filesystem::path& path)
{
    return std::make_shared<T>(from_buffer<T>(read_file(path)));
};

/**
 * @brief Verifies a client ivc proof and writes the result to stdout
 *
 * Communication:
 * - proc_exit: A boolean value is returned indicating whether the proof is valid.
 *   an exit code of 0 will be returned for success and 1 for failure.
 *
 * @param proof_path Path to the file containing the serialized proof
 * @param vk_path Path to the file containing the serialized verification key of the final mega honk instance
 * @param accumualtor_path Path to the file containing the serialized protogalaxy accumulator
 * @return true (resp., false) if the proof is valid (resp., invalid).
 */
bool verify_client_ivc(const std::filesystem::path& proof_path,
                       const std::filesystem::path& accumulator_path,
                       const std::filesystem::path& final_vk_path,
                       const std::filesystem::path& eccvm_vk_path,
                       const std::filesystem::path& translator_vk_path)
{
    init_bn254_crs(1 << 24);
    init_grumpkin_crs(1 << 14);

    const auto proof = from_buffer<ClientIVC::Proof>(read_file(proof_path));
    const auto accumulator = read_to_shared_ptr<ClientIVC::VerifierInstance>(accumulator_path);
    accumulator->verification_key->pcs_verification_key = std::make_shared<VerifierCommitmentKey<curve::BN254>>();
    const auto final_vk = read_to_shared_ptr<ClientIVC::VerificationKey>(final_vk_path);
    const auto eccvm_vk = read_to_shared_ptr<ECCVMFlavor::VerificationKey>(eccvm_vk_path);
    eccvm_vk->pcs_verification_key =
        std::make_shared<VerifierCommitmentKey<curve::Grumpkin>>(eccvm_vk->circuit_size + 1);
    const auto translator_vk = read_to_shared_ptr<TranslatorFlavor::VerificationKey>(translator_vk_path);
    translator_vk->pcs_verification_key = std::make_shared<VerifierCommitmentKey<curve::BN254>>();

    const bool verified = ClientIVC::verify(
        proof, accumulator, std::make_shared<ClientIVC::VerifierInstance>(final_vk), eccvm_vk, translator_vk);
    vinfo("verified: ", verified);
    return verified;
}

bool foldAndVerifyProgram(const std::string& bytecodePath, const std::string& witnessPath)
{
    using Flavor = MegaFlavor; // This is the only option
    using Builder = Flavor::CircuitBuilder;

    init_bn254_crs(1 << 22);
    init_grumpkin_crs(1 << 16);

    ClientIVC ivc;
    ivc.trace_structure = TraceStructure::SMALL_TEST;

    auto program_stack = acir_format::get_acir_program_stack(
        bytecodePath, witnessPath, false); // TODO(https://github.com/AztecProtocol/barretenberg/issues/1013): this
                                           // assumes that folding is never done with ultrahonk.

    // Accumulate the entire program stack into the IVC
    while (!program_stack.empty()) {
        auto stack_item = program_stack.back();

        // Construct a bberg circuit from the acir representation
        auto circuit = acir_format::create_circuit<Builder>(
            stack_item.constraints, 0, stack_item.witness, false, ivc.goblin.op_queue);

        ivc.accumulate(circuit);

        program_stack.pop_back();
    }
    return ivc.prove_and_verify();
}

/**
 * @brief Recieves an ACIR Program stack that gets accumulated with the ClientIVC logic and produces a client IVC proof.
 *
 * @param bytecodePath Path to the serialised circuit
 * @param witnessPath Path to witness data
 * @param outputPath Path to the folder where the proof and verification data are goingt obe wr itten (in practice this
 * going to be specified when bb main is called, i.e. as the working directory in typescript).
 */
void client_ivc_prove_output_all(const std::string& bytecodePath,
                                 const std::string& witnessPath,
                                 const std::string& outputPath)
{
    using Flavor = MegaFlavor; // This is the only option
    using Builder = Flavor::CircuitBuilder;
    using ECCVMVK = ECCVMFlavor::VerificationKey;
    using TranslatorVK = TranslatorFlavor::VerificationKey;

    init_bn254_crs(1 << 22);
    init_grumpkin_crs(1 << 16);

    ClientIVC ivc;
    ivc.trace_structure = TraceStructure::E2E_FULL_TEST;

    auto program_stack = acir_format::get_acir_program_stack(
        bytecodePath, witnessPath, false); // TODO(https://github.com/AztecProtocol/barretenberg/issues/1013): this
                                           // assumes that folding is never done with ultrahonk.

    // Accumulate the entire program stack into the IVC
    while (!program_stack.empty()) {
        auto stack_item = program_stack.back();

        // Construct a bberg circuit from the acir representation
        auto circuit = acir_format::create_circuit<Builder>(
            stack_item.constraints, 0, stack_item.witness, false, ivc.goblin.op_queue);

        ivc.accumulate(circuit);

        program_stack.pop_back();
    }

    // Write the proof and verification keys into the working directory in  'binary' format (in practice it seems this
    // directory is passed by bb.js)
    std::string vkPath = outputPath + "/inst_vk"; // the vk of the last instance
    std::string accPath = outputPath + "/pg_acc";
    std::string proofPath = outputPath + "/client_ivc_proof";
    std::string translatorVkPath = outputPath + "/translator_vk";
    std::string eccVkPath = outputPath + "/ecc_vk";

    auto proof = ivc.prove();
    auto eccvm_vk = std::make_shared<ECCVMVK>(ivc.goblin.get_eccvm_proving_key());
    auto translator_vk = std::make_shared<TranslatorVK>(ivc.goblin.get_translator_proving_key());

    auto last_instance = std::make_shared<ClientIVC::VerifierInstance>(ivc.instance_vk);
    vinfo("ensure valid proof: ", ivc.verify(proof, { ivc.verifier_accumulator, last_instance }));

    vinfo("write proof and vk data to files..");
    write_file(proofPath, to_buffer(proof));
    write_file(vkPath, to_buffer(ivc.instance_vk)); // maybe dereference
    write_file(accPath, to_buffer(ivc.verifier_accumulator));
    write_file(translatorVkPath, to_buffer(translator_vk));
    write_file(eccVkPath, to_buffer(eccvm_vk));
}

/**
 * @brief Creates a Honk Proof for the Tube circuit responsible for recursively verifying a ClientIVC proof.
 *
 * @param output_path the working directory from which the proof and verification data are read
 * @param num_unused_public_inputs
 */
void prove_tube(const std::string& output_path)
{
    using ClientIVC = stdlib::recursion::honk::ClientIVCRecursiveVerifier;
    using NativeInstance = ClientIVC::FoldVerifierInput::Instance;
    using InstanceFlavor = MegaFlavor;
    using ECCVMVk = ECCVMFlavor::VerificationKey;
    using TranslatorVk = TranslatorFlavor::VerificationKey;
    using FoldVerifierInput = ClientIVC::FoldVerifierInput;
    using GoblinVerifierInput = ClientIVC::GoblinVerifierInput;
    using VerifierInput = ClientIVC::VerifierInput;
    using Builder = UltraCircuitBuilder;
    using GrumpkinVk = bb::VerifierCommitmentKey<curve::Grumpkin>;

    std::string vkPath = output_path + "/inst_vk"; // the vk of the last instance
    std::string accPath = output_path + "/pg_acc";
    std::string proofPath = output_path + "/client_ivc_proof";
    std::string translatorVkPath = output_path + "/translator_vk";
    std::string eccVkPath = output_path + "/ecc_vk";

    // Note: this could be decreased once we optimise the size of the ClientIVC recursiveve rifier
    init_bn254_crs(1 << 25);
    init_grumpkin_crs(1 << 18);

    // Read the proof  and verification data from given files
    auto proof = from_buffer<ClientIVC::Proof>(read_file(proofPath));
    std::shared_ptr<InstanceFlavor::VerificationKey> instance_vk = std::make_shared<InstanceFlavor::VerificationKey>(
        from_buffer<InstanceFlavor::VerificationKey>(read_file(vkPath)));
    std::shared_ptr<NativeInstance> verifier_accumulator =
        std::make_shared<NativeInstance>(from_buffer<NativeInstance>(read_file(accPath)));
    std::shared_ptr<TranslatorVk> translator_vk =
        std::make_shared<TranslatorVk>(from_buffer<TranslatorVk>(read_file(translatorVkPath)));
    std::shared_ptr<ECCVMVk> eccvm_vk = std::make_shared<ECCVMVk>(from_buffer<ECCVMVk>(read_file(eccVkPath)));
    // We don't serialise and deserialise the Grumkin SRS so initialise with circuit_size + 1 to be able to recursively
    // IPA. The + 1 is to satisfy IPA verification key requirements.
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/1025)
    eccvm_vk->pcs_verification_key = std::make_shared<GrumpkinVk>(eccvm_vk->circuit_size + 1);

    FoldVerifierInput fold_verifier_input{ verifier_accumulator, { instance_vk } };
    GoblinVerifierInput goblin_verifier_input{ eccvm_vk, translator_vk };
    VerifierInput input{ fold_verifier_input, goblin_verifier_input };
    auto builder = std::make_shared<Builder>();
    // Padding needed for sending the right number of public inputs
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/1048): INSECURE - make this tube proof actually use
    // these public inputs by turning proof into witnesses and call
    // set_public on each witness
    auto num_public_inputs = (size_t)proof.folding_proof[1];
    for (size_t i = 0; i < num_public_inputs; i++) {
        // We offset 3
        builder->add_public_variable(proof.folding_proof[i + 3]);
    }
    ClientIVC verifier{ builder, input };

    verifier.verify(proof);
    info("num gates in tube circuit: ", builder->get_num_gates());
    using Prover = UltraProver_<UltraFlavor>;
    using Verifier = UltraVerifier_<UltraFlavor>;
    Prover tube_prover{ *builder };
    auto tube_proof = tube_prover.construct_proof();
    std::string tubeProofPath = output_path + "/proof";
    write_file(tubeProofPath, to_buffer<true>(tube_proof));

    std::string tubeProofAsFieldsPath = output_path + "/proof_fields.json";
    auto proof_data = to_json(tube_proof);
    write_file(tubeProofAsFieldsPath, { proof_data.begin(), proof_data.end() });

    std::string tubeVkPath = output_path + "/vk";
    auto tube_verification_key =
        std::make_shared<typename UltraFlavor::VerificationKey>(tube_prover.instance->proving_key);
    write_file(tubeVkPath, to_buffer(tube_verification_key));

    std::string tubeAsFieldsVkPath = output_path + "/vk_fields.json";
    auto field_els = tube_verification_key->to_field_elements();
    info("verificaton key length in fields:", field_els.size());
    auto data = to_json(field_els);
    write_file(tubeAsFieldsVkPath, { data.begin(), data.end() });

    info("Native verification of the tube_proof");
    Verifier tube_verifier(tube_verification_key);
    bool verified = tube_verifier.verify_proof(tube_proof);
    info("Tube proof verification: ", verified);
}

/**
 * @brief Creates a proof for an ACIR circuit
 *
 * Communication:
 * - stdout: The proof is written to stdout as a byte array
 * - Filesystem: The proof is written to the path specified by outputPath
 *
 * @param bytecodePath Path to the file containing the serialized circuit
 * @param witnessPath Path to the file containing the serialized witness
 * @param recursive Whether to use recursive proof generation of non-recursive
 * @param outputPath Path to write the proof to
 */
void prove(const std::string& bytecodePath, const std::string& witnessPath, const std::string& outputPath)
{
    auto constraint_system = get_constraint_system(bytecodePath, /*honk_recursion=*/false);
    auto witness = get_witness(witnessPath);

    acir_proofs::AcirComposer acir_composer{ 0, verbose_logging };
    acir_composer.create_circuit(constraint_system, witness);
    init_bn254_crs(acir_composer.get_dyadic_circuit_size());
    acir_composer.init_proving_key();
    auto proof = acir_composer.create_proof();

    if (outputPath == "-") {
        writeRawBytesToStdout(proof);
        vinfo("proof written to stdout");
    } else {
        write_file(outputPath, proof);
        vinfo("proof written to: ", outputPath);
    }
}

/**
 * @brief Computes the number of Barretenberg specific gates needed to create a proof for the specific ACIR circuit.
 *
 * Communication:
 * - stdout: A JSON string of the number of ACIR opcodes and final backend circuit size.
 *
 * @param bytecodePath Path to the file containing the serialized circuit
 */
void gateCount(const std::string& bytecodePath, bool honk_recursion)
{
    // All circuit reports will be built into the string below
    std::string functions_string = "{\"functions\": [\n  ";
    auto constraint_systems = get_constraint_systems(bytecodePath, honk_recursion);
    size_t i = 0;
    for (auto constraint_system : constraint_systems) {
        acir_proofs::AcirComposer acir_composer(0, verbose_logging);
        acir_composer.create_circuit(constraint_system, {}, true);
        auto circuit_size = acir_composer.get_total_circuit_size();

        // Build individual circuit report
        std::string gates_per_opcode_str;
        for (size_t j = 0; j < constraint_system.gates_per_opcode.size(); j++) {
            gates_per_opcode_str += std::to_string(constraint_system.gates_per_opcode[j]);
            if (j != constraint_system.gates_per_opcode.size() - 1) {
                gates_per_opcode_str += ",";
            }
        }

        auto result_string = format("{\n        \"acir_opcodes\": ",
                                    constraint_system.num_acir_opcodes,
                                    ",\n        \"circuit_size\": ",
                                    circuit_size,
                                    ",\n        \"gates_per_opcode\": [",
                                    gates_per_opcode_str,
                                    "]\n  }");

        // Attach a comma if we still circuit reports to generate
        if (i != (constraint_systems.size() - 1)) {
            result_string = format(result_string, ",");
        }

        functions_string = format(functions_string, result_string);

        i++;
    }
    functions_string = format(functions_string, "\n]}");

    const char* jsonData = functions_string.c_str();
    size_t length = strlen(jsonData);
    std::vector<uint8_t> data(jsonData, jsonData + length);
    writeRawBytesToStdout(data);
}

/**
 * @brief Verifies a proof for an ACIR circuit
 *
 * Note: The fact that the proof was computed originally by parsing an ACIR circuit is not of importance
 * because this method uses the verification key to verify the proof.
 *
 * Communication:
 * - proc_exit: A boolean value is returned indicating whether the proof is valid.
 *   an exit code of 0 will be returned for success and 1 for failure.
 *
 * @param proof_path Path to the file containing the serialized proof
 * @param vk_path Path to the file containing the serialized verification key
 * @return true If the proof is valid
 * @return false If the proof is invalid
 */
bool verify(const std::string& proof_path, const std::string& vk_path)
{
    auto acir_composer = verifier_init();
    auto vk_data = from_buffer<plonk::verification_key_data>(read_file(vk_path));
    acir_composer.load_verification_key(std::move(vk_data));
    auto verified = acir_composer.verify_proof(read_file(proof_path));

    vinfo("verified: ", verified);
    return verified;
}

/**
 * @brief Writes a verification key for an ACIR circuit to a file
 *
 * Communication:
 * - stdout: The verification key is written to stdout as a byte array
 * - Filesystem: The verification key is written to the path specified by outputPath
 *
 * @param bytecodePath Path to the file containing the serialized circuit
 * @param outputPath Path to write the verification key to
 */
void write_vk(const std::string& bytecodePath, const std::string& outputPath)
{
    auto constraint_system = get_constraint_system(bytecodePath, false);
    acir_proofs::AcirComposer acir_composer{ 0, verbose_logging };
    acir_composer.create_circuit(constraint_system);
    init_bn254_crs(acir_composer.get_dyadic_circuit_size());
    acir_composer.init_proving_key();
    auto vk = acir_composer.init_verification_key();
    auto serialized_vk = to_buffer(*vk);
    if (outputPath == "-") {
        writeRawBytesToStdout(serialized_vk);
        vinfo("vk written to stdout");
    } else {
        write_file(outputPath, serialized_vk);
        vinfo("vk written to: ", outputPath);
    }
}

void write_pk(const std::string& bytecodePath, const std::string& outputPath)
{
    auto constraint_system = get_constraint_system(bytecodePath, /*honk_recursion=*/false);
    acir_proofs::AcirComposer acir_composer{ 0, verbose_logging };
    acir_composer.create_circuit(constraint_system);
    init_bn254_crs(acir_composer.get_dyadic_circuit_size());
    auto pk = acir_composer.init_proving_key();
    auto serialized_pk = to_buffer(*pk);

    if (outputPath == "-") {
        writeRawBytesToStdout(serialized_pk);
        vinfo("pk written to stdout");
    } else {
        write_file(outputPath, serialized_pk);
        vinfo("pk written to: ", outputPath);
    }
}

/**
 * @brief Writes a Solidity verifier contract for an ACIR circuit to a file
 *
 * Communication:
 * - stdout: The Solidity verifier contract is written to stdout as a string
 * - Filesystem: The Solidity verifier contract is written to the path specified by outputPath
 *
 * Note: The fact that the contract was computed is for an ACIR circuit is not of importance
 * because this method uses the verification key to compute the Solidity verifier contract
 *
 * @param output_path Path to write the contract to
 * @param vk_path Path to the file containing the serialized verification key
 */
void contract(const std::string& output_path, const std::string& vk_path)
{
    auto acir_composer = verifier_init();
    auto vk_data = from_buffer<plonk::verification_key_data>(read_file(vk_path));
    acir_composer.load_verification_key(std::move(vk_data));
    auto contract = acir_composer.get_solidity_verifier();

    if (output_path == "-") {
        writeStringToStdout(contract);
        vinfo("contract written to stdout");
    } else {
        write_file(output_path, { contract.begin(), contract.end() });
        vinfo("contract written to: ", output_path);
    }
}

/**
 * @brief Converts a proof from a byte array into a list of field elements
 *
 * Why is this needed?
 *
 * The proof computed by the non-recursive proof system is a byte array. This is fine since the proof will be
 * verified either natively or in a Solidity verifier. For the recursive proof system, the proof is verified in a
 * circuit where it is cheaper to work with field elements than byte arrays. This method converts the proof into a
 * list of field elements which can be used in the recursive proof system.
 *
 * This is an optimization which unfortunately leaks through the API. The repercussions of this are that users need
 * to convert proofs which are byte arrays to proofs which are lists of field elements, using the below method.
 *
 * Ideally, we find out what is the cost to convert this in the circuit and if it is not too expensive, we pass the
 * byte array directly to the circuit and convert it there. This also applies to the `vkAsFields` method.
 *
 * Communication:
 * - stdout: The proof as a list of field elements is written to stdout as a string
 * - Filesystem: The proof as a list of field elements is written to the path specified by outputPath
 *
 *
 * @param proof_path Path to the file containing the serialized proof
 * @param vk_path Path to the file containing the serialized verification key
 * @param output_path Path to write the proof to
 */
void proof_as_fields(const std::string& proof_path, std::string const& vk_path, const std::string& output_path)
{
    auto acir_composer = verifier_init();
    auto vk_data = from_buffer<plonk::verification_key_data>(read_file(vk_path));
    auto data = acir_composer.serialize_proof_into_fields(read_file(proof_path), vk_data.num_public_inputs);
    auto json = to_json(data);

    if (output_path == "-") {
        writeStringToStdout(json);
        vinfo("proof as fields written to stdout");
    } else {
        write_file(output_path, { json.begin(), json.end() });
        vinfo("proof as fields written to: ", output_path);
    }
}

/**
 * @brief Converts a verification key from a byte array into a list of field elements
 *
 * Why is this needed?
 * This follows the same rationale as `proofAsFields`.
 *
 * Communication:
 * - stdout: The verification key as a list of field elements is written to stdout as a string
 * - Filesystem: The verification key as a list of field elements is written to the path specified by outputPath
 *
 * @param vk_path Path to the file containing the serialized verification key
 * @param output_path Path to write the verification key to
 */
void vk_as_fields(const std::string& vk_path, const std::string& output_path)
{
    auto acir_composer = verifier_init();
    auto vk_data = from_buffer<plonk::verification_key_data>(read_file(vk_path));
    acir_composer.load_verification_key(std::move(vk_data));
    auto data = acir_composer.serialize_verification_key_into_fields();

    auto json = vk_to_json(data);
    if (output_path == "-") {
        writeStringToStdout(json);
        vinfo("vk as fields written to stdout");
    } else {
        write_file(output_path, { json.begin(), json.end() });
        vinfo("vk as fields written to: ", output_path);
    }
}

#ifndef DISABLE_AZTEC_VM
/**
 * @brief Writes an avm proof and corresponding (incomplete) verification key to files.
 *
 * Communication:
 * - Filesystem: The proof and vk are written to the paths output_path/proof and output_path/{vk, vk_fields.json}
 *
 * @param bytecode_path Path to the file containing the serialised bytecode
 * @param calldata_path Path to the file containing the serialised calldata (could be empty)
 * @param public_inputs_path Path to the file containing the serialised avm public inputs
 * @param hints_path Path to the file containing the serialised avm circuit hints
 * @param output_path Path (directory) to write the output proof and verification keys
 */
void avm_prove(const std::filesystem::path& bytecode_path,
               const std::filesystem::path& calldata_path,
               const std::filesystem::path& public_inputs_path,
               const std::filesystem::path& hints_path,
               const std::filesystem::path& output_path)
{
    std::vector<uint8_t> const bytecode = read_file(bytecode_path);
    std::vector<fr> const calldata = many_from_buffer<fr>(read_file(calldata_path));
    std::vector<fr> const public_inputs_vec = many_from_buffer<fr>(read_file(public_inputs_path));
    auto const avm_hints = bb::avm_trace::ExecutionHints::from(read_file(hints_path));

    vinfo("bytecode size: ", bytecode.size());
    vinfo("calldata size: ", calldata.size());
    vinfo("public_inputs size: ", public_inputs_vec.size());
    vinfo("hints.storage_value_hints size: ", avm_hints.storage_value_hints.size());
    vinfo("hints.note_hash_exists_hints size: ", avm_hints.note_hash_exists_hints.size());
    vinfo("hints.nullifier_exists_hints size: ", avm_hints.nullifier_exists_hints.size());
    vinfo("hints.l1_to_l2_message_exists_hints size: ", avm_hints.l1_to_l2_message_exists_hints.size());
    vinfo("hints.externalcall_hints size: ", avm_hints.externalcall_hints.size());
    vinfo("hints.contract_instance_hints size: ", avm_hints.contract_instance_hints.size());

    // Hardcoded circuit size for now, with enough to support 16-bit range checks
    init_bn254_crs(1 << 17);

    // Prove execution and return vk
    auto const [verification_key, proof] =
        avm_trace::Execution::prove(bytecode, calldata, public_inputs_vec, avm_hints);
    vinfo("------- PROVING DONE -------");

    // TODO(ilyas): <#4887>: Currently we only need these two parts of the vk, look into pcs_verification key reqs
    std::vector<uint64_t> vk_vector = { verification_key.circuit_size, verification_key.num_public_inputs };
    std::vector<fr> vk_as_fields = { verification_key.circuit_size, verification_key.num_public_inputs };
    std::string vk_json = vk_to_json(vk_as_fields);

    const auto proof_path = output_path / "proof";
    const auto vk_path = output_path / "vk";
    const auto vk_fields_path = output_path / "vk_fields.json";

    write_file(proof_path, to_buffer(proof));
    vinfo("proof written to: ", proof_path);
    write_file(vk_path, to_buffer(vk_vector));
    vinfo("vk written to: ", vk_path);
    write_file(vk_fields_path, { vk_json.begin(), vk_json.end() });
    vinfo("vk as fields written to: ", vk_fields_path);

#ifdef AVM_TRACK_STATS
    info("------- STATS -------");
    const auto& stats = avm_trace::Stats::get();
    info(stats.to_string());
#endif
}

/**
 * @brief Verifies an avm proof and writes the result to stdout
 *
 * Communication:
 * - proc_exit: A boolean value is returned indicating whether the proof is valid.
 *   an exit code of 0 will be returned for success and 1 for failure.
 *
 * @param proof_path Path to the file containing the serialized proof
 * @param vk_path Path to the file containing the serialized verification key
 * @return true If the proof is valid
 * @return false If the proof is invalid
 */
bool avm_verify(const std::filesystem::path& proof_path, const std::filesystem::path& vk_path)
{
    std::vector<fr> const proof = many_from_buffer<fr>(read_file(proof_path));
    std::vector<uint8_t> vk_bytes = read_file(vk_path);
    auto circuit_size = from_buffer<size_t>(vk_bytes, 0);
    auto num_public_inputs = from_buffer<size_t>(vk_bytes, sizeof(size_t));
    auto vk = AvmFlavor::VerificationKey(circuit_size, num_public_inputs);

    const bool verified = avm_trace::Execution::verify(vk, proof);
    vinfo("verified: ", verified);
    return verified;
}
#endif

/**
 * @brief Create a Honk a prover from program bytecode and an optional witness
 *
 * @tparam Flavor
 * @param bytecodePath
 * @param witnessPath
 * @return UltraProver_<Flavor>
 */
template <typename Flavor>
UltraProver_<Flavor> compute_valid_prover(const std::string& bytecodePath, const std::string& witnessPath)
{
    using Builder = Flavor::CircuitBuilder;
    using Prover = UltraProver_<Flavor>;

    bool honk_recursion = false;
    if constexpr (IsAnyOf<Flavor, UltraFlavor>) {
        honk_recursion = true;
    }
    auto constraint_system = get_constraint_system(bytecodePath, honk_recursion);
    acir_format::WitnessVector witness = {};
    if (!witnessPath.empty()) {
        witness = get_witness(witnessPath);
    }

    auto builder = acir_format::create_circuit<Builder>(constraint_system, 0, witness, honk_recursion);

    auto num_extra_gates = builder.get_num_gates_added_to_ensure_nonzero_polynomials();
    size_t srs_size = builder.get_circuit_subgroup_size(builder.get_total_circuit_size() + num_extra_gates);
    init_bn254_crs(srs_size);

    Prover prover{ builder };
    return prover;
}

/**
 * @brief Creates a proof for an ACIR circuit
 *
 * Communication:
 * - stdout: The proof is written to stdout as a byte array
 * - Filesystem: The proof is written to the path specified by outputPath
 *
 * @param bytecodePath Path to the file containing the serialized circuit
 * @param witnessPath Path to the file containing the serialized witness
 * @param outputPath Path to write the proof to
 */
template <IsUltraFlavor Flavor>
void prove_honk(const std::string& bytecodePath, const std::string& witnessPath, const std::string& outputPath)
{
    // using Builder = Flavor::CircuitBuilder;
    using Prover = UltraProver_<Flavor>;

    // Construct Honk proof
    Prover prover = compute_valid_prover<Flavor>(bytecodePath, witnessPath);
    auto proof = prover.construct_proof();

    if (outputPath == "-") {
        writeRawBytesToStdout(to_buffer</*include_size=*/true>(proof));
        vinfo("proof written to stdout");
    } else {
        write_file(outputPath, to_buffer</*include_size=*/true>(proof));
        vinfo("proof written to: ", outputPath);
    }
}

/**
 * @brief Verifies a proof for an ACIR circuit
 *
 * Note: The fact that the proof was computed originally by parsing an ACIR circuit is not of importance
 * because this method uses the verification key to verify the proof.
 *
 * Communication:
 * - proc_exit: A boolean value is returned indicating whether the proof is valid.
 *   an exit code of 0 will be returned for success and 1 for failure.
 *
 * @param proof_path Path to the file containing the serialized proof
 * @param vk_path Path to the file containing the serialized verification key
 * @return true If the proof is valid
 * @return false If the proof is invalid
 */
template <IsUltraFlavor Flavor> bool verify_honk(const std::string& proof_path, const std::string& vk_path)
{
    using VerificationKey = Flavor::VerificationKey;
    using Verifier = UltraVerifier_<Flavor>;
    using VerifierCommitmentKey = bb::VerifierCommitmentKey<curve::BN254>;

    auto g2_data = get_bn254_g2_data(CRS_PATH);
    srs::init_crs_factory({}, g2_data);
    auto proof = from_buffer<std::vector<bb::fr>>(read_file(proof_path));
    auto vk = std::make_shared<VerificationKey>(from_buffer<VerificationKey>(read_file(vk_path)));
    vk->pcs_verification_key = std::make_shared<VerifierCommitmentKey>();
    Verifier verifier{ vk };

    bool verified = verifier.verify_proof(proof);

    vinfo("verified: ", verified);
    return verified;
}

/**
 * @brief Writes a Honk verification key for an ACIR circuit to a file
 *
 * Communication:
 * - stdout: The verification key is written to stdout as a byte array
 * - Filesystem: The verification key is written to the path specified by outputPath
 *
 * @param bytecodePath Path to the file containing the serialized circuit
 * @param outputPath Path to write the verification key to
 */
template <IsUltraFlavor Flavor> void write_vk_honk(const std::string& bytecodePath, const std::string& outputPath)
{
    using Prover = UltraProver_<Flavor>;
    using ProverInstance = ProverInstance_<Flavor>;
    using VerificationKey = Flavor::VerificationKey;

    Prover prover = compute_valid_prover<Flavor>(bytecodePath, "");
    ProverInstance& prover_inst = *prover.instance;
    VerificationKey vk(
        prover_inst.proving_key); // uses a partial form of the proving key which only has precomputed entities

    auto serialized_vk = to_buffer(vk);
    if (outputPath == "-") {
        writeRawBytesToStdout(serialized_vk);
        vinfo("vk written to stdout");
    } else {
        write_file(outputPath, serialized_vk);
        vinfo("vk written to: ", outputPath);
    }
}

/**
 * @brief Outputs proof as vector of field elements in readable format.
 *
 * Communication:
 * - stdout: The proof as a list of field elements is written to stdout as a string
 * - Filesystem: The proof as a list of field elements is written to the path specified by outputPath
 *
 *
 * @param proof_path Path to the file containing the serialized proof
 * @param output_path Path to write the proof to
 */
void proof_as_fields_honk(const std::string& proof_path, const std::string& output_path)
{
    auto proof = from_buffer<std::vector<bb::fr>>(read_file(proof_path));
    auto json = to_json(proof);

    if (output_path == "-") {
        writeStringToStdout(json);
        vinfo("proof as fields written to stdout");
    } else {
        write_file(output_path, { json.begin(), json.end() });
        vinfo("proof as fields written to: ", output_path);
    }
}

/**
 * @brief Converts a verification key from a byte array into a list of field elements.
 *
 * Why is this needed?
 * This follows the same rationale as `proofAsFields`.
 *
 * Communication:
 * - stdout: The verification key as a list of field elements is written to stdout as a string
 * - Filesystem: The verification key as a list of field elements is written to the path specified by outputPath
 *
 * @param vk_path Path to the file containing the serialized verification key
 * @param output_path Path to write the verification key to
 */
template <IsUltraFlavor Flavor> void vk_as_fields_honk(const std::string& vk_path, const std::string& output_path)
{
    using VerificationKey = Flavor::VerificationKey;

    auto verification_key = std::make_shared<VerificationKey>(from_buffer<VerificationKey>(read_file(vk_path)));
    std::vector<bb::fr> data = verification_key->to_field_elements();
    auto json = honk_vk_to_json(data);
    if (output_path == "-") {
        writeStringToStdout(json);
        vinfo("vk as fields written to stdout");
    } else {
        write_file(output_path, { json.begin(), json.end() });
        vinfo("vk as fields written to: ", output_path);
    }
}

/**
 * @brief Creates a proof for an ACIR circuit, outputs the proof and verification key in binary and 'field' format
 *
 * Communication:
 * - Filesystem: The proof is written to the path specified by outputPath
 *
 * @param bytecodePath Path to the file containing the serialized circuit
 * @param witnessPath Path to the file containing the serialized witness
 * @param outputPath Directory into which we write the proof and verification key data
 */
void prove_output_all(const std::string& bytecodePath, const std::string& witnessPath, const std::string& outputPath)
{
    auto constraint_system = get_constraint_system(bytecodePath, /*honk_recursion=*/false);
    auto witness = get_witness(witnessPath);

    acir_proofs::AcirComposer acir_composer{ 0, verbose_logging };
    acir_composer.create_circuit(constraint_system, witness);
    init_bn254_crs(acir_composer.get_dyadic_circuit_size());
    acir_composer.init_proving_key();
    auto proof = acir_composer.create_proof();

    // We have been given a directory, we will write the proof and verification key
    // into the directory in both 'binary' and 'fields' formats
    std::string vkOutputPath = outputPath + "/vk";
    std::string proofPath = outputPath + "/proof";
    std::string vkFieldsOutputPath = outputPath + "/vk_fields.json";
    std::string proofFieldsPath = outputPath + "/proof_fields.json";

    std::shared_ptr<bb::plonk::verification_key> vk = acir_composer.init_verification_key();

    // Write the 'binary' proof
    write_file(proofPath, proof);
    vinfo("proof written to: ", proofPath);

    // Write the proof as fields
    auto proofAsFields = acir_composer.serialize_proof_into_fields(proof, vk->as_data().num_public_inputs);
    std::string proofJson = to_json(proofAsFields);
    write_file(proofFieldsPath, { proofJson.begin(), proofJson.end() });
    info("proof as fields written to: ", proofFieldsPath);

    // Write the vk as binary
    auto serialized_vk = to_buffer(*vk);
    write_file(vkOutputPath, serialized_vk);
    vinfo("vk written to: ", vkOutputPath);

    // Write the vk as fields
    auto data = acir_composer.serialize_verification_key_into_fields();
    std::string vk_json = vk_to_json(data);
    write_file(vkFieldsOutputPath, { vk_json.begin(), vk_json.end() });
    vinfo("vk as fields written to: ", vkFieldsOutputPath);
}

/**
 * @brief Creates a Honk proof for an ACIR circuit, outputs the proof and verification key in binary and 'field' format
 *
 * Communication:
 * - Filesystem: The proof is written to the path specified by outputPath
 *
 * @param bytecodePath Path to the file containing the serialized circuit
 * @param witnessPath Path to the file containing the serialized witness
 * @param outputPath Directory into which we write the proof and verification key data
 */
template <IsUltraFlavor Flavor>
void prove_honk_output_all(const std::string& bytecodePath,
                           const std::string& witnessPath,
                           const std::string& outputPath)
{
    using Builder = Flavor::CircuitBuilder;
    using Prover = UltraProver_<Flavor>;
    using VerificationKey = Flavor::VerificationKey;

    bool honk_recursion = false;
    if constexpr (IsAnyOf<Flavor, UltraFlavor>) {
        honk_recursion = true;
    }

    auto constraint_system = get_constraint_system(bytecodePath, honk_recursion);
    auto witness = get_witness(witnessPath);

    auto builder = acir_format::create_circuit<Builder>(constraint_system, 0, witness, honk_recursion);

    auto num_extra_gates = builder.get_num_gates_added_to_ensure_nonzero_polynomials();
    size_t srs_size = builder.get_circuit_subgroup_size(builder.get_total_circuit_size() + num_extra_gates);
    init_bn254_crs(srs_size);

    // Construct Honk proof
    Prover prover{ builder };
    auto proof = prover.construct_proof();

    // We have been given a directory, we will write the proof and verification key
    // into the directory in both 'binary' and 'fields' formats
    std::string vkOutputPath = outputPath + "/vk";
    std::string proofPath = outputPath + "/proof";
    std::string vkFieldsOutputPath = outputPath + "/vk_fields.json";
    std::string proofFieldsPath = outputPath + "/proof_fields.json";

    VerificationKey vk(
        prover.instance->proving_key); // uses a partial form of the proving key which only has precomputed entities

    // Write the 'binary' proof
    write_file(proofPath, to_buffer</*include_size=*/true>(proof));
    vinfo("binary proof written to: ", proofPath);

    // Write the proof as fields
    std::string proofJson = to_json(proof);
    write_file(proofFieldsPath, { proofJson.begin(), proofJson.end() });
    vinfo("proof as fields written to: ", proofFieldsPath);

    // Write the vk as binary
    auto serialized_vk = to_buffer(vk);
    write_file(vkOutputPath, serialized_vk);
    vinfo("vk written to: ", vkOutputPath);

    // Write the vk as fields
    std::vector<bb::fr> vk_data = vk.to_field_elements();
    auto vk_json = honk_vk_to_json(vk_data);
    write_file(vkFieldsOutputPath, { vk_json.begin(), vk_json.end() });
    vinfo("vk as fields written to: ", vkFieldsOutputPath);
}

bool flag_present(std::vector<std::string>& args, const std::string& flag)
{
    return std::find(args.begin(), args.end(), flag) != args.end();
}

std::string get_option(std::vector<std::string>& args, const std::string& option, const std::string& defaultValue)
{
    auto itr = std::find(args.begin(), args.end(), option);
    return (itr != args.end() && std::next(itr) != args.end()) ? *(std::next(itr)) : defaultValue;
}

int main(int argc, char* argv[])
{
    try {
        std::vector<std::string> args(argv + 1, argv + argc);
        debug_logging = flag_present(args, "-d") || flag_present(args, "--debug_logging");
        verbose_logging = debug_logging || flag_present(args, "-v") || flag_present(args, "--verbose_logging");
        if (args.empty()) {
            std::cerr << "No command provided.\n";
            return 1;
        }

        std::string command = args[0];
        std::string bytecode_path = get_option(args, "-b", "./target/program.json");
        std::string witness_path = get_option(args, "-w", "./target/witness.gz");
        std::string proof_path = get_option(args, "-p", "./proofs/proof");
        std::string vk_path = get_option(args, "-k", "./target/vk");
        std::string pk_path = get_option(args, "-r", "./target/pk");
        bool honk_recursion = flag_present(args, "-h");
        CRS_PATH = get_option(args, "-c", CRS_PATH);

        // Skip CRS initialization for any command which doesn't require the CRS.
        if (command == "--version") {
            writeStringToStdout(BB_VERSION);
            return 0;
        }
        if (command == "prove_and_verify") {
            return proveAndVerify(bytecode_path, witness_path) ? 0 : 1;
        }
        if (command == "prove_and_verify_ultra_honk") {
            return proveAndVerifyHonk<UltraFlavor>(bytecode_path, witness_path) ? 0 : 1;
        }
        if (command == "prove_and_verify_mega_honk") {
            return proveAndVerifyHonk<MegaFlavor>(bytecode_path, witness_path) ? 0 : 1;
        }
        if (command == "prove_and_verify_ultra_honk_program") {
            return proveAndVerifyHonkProgram<UltraFlavor>(bytecode_path, witness_path) ? 0 : 1;
        }
        if (command == "prove_and_verify_mega_honk_program") {
            return proveAndVerifyHonkProgram<MegaFlavor>(bytecode_path, witness_path) ? 0 : 1;
        }
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/1050) we need a verify_client_ivc bb cli command
        // TODO(#7371): remove this
        if (command == "client_ivc_prove_output_all_msgpack") {
            std::filesystem::path output_dir = get_option(args, "-o", "./target");
            client_ivc_prove_output_all_msgpack(bytecode_path, witness_path, output_dir);
            return 0;
        }
        if (command == "verify_client_ivc") {
            std::filesystem::path output_dir = get_option(args, "-o", "./target");
            std::filesystem::path client_ivc_proof_path = output_dir / "client_ivc_proof";
            std::filesystem::path accumulator_path = output_dir / "pg_acc";
            std::filesystem::path final_vk_path = output_dir / "inst_vk";
            std::filesystem::path eccvm_vk_path = output_dir / "ecc_vk";
            std::filesystem::path translator_vk_path = output_dir / "translator_vk";

            return verify_client_ivc(
                       client_ivc_proof_path, accumulator_path, final_vk_path, eccvm_vk_path, translator_vk_path)
                       ? 0
                       : 1;
        }
        if (command == "fold_and_verify_program") {
            return foldAndVerifyProgram(bytecode_path, witness_path) ? 0 : 1;
        }

        if (command == "prove") {
            std::string output_path = get_option(args, "-o", "./proofs/proof");
            prove(bytecode_path, witness_path, output_path);
        } else if (command == "prove_output_all") {
            std::string output_path = get_option(args, "-o", "./proofs");
            prove_output_all(bytecode_path, witness_path, output_path);
        } else if (command == "prove_ultra_honk_output_all") {
            std::string output_path = get_option(args, "-o", "./proofs");
            prove_honk_output_all<UltraFlavor>(bytecode_path, witness_path, output_path);
        } else if (command == "prove_mega_honk_output_all") {
            std::string output_path = get_option(args, "-o", "./proofs");
            prove_honk_output_all<MegaFlavor>(bytecode_path, witness_path, output_path);
        } else if (command == "client_ivc_prove_output_all") {
            std::string output_path = get_option(args, "-o", "./target");
            client_ivc_prove_output_all(bytecode_path, witness_path, output_path);
        } else if (command == "prove_tube") {
            std::string output_path = get_option(args, "-o", "./target");
            prove_tube(output_path);
        } else if (command == "verify_tube") {
            std::string output_path = get_option(args, "-o", "./target");
            auto tube_proof_path = output_path + "/proof";
            auto tube_vk_path = output_path + "/vk";
            return verify_honk<UltraFlavor>(tube_proof_path, tube_vk_path) ? 0 : 1;
        } else if (command == "gates") {
            gateCount(bytecode_path, honk_recursion);
        } else if (command == "verify") {
            return verify(proof_path, vk_path) ? 0 : 1;
        } else if (command == "contract") {
            std::string output_path = get_option(args, "-o", "./target/contract.sol");
            contract(output_path, vk_path);
        } else if (command == "write_vk") {
            std::string output_path = get_option(args, "-o", "./target/vk");
            write_vk(bytecode_path, output_path);
        } else if (command == "write_pk") {
            std::string output_path = get_option(args, "-o", "./target/pk");
            write_pk(bytecode_path, output_path);
        } else if (command == "proof_as_fields") {
            std::string output_path = get_option(args, "-o", proof_path + "_fields.json");
            proof_as_fields(proof_path, vk_path, output_path);
        } else if (command == "vk_as_fields") {
            std::string output_path = get_option(args, "-o", vk_path + "_fields.json");
            vk_as_fields(vk_path, output_path);
#ifndef DISABLE_AZTEC_VM
        } else if (command == "avm_prove") {
            std::filesystem::path avm_bytecode_path = get_option(args, "--avm-bytecode", "./target/avm_bytecode.bin");
            std::filesystem::path avm_calldata_path = get_option(args, "--avm-calldata", "./target/avm_calldata.bin");
            std::filesystem::path avm_public_inputs_path =
                get_option(args, "--avm-public-inputs", "./target/avm_public_inputs.bin");
            std::filesystem::path avm_hints_path = get_option(args, "--avm-hints", "./target/avm_hints.bin");
            // This outputs both files: proof and vk, under the given directory.
            std::filesystem::path output_path = get_option(args, "-o", "./proofs");
            extern std::filesystem::path avm_dump_trace_path;
            avm_dump_trace_path = get_option(args, "--avm-dump-trace", "");
            avm_prove(avm_bytecode_path, avm_calldata_path, avm_public_inputs_path, avm_hints_path, output_path);
        } else if (command == "avm_verify") {
            return avm_verify(proof_path, vk_path) ? 0 : 1;
#endif
        } else if (command == "prove_ultra_honk") {
            std::string output_path = get_option(args, "-o", "./proofs/proof");
            prove_honk<UltraFlavor>(bytecode_path, witness_path, output_path);
        } else if (command == "verify_ultra_honk") {
            return verify_honk<UltraFlavor>(proof_path, vk_path) ? 0 : 1;
        } else if (command == "write_vk_ultra_honk") {
            std::string output_path = get_option(args, "-o", "./target/vk");
            write_vk_honk<UltraFlavor>(bytecode_path, output_path);
        } else if (command == "prove_mega_honk") {
            std::string output_path = get_option(args, "-o", "./proofs/proof");
            prove_honk<MegaFlavor>(bytecode_path, witness_path, output_path);
        } else if (command == "verify_mega_honk") {
            return verify_honk<MegaFlavor>(proof_path, vk_path) ? 0 : 1;
        } else if (command == "write_vk_mega_honk") {
            std::string output_path = get_option(args, "-o", "./target/vk");
            write_vk_honk<MegaFlavor>(bytecode_path, output_path);
        } else if (command == "proof_as_fields_honk") {
            std::string output_path = get_option(args, "-o", proof_path + "_fields.json");
            proof_as_fields_honk(proof_path, output_path);
        } else if (command == "vk_as_fields_ultra_honk") {
            std::string output_path = get_option(args, "-o", vk_path + "_fields.json");
            vk_as_fields_honk<UltraFlavor>(vk_path, output_path);
        } else if (command == "vk_as_fields_mega_honk") {
            std::string output_path = get_option(args, "-o", vk_path + "_fields.json");
            vk_as_fields_honk<MegaFlavor>(vk_path, output_path);
        } else {
            std::cerr << "Unknown command: " << command << "\n";
            return 1;
        }
    } catch (std::runtime_error const& err) {
        std::cerr << err.what() << std::endl;
        return 1;
    }
}
