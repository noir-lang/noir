#pragma once
#include "join_split/join_split.hpp"
#include "mock/mock_circuit.hpp"
#include "../constants.hpp"
#include <fstream>
#include <sys/stat.h>
#include <common/timer.hpp>
#include <proof_system/proving_key/serialize.hpp>
#include <filesystem>

#define GET_COMPOSER_NAME_STRING(composer)                                                                             \
    (typeid(composer) == typeid(waffle::StandardComposer)                                                              \
         ? "StandardPlonk"                                                                                             \
         : typeid(composer) == typeid(waffle::TurboComposer) ? "TurboPlonk" : "NULLPlonk")

namespace rollup {
namespace proofs {

struct circuit_data {
    circuit_data()
        : num_gates(0)
    {}

    std::shared_ptr<waffle::ReferenceStringFactory> srs;
    std::shared_ptr<waffle::proving_key> proving_key;
    std::shared_ptr<waffle::verification_key> verification_key;
    size_t num_gates;
    std::vector<uint8_t> padding_proof;
    bool mock;
};

namespace {
inline bool exists(std::string const& path)
{
    struct stat st;
    return (stat(path.c_str(), &st) != -1);
}
} // namespace

template <typename ComposerType, typename F>
circuit_data get_circuit_data(std::string const& name,
                              std::string const& path_name,
                              std::shared_ptr<waffle::ReferenceStringFactory> const& srs,
                              std::string const& key_path,
                              bool compute,
                              bool save,
                              bool load,
                              bool pk,
                              bool vk,
                              bool padding,
                              bool mock,
                              F const& build_circuit,
                              std::string const name_suffix_for_benchmarks = "")
{
    circuit_data data;
    data.srs = srs;
    data.mock = mock;
    ComposerType composer(srs);
    ComposerType mock_proof_composer(srs);
    BenchmarkInfoCollator benchmark_collator;

    auto circuit_key_path = key_path + "/" + path_name;
    auto pk_path = circuit_key_path + "/proving_key/proving_key";
    auto vk_path = circuit_key_path + "/verification_key";
    auto padding_path = circuit_key_path + "/padding_proof";

    // If we're missing required data, and compute is enabled, or if
    // compute is enabled and load is disabled, build the circuit.
    if (((!exists(pk_path) || !exists(vk_path) || (!exists(padding_path) && padding)) && compute) ||
        (compute && !load)) {
        info(name, ": Building circuit...");
        Timer timer;
        build_circuit(composer);

        benchmark_collator.benchmark_info_deferred(GET_COMPOSER_NAME_STRING(ComposerType),
                                                   "Core",
                                                   name + name_suffix_for_benchmarks,
                                                   "Build time",
                                                   timer.toString());
        benchmark_collator.benchmark_info_deferred(GET_COMPOSER_NAME_STRING(ComposerType),
                                                   "Core",
                                                   name + name_suffix_for_benchmarks,
                                                   "Gates",
                                                   composer.get_num_gates());
        info(name, ": Circuit built in: ", timer.toString(), "s");
        info(name, ": Circuit size: ", composer.get_num_gates());
        if (mock) {
            auto public_inputs = composer.get_public_inputs();
            mock::mock_circuit(mock_proof_composer, public_inputs);
            info(name, ": Mock circuit size: ", mock_proof_composer.get_num_gates());
            benchmark_collator.benchmark_info_deferred(GET_COMPOSER_NAME_STRING(ComposerType),
                                                       "Core",
                                                       name + name_suffix_for_benchmarks,
                                                       "Mock Gates",
                                                       composer.get_num_gates());
        }
    }

    // If we're saving data, create the circuit data directory.
    if (save) {
        std::filesystem::create_directories(key_path.c_str());
        std::filesystem::create_directories(circuit_key_path.c_str());
    }

    if (pk) {
        auto pk_dir = circuit_key_path + "/proving_key";
        if (exists(pk_path) && load) {
            info(name, ": Loading proving key: ", pk_path);
            auto pk_stream = std::ifstream(pk_path);
            waffle::proving_key_data pk_data;
            read_mmap(pk_stream, pk_dir, pk_data);
            data.proving_key =
                std::make_shared<waffle::proving_key>(std::move(pk_data), srs->get_prover_crs(pk_data.n + 1));
            data.num_gates = pk_data.n;
            info(name, ": Circuit size 2^n: ", data.num_gates);
            benchmark_collator.benchmark_info_deferred(GET_COMPOSER_NAME_STRING(ComposerType),
                                                       "Core",
                                                       name + name_suffix_for_benchmarks,
                                                       "Gates 2^n",
                                                       data.num_gates);
        } else if (compute) {
            Timer timer;
            info(name, ": Computing proving key...");

            if (!mock) {
                data.num_gates = composer.get_num_gates();
                data.proving_key = composer.compute_proving_key();
                info(name, ": Circuit size 2^n: ", data.proving_key->n);

                benchmark_collator.benchmark_info_deferred(GET_COMPOSER_NAME_STRING(ComposerType),
                                                           "Core",
                                                           name + name_suffix_for_benchmarks,
                                                           "Gates 2^n",
                                                           data.proving_key->n);
            } else {
                data.num_gates = mock_proof_composer.get_num_gates();
                data.proving_key = mock_proof_composer.compute_proving_key();
                info(name, ": Mock circuit size 2^n: ", data.proving_key->n);
                benchmark_collator.benchmark_info_deferred(GET_COMPOSER_NAME_STRING(ComposerType),
                                                           "Core",
                                                           name + name_suffix_for_benchmarks,
                                                           "Mock Gates 2^n",
                                                           data.proving_key->n);
            }

            info(name, ": Proving key computed in ", timer.toString(), "s");

            benchmark_collator.benchmark_info_deferred(GET_COMPOSER_NAME_STRING(ComposerType),
                                                       "Core",
                                                       name + name_suffix_for_benchmarks,
                                                       "Proving key computed in",
                                                       timer.toString());
            if (save) {
                info(name, ": Saving proving key...");
                std::filesystem::create_directories(pk_dir.c_str());
                Timer write_timer;
                std::ofstream os(pk_path);
                write_mmap(os, pk_dir, *data.proving_key);
                if (!os.good()) {
                    throw_or_abort(format("Failed to write: ", pk_path));
                }
                info(name, ": Saved in ", write_timer.toString(), "s");
            }
        }
    }

    if (vk) {
        if (exists(vk_path) && load) {
            info(name, ": Loading verification key from: ", vk_path);
            auto vk_stream = std::ifstream(vk_path);
            waffle::verification_key_data vk_data;
            read(vk_stream, vk_data);
            data.verification_key =
                std::make_shared<waffle::verification_key>(std::move(vk_data), data.srs->get_verifier_crs());
            info(name, ": Verification key hash: ", data.verification_key->sha256_hash());
            benchmark_collator.benchmark_info_deferred(GET_COMPOSER_NAME_STRING(ComposerType),
                                                       "Core",
                                                       name + name_suffix_for_benchmarks,
                                                       "Verification key hash",
                                                       data.verification_key->sha256_hash());
        } else if (compute) {
            info(name, ": Computing verification key...");
            Timer timer;

            if (!mock) {
                data.verification_key = composer.compute_verification_key();
            } else {
                data.verification_key = mock_proof_composer.compute_verification_key();
            }
            info(name, ": Computed verification key in ", timer.toString(), "s");

            benchmark_collator.benchmark_info_deferred(GET_COMPOSER_NAME_STRING(ComposerType),
                                                       "Core",
                                                       name + name_suffix_for_benchmarks,
                                                       "Verification key computed in",
                                                       timer.toString());
            info(name, ": Verification key hash: ", data.verification_key->sha256_hash());
            benchmark_collator.benchmark_info_deferred(GET_COMPOSER_NAME_STRING(ComposerType),
                                                       "Core",
                                                       name + name_suffix_for_benchmarks,
                                                       "Verification key hash",
                                                       data.verification_key->sha256_hash());

            if (save) {
                std::ofstream os(vk_path);
                write(os, *data.verification_key);
                if (!os.good()) {
                    throw_or_abort(format("Failed to write: ", vk_path));
                }
            }
        }
    }

    if (padding) {
        if (exists(padding_path) && load) {
            info(name, ": Loading padding proof from: ", padding_path);
            std::ifstream is(padding_path);
            std::vector<uint8_t> proof((std::istreambuf_iterator<char>(is)), std::istreambuf_iterator<char>());
            data.padding_proof = proof;
        } else if (data.proving_key) {
            info(name, ": Computing padding proof...");

            if (composer.failed()) {
                info(name, ": Composer logic failed: ", composer.err());
                info(name, ": Warning, padding proof can only be used to aid upstream pk construction!");
            }

            Timer timer;
            if (!mock) {
                auto prover = composer.create_unrolled_prover();
                auto proof = prover.construct_proof();
                data.padding_proof = proof.proof_data;
                data.num_gates = composer.get_num_gates();
                info(name, ": Circuit size: ", data.num_gates);
                auto verifier = composer.create_unrolled_verifier();
                info(name, ": Padding verified: ", verifier.verify_proof(proof));
            } else {
                auto prover = mock_proof_composer.create_unrolled_prover();
                auto proof = prover.construct_proof();
                data.padding_proof = proof.proof_data;
                data.num_gates = mock_proof_composer.get_num_gates();
                info(name, ": Mock circuit size: ", data.num_gates);
                auto verifier = mock_proof_composer.create_unrolled_verifier();
                info(name, ": Padding verified: ", verifier.verify_proof(proof));
            }
            info(name, ": Padding proof computed in ", timer.toString(), "s");
            benchmark_collator.benchmark_info_deferred(GET_COMPOSER_NAME_STRING(ComposerType),
                                                       "Core",
                                                       name + name_suffix_for_benchmarks,
                                                       "Padding proof computed in",
                                                       timer.toString());

            if (save) {
                std::ofstream os(padding_path);
                os.write((char*)data.padding_proof.data(), (std::streamsize)data.padding_proof.size());
                if (!os.good()) {
                    throw_or_abort(format("Failed to write: ", padding_path));
                }
            }
        }
    }

    return data;
}

} // namespace proofs
} // namespace rollup
