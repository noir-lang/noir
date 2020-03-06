#pragma once
#include <barretenberg/curves/bn254/fq.hpp>
#include <barretenberg/curves/bn254/fr.hpp>
#include <barretenberg/curves/bn254/g1.hpp>
#include <barretenberg/uint256/uint256.hpp>
#include <barretenberg/uint256/uint512.hpp>

#include <barretenberg/transcript/manifest.hpp>
#include <barretenberg/transcript/transcript.hpp>
#include <barretenberg/waffle/composer/standard_composer.hpp>

namespace test_helpers {
static std::seed_seq seq{ 1, 2, 3, 4, 5, 6, 7, 8 };
static std::mt19937_64 engine = std::mt19937_64(seq);
static std::uniform_int_distribution<uint64_t> dist{ 0ULL, UINT64_MAX };

inline uint32_t get_pseudorandom_uint32()
{
    return (uint32_t)dist(engine);
}

inline uint256_t get_pseudorandom_uint256()
{
    return { (uint64_t)dist(engine), (uint64_t)dist(engine), (uint64_t)dist(engine), (uint64_t)dist(engine) };
}

inline uint512_t get_pseudorandom_uint512()
{
    return { get_pseudorandom_uint256(), get_pseudorandom_uint256() };
}

inline barretenberg::fq get_pseudorandom_fq()
{
    barretenberg::fq out{
        (uint64_t)dist(engine), (uint64_t)dist(engine), (uint64_t)dist(engine), (uint64_t)dist(engine)
    };
    out.self_reduce_once();
    out.self_reduce_once();
    out.self_reduce_once();
    out.self_reduce_once();
    return out;
}
inline transcript::Transcript create_dummy_standard_transcript()
{
    std::vector<uint8_t> g1_vector(64);
    std::vector<uint8_t> fr_vector(32);

    for (size_t i = 0; i < g1_vector.size(); ++i) {
        g1_vector[i] = 1;
    }

    for (size_t i = 0; i < fr_vector.size(); ++i) {
        fr_vector[i] = 1;
    }
    transcript::Transcript transcript = transcript::Transcript(waffle::StandardComposer::create_manifest(0));
    transcript.add_element("circuit_size", { 1, 2, 3, 4 });
    transcript.add_element("public_input_size", { 0, 0, 0, 0 });
    transcript.apply_fiat_shamir("init");
    transcript.add_element("public_inputs", {});
    transcript.add_element("W_1", g1_vector);
    transcript.add_element("W_2", g1_vector);
    transcript.add_element("W_3", g1_vector);
    transcript.apply_fiat_shamir("beta");
    transcript.apply_fiat_shamir("gamma");
    transcript.add_element("Z", g1_vector);
    transcript.apply_fiat_shamir("alpha");
    return transcript;
}
} // namespace test_helpers