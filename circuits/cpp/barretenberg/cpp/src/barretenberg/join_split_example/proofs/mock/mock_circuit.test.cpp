#include "mock_circuit.hpp"
#include "../join_split/join_split_tx.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/join_split_example/types.hpp"

using namespace proof_system::plonk::stdlib;

namespace rollup {
namespace proofs {
namespace mock {

TEST(mock_circuit_tests, test_simple_circuit)
{
    // Dummy public inputs
    std::vector<fr> public_inputs;
    for (size_t i = 0; i < 16; i++) {
        public_inputs.push_back(fr::random_element());
    }

    Composer composer = Composer("../srs_db/ignition");
    mock_circuit(composer, public_inputs);

    auto prover = composer.create_prover();
    plonk::proof proof = prover.construct_proof();

    std::cout << "gates: " << composer.get_num_gates() << std::endl;
    std::cout << "proof size: " << proof.proof_data.size() << std::endl;
    std::cout << "public inputs size: " << composer.public_inputs.size() << std::endl;

    auto verifier = composer.create_verifier();
    bool result = verifier.verify_proof(proof);

    EXPECT_TRUE(result);
}

} // namespace mock
} // namespace proofs
} // namespace rollup
