#include <common/timer.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include <plonk/proof_system/verification_key/sol_gen.hpp>
#include <stdlib/types/standard.hpp>
#include <stdlib/hash/sha256/sha256.hpp>

using namespace plonk::stdlib::types::standard;
using namespace serialize;

Composer test_circuit()
{
    Composer composer("../srs_db/ignition");
    field_ct a(public_witness_ct(&composer, 300));
    field_ct b(public_witness_ct(&composer, 100));
    field_ct c(public_witness_ct(&composer, 200));
    a.assert_equal(b + c);
    // byte_array_ct hash_input(a);
    // byte_array_ct hash_output(stdlib::sha256<Composer>(hash_input));
    // b += c * field_ct(hash_output);
    a += b * c;
    c += c * a + b + c + a;
    b *= a + b;
    return composer;
}

bool create_test_proof()
{
    auto composer = test_circuit();

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    auto verified = verifier.verify_proof(proof);

    std::cerr << "verified = " << verified << std::endl;

    std::ofstream os("StandardPlonkVkComparison.sol");
    output_vk_sol_standard(os, verifier.key, "StandardPlonkVk");

    write(std::cout, proof.proof_data);
    write(std::cout, verified);
    std::cout << std::flush;

    return verified;
}

void create_test_verification_key(const std::string& output_path)
{
    auto composer = test_circuit();

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    auto verified = verifier.verify_proof(proof);

    std::cerr << "creating verifier?" << std::endl;
    // auto key = composer.compute_verification_key();
    std::cerr << "verified ? " << verified << std::endl;
    std::ofstream os(output_path + "/" + "StandardPlonkVk.sol");
    output_vk_sol_standard(os, verifier.key, "StandardPlonkVk");
}

int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);

    size_t proof_id = (size_t)atoi(args[1].c_str());

    switch (proof_id) {
    case 0: {
        create_test_proof();
        break;
    }
    case 1: {
        const std::string output_path = args[2];
        create_test_verification_key(output_path);
        break;
    }
    }

    return 0;
}
