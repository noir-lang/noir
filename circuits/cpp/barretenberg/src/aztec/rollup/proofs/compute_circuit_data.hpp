#pragma once
#include "join_split/join_split.hpp"
#include "../constants.hpp"
#include <fstream>
#include <sys/stat.h>
#include <common/timer.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>

namespace rollup {
namespace proofs {

using namespace plonk::stdlib::types::turbo;

struct circuit_data {
    circuit_data()
        : num_gates(0)
    {}

    std::shared_ptr<waffle::proving_key> proving_key;
    std::shared_ptr<waffle::verification_key> verification_key;
    size_t num_gates;
    std::vector<uint8_t> padding_proof;
};

namespace {
inline bool exists(std::string const& path)
{
    struct stat st;
    return (stat(path.c_str(), &st) != -1);
}
} // namespace

template <typename F>
circuit_data get_circuit_data(std::string const& name,
                              std::string const& srs_path,
                              std::string const& key_path,
                              bool compute,
                              bool save,
                              bool load,
                              F const& build_circuit)
{
    circuit_data data;
    auto circuit_key_path = key_path + "/" + name;
    auto crs = std::make_unique<waffle::FileReferenceStringFactory>(srs_path);
    Composer composer = Composer(srs_path);

    if (save) {
        mkdir(key_path.c_str(), 0700);
        mkdir(circuit_key_path.c_str(), 0700);
    }

    {
        auto pk_dir = circuit_key_path + "/proving_key";
        mkdir(pk_dir.c_str(), 0700);
        auto pk_path = circuit_key_path + "/proving_key/proving_key";
        if (exists(pk_path) && load) {
            std::cerr << "Loading proving key: " << pk_path << std::endl;
            auto pk_stream = std::ifstream(pk_path);
            waffle::proving_key_data pk_data;
            read_mmap(pk_stream, pk_dir, pk_data);
            data.proving_key =
                std::make_shared<waffle::proving_key>(std::move(pk_data), crs->get_prover_crs(pk_data.n));
            data.num_gates = pk_data.n;
            std::cerr << "Circuit size (nearest 2^n): " << data.num_gates << std::endl;
        } else if (compute) {
            Timer timer;
            std::cerr << "Computing proving key..." << std::endl;

            build_circuit(composer);

            data.num_gates = composer.get_num_gates();
            std::cerr << "Circuit size: " << data.num_gates << std::endl;

            data.proving_key = composer.compute_proving_key();
            std::cerr << "Done: " << timer.toString() << "s" << std::endl;

            if (save) {
                std::cerr << "Saving proving key..." << std::endl;
                Timer write_timer;
                std::ofstream os(pk_path);
                write_mmap(os, pk_dir, *data.proving_key);
                if (!os.good()) {
                    throw_or_abort(format("Failed to write: ", pk_path));
                }
                std::cerr << "Done: " << write_timer.toString() << "s" << std::endl;
            }
        }
    }
    {
        auto vk_path = circuit_key_path + "/verification_key";
        if (exists(vk_path) && load) {
            std::cerr << "Loading verification key from: " << vk_path << std::endl;
            auto vk_stream = std::ifstream(vk_path);
            waffle::verification_key_data vk_data;
            read(vk_stream, vk_data);
            data.verification_key =
                std::make_shared<waffle::verification_key>(std::move(vk_data), crs->get_verifier_crs());
        } else if (compute) {
            std::cerr << "Computing verification key..." << std::endl;
            Timer timer;
            data.verification_key = composer.compute_verification_key();
            std::cerr << "Done: " << timer.toString() << "s" << std::endl;

            if (save) {
                std::ofstream os(vk_path);
                write(os, *data.verification_key);
                if (!os.good()) {
                    throw_or_abort(format("Failed to write: ", vk_path));
                }
            }
        }
    }
    {
        auto padding_path = circuit_key_path + "/padding_proof";
        if (exists(padding_path) && load) {
            std::cerr << "Loading padding proof from: " << padding_path << std::endl;
            std::ifstream is(padding_path);
            std::vector<uint8_t> proof((std::istreambuf_iterator<char>(is)), std::istreambuf_iterator<char>());
            data.padding_proof = proof;
        } else if (compute) {
            std::cerr << "Computing padding proof..." << std::endl;
            Timer timer;
            auto prover = composer.create_unrolled_prover();
            data.padding_proof = prover.construct_proof().proof_data;
            std::cerr << "Done: " << timer.toString() << "s" << std::endl;

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
