#include "../../constants.hpp"
#include "../../fixtures/user_context.hpp"
#include "index.hpp"
#include "../inner_proof_data/inner_proof_data.hpp"
#include "../notes/native/index.hpp"
#include <common/test.hpp>
#include <stdlib/merkle_tree/index.hpp>
#include <numeric/random/engine.hpp>

namespace rollup {
namespace proofs {
namespace claim {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;
using namespace rollup::proofs::notes::native;
using namespace rollup::proofs::notes::native::claim;
using namespace rollup::proofs::notes::native::defi_interaction;

namespace {
std::shared_ptr<waffle::FileReferenceStringFactory> srs;
circuit_data cd;
auto& engine = numeric::random::get_debug_engine();
} // namespace

class claim_tests : public ::testing::Test {
  protected:
    static void SetUpTestCase()
    {
        srs = std::make_shared<waffle::FileReferenceStringFactory>("../srs_db");
        cd = get_circuit_data(srs);
    }

    virtual void SetUp()
    {
        store = std::make_unique<MemoryStore>();
        data_tree = std::make_unique<MerkleTree<MemoryStore>>(*store, DATA_TREE_DEPTH, 0);
        defi_tree = std::make_unique<MerkleTree<MemoryStore>>(*store, DEFI_TREE_DEPTH, 1);
        user = rollup::fixtures::create_user_context();
    }

    template <typename T, typename Tree> void append_note(T const& note, Tree& tree)
    {
        tree->update_element(tree->size(), note.commit());
    }

    claim_tx create_claim_tx(claim_note const& claim_note,
                             uint32_t claim_note_index,
                             defi_interaction::note const& interaction_note)
    {
        claim_tx tx;
        tx.data_root = data_tree->root();
        tx.claim_note = claim_note;
        tx.claim_note_index = claim_note_index;
        tx.claim_note.fee = claim_note.fee;
        tx.claim_note_path = data_tree->get_hash_path(claim_note_index);

        tx.defi_root = defi_tree->root();
        tx.defi_interaction_note = interaction_note;
        tx.defi_interaction_note_path = defi_tree->get_hash_path(interaction_note.interaction_nonce);
        tx.defi_interaction_note_dummy_nullifier_nonce = fr::random_element();

        tx.output_value_a = ((uint512_t(claim_note.deposit_value) * uint512_t(interaction_note.total_output_value_a)) /
                             uint512_t(interaction_note.total_input_value))
                                .lo;
        tx.output_value_b = ((uint512_t(claim_note.deposit_value) * uint512_t(interaction_note.total_output_value_b)) /
                             uint512_t(interaction_note.total_input_value))
                                .lo;
        return tx;
    }

    rollup::fixtures::user_context user;
    std::unique_ptr<MemoryStore> store;
    std::unique_ptr<MerkleTree<MemoryStore>> data_tree;
    std::unique_ptr<MerkleTree<MemoryStore>> defi_tree;
    const uint32_t asset_id = 1;
};

TEST_F(claim_tests, test_claim)
{
    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 100,
                                           .total_output_value_a = 200,
                                           .total_output_value_b = 300,
                                           .interaction_result = 1 };
    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);

    EXPECT_TRUE(verify_logic(tx, cd).valid);
}

TEST_F(claim_tests, test_theft_via_field_overflow_fails_1)
{
    // Choose values to retain the ratio (deposit * total_output_value_a) == (output_value_a * total_input_value)
    // deposit value: 1
    // total_input_value: 2
    // total_output_value_a: 1
    // output_value_a: 10944121435919637611123202872628637544274182200208017171849102093287904247809; // = 2^(-1)

    uint256_t o_v_a(
        0xA1F0FAC9F8000001ULL, 0x9419F4243CDCB848ULL, 0xDC2822DB40C0AC2EULL, 0x183227397098D014ULL); // 2^(-1)

    const claim_note note1 = { .deposit_value = 1,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 2,
                                           .total_output_value_a = 1,
                                           .total_output_value_b = 0,
                                           .interaction_result = 1 };
    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);
    tx.output_value_a = o_v_a; // choose the cheeky large output value

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: output_value_a");
}

TEST_F(claim_tests, test_theft_via_field_overflow_fails_2)
{
    // Choose values to retain the ratio (deposit * total_output_value_a) == (output_value_a * total_input_value)
    // deposit value: 1
    // total_input_value: 74 // chosen (by brute force) so that the inverse is under 252 bits.
    // total_output_value_a: 1
    // output_value_a: 295787065835665881381708185746719933629031951356973437077002759278051466157 // 74^(-1)

    uint256_t o_v_a(
        0x507c2274294c1badULL, 0x11d7301ca7b2f039ULL, 0x21a0384b1d6cfdbcULL, 0x00a768d809f64ad0ULL); // 74^(-1)

    const claim_note note1 = { .deposit_value = 1,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 74,
                                           .total_output_value_a = 1,
                                           .total_output_value_b = 0,
                                           .interaction_result = 1 };
    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);
    tx.output_value_a = o_v_a; // choose the cheeky large output value, that flies under the 252-bit radar

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: subtract: output_value_a > total_output_value_a");
}

TEST_F(claim_tests, test_integer_division_works)
{
    // Choose a total_output_value_a which is not divisible (in the integers) by the user's share.
    // E.g. deposit 3 / 9. Then if the total output is 10, 1/3 * 10 = 3.333333, so should yield '3', rather than some
    // giant field element (3^(-1) * 10).
    // Tests to ensure the circuit copes with residuals correctly.

    const claim_note note1 = { .deposit_value = 3,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 9,
                                           .total_output_value_a = 10,
                                           .total_output_value_b = 0,
                                           .interaction_result = 1 };
    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);
    auto result = verify_logic(tx, cd);
    EXPECT_TRUE(result.valid);
}

TEST_F(claim_tests, test_outputs_larger_than_252_bits_fails)
{
    uint256_t r(
        0x43E1F593F0000001ULL, 0x2833E84879B97091ULL, 0xB85045B68181585DULL, 0x30644E72E131A029ULL); // field modulus

    const claim_note note1 = { .deposit_value = 1,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 1,
                                           .total_output_value_a = r - 1,
                                           .total_output_value_b = 0,
                                           .interaction_result = 1 };
    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);
    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: total_output_value_a");
}

TEST_F(claim_tests, test_zero_deposit_fails)
{
    const claim_note note1 = { .deposit_value = 0,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 1,
                                           .total_output_value_a = 1,
                                           .total_output_value_b = 0,
                                           .interaction_result = 1 };
    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);
    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "Not supported: zero deposit");
}

TEST_F(claim_tests, test_theft_via_zero_equality_fails)
{
    // Choose values so that the both sides are zero:
    // (deposit * total_output_value_a) == (output_value_a * total_input_value)
    // deposit = 0
    // total_input_value: 1
    // total_output_value_a: 0
    // output_value_a: MAX_252_BIT_VALUE

    uint256_t MAX_252_BIT_VALUE(
        0xffffffffffffffffULL, 0xffffffffffffffffULL, 0xffffffffffffffffULL, 0x00ffffffffffffffULL);
    const claim_note note1 = { .deposit_value = 0,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };
    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 1,
                                           .total_output_value_a = 1,
                                           .total_output_value_b = 0,
                                           .interaction_result = 1 };
    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);
    tx.output_value_a = MAX_252_BIT_VALUE; // Try to steal loads of money.

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err,
              "Not supported: zero deposit"); // This case was already caught by the ratio_check function preventing
                                              // a zero-valued denominator of b2 = total_output_value_a.
}

TEST_F(claim_tests, test_deposit_greater_than_total_fails)
{
    const claim_note note1 = { .deposit_value = 100,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };
    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 10,
                                           .total_output_value_a = 10,
                                           .total_output_value_b = 0,
                                           .interaction_result = 1 };
    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);
    tx.output_value_a = 100; // Match the malicious ratio of the deposit_value:total_input_value

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: subtract: deposit_value > total_input_value");
}

TEST_F(claim_tests, test_output_value_greater_than_total_fails)
{
    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };
    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 10,
                                           .total_output_value_a = 10,
                                           .total_output_value_b = 0,
                                           .interaction_result = 1 };
    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);
    tx.output_value_a = 100; // Cheeky

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: subtract: output_value_a > total_output_value_a");
}

TEST_F(claim_tests, test_zero_output_value_fails)
{
    const claim_note note1 = { .deposit_value = 1,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 1,
                                           .total_output_value_a = 1,
                                           .total_output_value_b = 0,
                                           .interaction_result = 1 };

    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);
    tx.output_value_a = 0; // We want to test whether a 0 output_value will fail

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "ratio check 1 failed");
}

TEST_F(claim_tests, test_zero_total_output_value_fails)
{
    const claim_note note1 = { .deposit_value = 1,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 1,
                                           .total_output_value_a = 0,
                                           .total_output_value_b = 0,
                                           .interaction_result = 1 };

    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);
    tx.output_value_a = 1; // We want to test whether a 0 output_value will fail

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(
        result.err,
        "safe_uint_t range constraint failure: subtract: output_value_a > total_output_value_a"); // The 'division by
                                                                                                  // zero' checks aren't
                                                                                                  // even reached,
                                                                                                  // because this one
                                                                                                  // gets triggered
                                                                                                  // first.
}

TEST_F(claim_tests, test_unmatching_ratio_a_fails)
{
    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 100,
                                           .total_output_value_a = 200,
                                           .total_output_value_b = 300,
                                           .interaction_result = 1 };

    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);
    tx.output_value_a = 10; // Force an unmatching ratio (it should be 20)

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "ratio check 1 failed");
}

TEST_F(claim_tests, test_unmatching_ratio_b_fails)
{
    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 100,
                                           .total_output_value_a = 200,
                                           .total_output_value_b = 300,
                                           .interaction_result = 1 };

    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);
    tx.output_value_b = 10; // Force an unmatching ratio (it should be 20)

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "ratio check 2 failed");
}

TEST_F(claim_tests, test_unmatching_bridge_ids_fails)
{
    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = 1, // mismatch
                                           .interaction_nonce = 0,
                                           .total_input_value = 100,
                                           .total_output_value_a = 200,
                                           .total_output_value_b = 300,
                                           .interaction_result = 1 };

    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "note bridge ids don't match");
}

TEST_F(claim_tests, test_unmatching_interaction_nonces_fails)
{
    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 1, // mismatch
                                           .total_input_value = 100,
                                           .total_output_value_a = 200,
                                           .total_output_value_b = 300,
                                           .interaction_result = 1 };

    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "note nonces don't match");
}

TEST_F(claim_tests, test_missing_claim_note_fails)
{
    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 100,
                                           .total_output_value_a = 200,
                                           .total_output_value_b = 300,
                                           .interaction_result = 1 };

    // Notice: note1 not being appended
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "claim note not a member");
}

TEST_F(claim_tests, test_missing_interaction_note_fails)
{
    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = 0,
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 100,
                                           .total_output_value_a = 200,
                                           .total_output_value_b = 300,
                                           .interaction_result = 1 };

    append_note(note1, data_tree);
    // Notice: note2 not being appended
    claim_tx tx = create_claim_tx(note1, 0, note2);

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "defi interaction note not a member");
}

TEST_F(claim_tests, test_claim_for_virtual_note)
{
    const bridge_id bridge_id = { .bridge_address_id = 0,
                                  .input_asset_id = 0,
                                  .output_asset_id_a = 0,
                                  .output_asset_id_b = 0,
                                  .opening_nonce = 0,
                                  .config = bridge_id::bit_config{ .first_input_virtual = false,
                                                                   .second_input_virtual = false,
                                                                   .first_output_virtual = false,
                                                                   .second_output_virtual = true, // <--
                                                                   .second_input_real = false,
                                                                   .second_output_real = false },
                                  .aux_data = 0 };
    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = bridge_id.to_uint256_t(),
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = bridge_id.to_uint256_t(),
                                           .interaction_nonce = 0,
                                           .total_input_value = 100,
                                           .total_output_value_a = 300,
                                           .total_output_value_b = 300,
                                           .interaction_result = 1 };
    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);

    EXPECT_TRUE(verify_logic(tx, cd).valid);
}

TEST_F(claim_tests, test_unsupported_first_input_note_virtual_fails)
{
    const bridge_id bridge_id = { .bridge_address_id = 0,
                                  .input_asset_id = 0,
                                  .output_asset_id_a = 111,
                                  .output_asset_id_b = 222,
                                  .opening_nonce = 0,
                                  .config = bridge_id::bit_config{ .first_input_virtual = true, // <--
                                                                   .second_input_virtual = false,
                                                                   .first_output_virtual = false,
                                                                   .second_output_virtual = false,
                                                                   .second_input_real = false,
                                                                   .second_output_real = false },
                                  .aux_data = 0 };

    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = bridge_id.to_uint256_t(),
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = bridge_id.to_uint256_t(),
                                           .interaction_nonce = 0,
                                           .total_input_value = 100,
                                           .total_output_value_a = 200,
                                           .total_output_value_b = 300,
                                           .interaction_result = 1 };
    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "Not yet supported: first input asset cannot be virtual");
}

TEST_F(claim_tests, test_unsupported_first_output_note_virtual_fails)
{
    const bridge_id bridge_id = { .bridge_address_id = 0,
                                  .input_asset_id = 0,
                                  .output_asset_id_a = 111,
                                  .output_asset_id_b = 222,
                                  .opening_nonce = 0,
                                  .config = bridge_id::bit_config{ .first_input_virtual = false,
                                                                   .second_input_virtual = false,
                                                                   .first_output_virtual = true, // <--
                                                                   .second_output_virtual = false,
                                                                   .second_input_real = false,
                                                                   .second_output_real = false },
                                  .aux_data = 0 };

    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = bridge_id.to_uint256_t(),
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = bridge_id.to_uint256_t(),
                                           .interaction_nonce = 0,
                                           .total_input_value = 100,
                                           .total_output_value_a = 200,
                                           .total_output_value_b = 300,
                                           .interaction_result = 1 };
    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "Not yet supported: first output asset cannot be virtual");
}

TEST_F(claim_tests, test_second_input_note_virtual_and_real_fails)
{
    const bridge_id bridge_id = { .bridge_address_id = 0,
                                  .input_asset_id = 0,
                                  .output_asset_id_a = 111,
                                  .output_asset_id_b = 222,
                                  .opening_nonce = 0,
                                  .config = bridge_id::bit_config{ .first_input_virtual = false,
                                                                   .second_input_virtual = true, // <--
                                                                   .first_output_virtual = false,
                                                                   .second_output_virtual = false,
                                                                   .second_input_real = true, // <--
                                                                   .second_output_real = false },
                                  .aux_data = 0 };

    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = bridge_id.to_uint256_t(),
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = bridge_id.to_uint256_t(),
                                           .interaction_nonce = 0,
                                           .total_input_value = 100,
                                           .total_output_value_a = 200,
                                           .total_output_value_b = 300,
                                           .interaction_result = 1 };
    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "Contradiction: second_input_virtual AND second_input_real cannot both be true");
}

TEST_F(claim_tests, test_second_output_note_virtual_and_real_fails)
{
    const bridge_id bridge_id = { .bridge_address_id = 0,
                                  .input_asset_id = 0,
                                  .output_asset_id_a = 111,
                                  .output_asset_id_b = 222,
                                  .opening_nonce = 0,
                                  .config = bridge_id::bit_config{ .first_input_virtual = false,
                                                                   .second_input_virtual = false,
                                                                   .first_output_virtual = false,
                                                                   .second_output_virtual = true, // <--
                                                                   .second_input_real = false,
                                                                   .second_output_real = true }, // <--
                                  .aux_data = 0 };

    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = bridge_id.to_uint256_t(),
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = bridge_id.to_uint256_t(),
                                           .interaction_nonce = 0,
                                           .total_input_value = 100,
                                           .total_output_value_a = 200,
                                           .total_output_value_b = 300,
                                           .interaction_result = 1 };
    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "Contradiction: second_output_virtual AND second_output_real cannot both be true");
}

TEST_F(claim_tests, test_second_output_real_means_asset_ids_equal_fails)
{
    const bridge_id bridge_id = { .bridge_address_id = 0,
                                  .input_asset_id = 0,
                                  .output_asset_id_a = 111,
                                  .output_asset_id_b = 111, // <-- equal
                                  .opening_nonce = 0,
                                  .config = bridge_id::bit_config{ .first_input_virtual = false,
                                                                   .second_input_virtual = false,
                                                                   .first_output_virtual = false,
                                                                   .second_output_virtual = false,
                                                                   .second_input_real = false,
                                                                   .second_output_real = true }, // <--
                                  .aux_data = 0 };

    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = bridge_id.to_uint256_t(),
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = bridge_id.to_uint256_t(),
                                           .interaction_nonce = 0,
                                           .total_input_value = 100,
                                           .total_output_value_a = 200,
                                           .total_output_value_b = 300,
                                           .interaction_result = 1 };
    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "second_output_real == true AND both output asset ids are identical");
}

TEST_F(claim_tests, test_claim_2_outputs_full_proof)
{
    const bridge_id bridge_id = { .bridge_address_id = 0,
                                  .input_asset_id = 0,
                                  .output_asset_id_a = 111,
                                  .output_asset_id_b = 222,
                                  .opening_nonce = 0,
                                  .config = bridge_id::bit_config{ .first_input_virtual = false,
                                                                   .second_input_virtual = false,
                                                                   .first_output_virtual = false,
                                                                   .second_output_virtual = false,
                                                                   .second_input_real = false,
                                                                   .second_output_real = true },
                                  .aux_data = 0 };

    // Create some values for our circuit that are large enough to properly test the ratio checks.
    // The defi deposit value must be atmost 242 bits (since we sum up defi deposits in rollup circuit).
    auto random_value = []() {
        uint256_t a = engine.get_random_uint256();
        a.data[3] = a.data[3] & 0x0003ffffffffffffULL;
        return a;
    };
    uint256_t input_value = random_value();
    uint256_t total_input = random_value();
    uint256_t total_output_a = random_value();
    uint256_t total_output_b = random_value();

    // Check total_in >= user_in. Does not work otherwise because we get integer overflow.
    if (input_value > total_input) {
        std::swap(input_value, total_input);
    }

    // Create and add a claim note, and a defi interaction note, to the data tree.
    const claim_note note1 = { .deposit_value = input_value,
                               .bridge_id = bridge_id.to_uint256_t(),
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = bridge_id.to_uint256_t(),
                                           .interaction_nonce = 0,
                                           .total_input_value = total_input,
                                           .total_output_value_a = total_output_a,
                                           .total_output_value_b = total_output_b,
                                           .interaction_result = 1 };

    append_note(note1, data_tree);
    append_note(note2, defi_tree);

    // Construct transaction data.
    claim_tx tx = create_claim_tx(note1, 0, note2);

    // Verify proof.
    auto result = verify(tx, cd);
    ASSERT_TRUE(result.verified);

    uint256_t nullifier1 = compute_nullifier(note1.commit());
    uint256_t nullifier2 = compute_dummy_nullifier(note2.commit(), tx.defi_interaction_note_dummy_nullifier_nonce);

    // Compute expected public inputs.
    auto proof_data = inner_proof_data(result.proof_data);

    const value_note expected_output_note1 = { .value = tx.output_value_a,
                                               .asset_id = bridge_id.output_asset_id_a,
                                               .nonce = 0,
                                               .owner = user.owner.public_key,
                                               .secret = user.note_secret,
                                               .creator_pubkey = 0,
                                               .input_nullifier = nullifier1 };

    const value_note expected_output_note2 = { .value = tx.output_value_b,
                                               .asset_id = bridge_id.output_asset_id_b,
                                               .nonce = 0,
                                               .owner = user.owner.public_key,
                                               .secret = user.note_secret,
                                               .creator_pubkey = 0,
                                               .input_nullifier = nullifier2 };

    // Validate public inputs.
    EXPECT_EQ(proof_data.proof_id, ProofIds::DEFI_CLAIM);
    EXPECT_EQ(proof_data.note_commitment1, expected_output_note1.commit());
    EXPECT_EQ(proof_data.note_commitment2, expected_output_note2.commit());
    EXPECT_EQ(proof_data.nullifier1, nullifier1);
    EXPECT_EQ(proof_data.nullifier2, uint256_t(nullifier2));
    EXPECT_EQ(proof_data.public_value, uint256_t(0));
    EXPECT_EQ(proof_data.public_owner, fr(0));
    EXPECT_EQ(proof_data.asset_id, uint256_t(0));
    EXPECT_EQ(proof_data.merkle_root, data_tree->root());
    EXPECT_EQ(proof_data.tx_fee, uint256_t(0));
    EXPECT_EQ(proof_data.tx_fee_asset_id, bridge_id.input_asset_id);
    EXPECT_EQ(proof_data.bridge_id, tx.claim_note.bridge_id);
    EXPECT_EQ(proof_data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(proof_data.defi_root, defi_tree->root());
    EXPECT_EQ(proof_data.backward_link, fr(0));
    EXPECT_EQ(proof_data.allow_chain, uint256_t(0));
}

TEST_F(claim_tests, test_claim_1_output_full_proof)
{
    const bridge_id bridge_id = { .bridge_address_id = 0,
                                  .input_asset_id = 0,
                                  .output_asset_id_a = 111,
                                  .output_asset_id_b = 222,
                                  .opening_nonce = 0,
                                  .config = bridge_id::bit_config{ .first_input_virtual = false,
                                                                   .second_input_virtual = false,
                                                                   .first_output_virtual = false,
                                                                   .second_output_virtual = false,
                                                                   .second_input_real = false,
                                                                   .second_output_real = false },
                                  .aux_data = 0 };
    const uint32_t claim_fee = 8;

    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = bridge_id.to_uint256_t(),
                               .defi_interaction_nonce = 0,
                               .fee = claim_fee,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = bridge_id.to_uint256_t(),
                                           .interaction_nonce = 0,
                                           .total_input_value = 100,
                                           .total_output_value_a = 200,
                                           .total_output_value_b = 300,
                                           .interaction_result = 1 };

    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);
    auto result = verify(tx, cd);

    auto proof_data = inner_proof_data(result.proof_data);

    uint256_t nullifier1 = compute_nullifier(note1.commit());
    uint256_t nullifier2 = compute_dummy_nullifier(note2.commit(), tx.defi_interaction_note_dummy_nullifier_nonce);

    const value_note expected_output_note1 = { .value = 20,
                                               .asset_id = bridge_id.output_asset_id_a,
                                               .nonce = 0,
                                               .owner = user.owner.public_key,
                                               .secret = user.note_secret,
                                               .creator_pubkey = 0,
                                               .input_nullifier = nullifier1 };

    EXPECT_EQ(proof_data.proof_id, ProofIds::DEFI_CLAIM);
    EXPECT_EQ(proof_data.note_commitment1, expected_output_note1.commit());
    EXPECT_EQ(proof_data.note_commitment2, fr(0));
    EXPECT_EQ(proof_data.nullifier1, nullifier1);
    EXPECT_EQ(proof_data.nullifier2, uint256_t(nullifier2));
    EXPECT_EQ(proof_data.public_value, uint256_t(0));
    EXPECT_EQ(proof_data.public_owner, fr(0));
    EXPECT_EQ(proof_data.asset_id, uint256_t(0));
    EXPECT_EQ(proof_data.merkle_root, data_tree->root());
    EXPECT_EQ(proof_data.tx_fee, claim_fee);
    EXPECT_EQ(proof_data.tx_fee_asset_id, bridge_id.input_asset_id);
    EXPECT_EQ(proof_data.bridge_id, tx.claim_note.bridge_id);
    EXPECT_EQ(proof_data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(proof_data.defi_root, defi_tree->root());
    EXPECT_EQ(proof_data.backward_link, fr(0));
    EXPECT_EQ(proof_data.allow_chain, uint256_t(0));

    EXPECT_TRUE(result.verified);
}

TEST_F(claim_tests, test_claim_1_output_with_virtual_note_full_proof)
{
    const bridge_id bridge_id = { .bridge_address_id = 0,
                                  .input_asset_id = 0,
                                  .output_asset_id_a = 111,
                                  .output_asset_id_b = 222,
                                  .opening_nonce = 0,
                                  .config = bridge_id::bit_config{ .first_input_virtual = false,
                                                                   .second_input_virtual = false,
                                                                   .first_output_virtual = false,
                                                                   .second_output_virtual = true,
                                                                   .second_input_real = false,
                                                                   .second_output_real = false },
                                  .aux_data = 0 };
    const uint32_t claim_fee = 8;
    const uint64_t nonce = 2;

    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = bridge_id.to_uint256_t(),
                               .defi_interaction_nonce = nonce,
                               .fee = claim_fee,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = bridge_id.to_uint256_t(),
                                           .interaction_nonce = nonce,
                                           .total_input_value = 100,
                                           .total_output_value_a = 200,
                                           .total_output_value_b = 300,
                                           .interaction_result = 1 };

    const defi_interaction::note dummy = { .bridge_id = 0,
                                           .interaction_nonce = 0,
                                           .total_input_value = 100,
                                           .total_output_value_a = 100,
                                           .total_output_value_b = 100,
                                           .interaction_result = 1 };

    append_note(note1, data_tree);
    append_note(dummy, defi_tree);
    append_note(dummy, defi_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);
    auto result = verify(tx, cd);

    auto proof_data = inner_proof_data(result.proof_data);

    uint256_t nullifier1 = compute_nullifier(note1.commit());
    uint256_t nullifier2 = compute_dummy_nullifier(note2.commit(), tx.defi_interaction_note_dummy_nullifier_nonce);

    const value_note expected_output_note1 = { .value = 20,
                                               .asset_id = bridge_id.output_asset_id_a,
                                               .nonce = 0,
                                               .owner = user.owner.public_key,
                                               .secret = user.note_secret,
                                               .creator_pubkey = 0,
                                               .input_nullifier = nullifier1 };

    const value_note expected_output_note2 = { .value = 20,
                                               .asset_id =
                                                   static_cast<uint32_t>(1 << (MAX_NUM_ASSETS_BIT_LENGTH - 1)) + nonce,
                                               .nonce = 0,
                                               .owner = user.owner.public_key,
                                               .secret = user.note_secret,
                                               .creator_pubkey = 0,
                                               .input_nullifier = nullifier2 };

    EXPECT_EQ(proof_data.proof_id, ProofIds::DEFI_CLAIM);
    EXPECT_EQ(proof_data.merkle_root, data_tree->root());
    EXPECT_EQ(proof_data.note_commitment1, expected_output_note1.commit());
    EXPECT_EQ(proof_data.note_commitment2, expected_output_note2.commit());
    EXPECT_EQ(proof_data.nullifier1, nullifier1);
    EXPECT_EQ(proof_data.nullifier2, nullifier2);
    EXPECT_EQ(proof_data.public_value, uint256_t(0));
    EXPECT_EQ(proof_data.public_owner, fr(0));
    EXPECT_EQ(proof_data.bridge_id, tx.claim_note.bridge_id);
    EXPECT_EQ(proof_data.tx_fee, claim_fee);
    EXPECT_EQ(proof_data.tx_fee_asset_id, bridge_id.input_asset_id);
    EXPECT_EQ(proof_data.bridge_id, tx.claim_note.bridge_id);
    EXPECT_EQ(proof_data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(proof_data.defi_root, defi_tree->root());

    EXPECT_TRUE(result.verified);
}

TEST_F(claim_tests, test_claim_refund_full_proof)
{
    const bridge_id bridge_id = { .bridge_address_id = 0,
                                  .input_asset_id = 0,
                                  .output_asset_id_a = 111,
                                  .output_asset_id_b = 222,
                                  .opening_nonce = 0,
                                  .config = bridge_id::bit_config{ .first_input_virtual = false,
                                                                   .second_input_virtual = false,
                                                                   .first_output_virtual = false,
                                                                   .second_output_virtual = false,
                                                                   .second_input_real = false,
                                                                   .second_output_real = false },
                                  .aux_data = 0 };

    const claim_note note1 = { .deposit_value = 10,
                               .bridge_id = bridge_id.to_uint256_t(),
                               .defi_interaction_nonce = 0,
                               .fee = 0,
                               .value_note_partial_commitment =
                                   create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                               .input_nullifier = fr::random_element() };

    const defi_interaction::note note2 = { .bridge_id = bridge_id.to_uint256_t(),
                                           .interaction_nonce = 0,
                                           .total_input_value = 100,
                                           .total_output_value_a = 200,
                                           .total_output_value_b = 300,
                                           .interaction_result = 0 }; // <-- refund

    append_note(note1, data_tree);
    append_note(note2, defi_tree);
    claim_tx tx = create_claim_tx(note1, 0, note2);
    auto result = verify(tx, cd);

    auto proof_data = inner_proof_data(result.proof_data);

    uint256_t nullifier1 = compute_nullifier(note1.commit());
    uint256_t nullifier2 = compute_dummy_nullifier(note2.commit(), tx.defi_interaction_note_dummy_nullifier_nonce);

    const value_note expected_output_note1 = { .value = 10,
                                               .asset_id = bridge_id.input_asset_id,
                                               .nonce = 0,
                                               .owner = user.owner.public_key,
                                               .secret = user.note_secret,
                                               .creator_pubkey = 0,
                                               .input_nullifier = nullifier1 };

    EXPECT_EQ(proof_data.proof_id, ProofIds::DEFI_CLAIM);
    EXPECT_EQ(proof_data.note_commitment1, expected_output_note1.commit());
    EXPECT_EQ(proof_data.note_commitment2, fr(0));
    EXPECT_EQ(proof_data.nullifier1, nullifier1);
    EXPECT_EQ(proof_data.nullifier2, uint256_t(nullifier2));
    EXPECT_EQ(proof_data.public_value, uint256_t(0));
    EXPECT_EQ(proof_data.public_owner, fr(0));
    EXPECT_EQ(proof_data.asset_id, uint256_t(0));
    EXPECT_EQ(proof_data.merkle_root, data_tree->root());
    EXPECT_EQ(proof_data.tx_fee, uint256_t(0));
    EXPECT_EQ(proof_data.tx_fee_asset_id, bridge_id.input_asset_id);
    EXPECT_EQ(proof_data.bridge_id, tx.claim_note.bridge_id);
    EXPECT_EQ(proof_data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(proof_data.defi_root, defi_tree->root());
    EXPECT_EQ(proof_data.backward_link, fr(0));
    EXPECT_EQ(proof_data.allow_chain, uint256_t(0));

    EXPECT_TRUE(result.verified);
}

// RANGE CHECK TESTS

// For less verbose code, we set up some default test data here. Individual elements of the test_data can then be
// modified in each test.
class test_data {
  public:
    bridge_id bid;
    claim_note note1;
    defi_interaction::note note2;

    test_data(rollup::fixtures::user_context user)
    {
        bid = { .bridge_address_id = 0,
                .input_asset_id = 0,
                .output_asset_id_a = 111,
                .output_asset_id_b = 222,
                .opening_nonce = 0,
                .config = bridge_id::bit_config{ .first_input_virtual = false,
                                                 .second_input_virtual = false,
                                                 .first_output_virtual = false,
                                                 .second_output_virtual = false,
                                                 .second_input_real = false,
                                                 .second_output_real = false },
                .aux_data = 0 };

        note1 = { .deposit_value = 10,
                  .bridge_id = bid.to_uint256_t(),
                  .defi_interaction_nonce = 0,
                  .fee = 0,
                  .value_note_partial_commitment =
                      create_partial_commitment(user.note_secret, user.owner.public_key, 0, 0),
                  .input_nullifier = fr::random_element() };

        note2 = { .bridge_id = bid.to_uint256_t(),
                  .interaction_nonce = 0,
                  .total_input_value = 100,
                  .total_output_value_a = 200,
                  .total_output_value_b = 300,
                  .interaction_result = 1 };
    };
};

// Elements of bridge_id are implicitly range-constrained by the bit-shifting in bridge_id.hpp (since bits outside of
// the valid ranges are ignored)

// Can't create tests which attempt to exceed 32-bit range for values which are 'fed in' as uint32_t

TEST_F(claim_tests, test_total_input_value_out_of_range_fails)
{
    test_data test_data(user);
    uint256_t total_input_value = uint256_t(1) << 253;
    test_data.note2.total_input_value = total_input_value;
    append_note(test_data.note1, data_tree);
    append_note(test_data.note2, defi_tree);
    claim_tx tx = create_claim_tx(test_data.note1, 0, test_data.note2);

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: total_input_value");
}

TEST_F(claim_tests, test_total_output_value_a_out_of_range_fails)
{
    test_data test_data(user);
    uint256_t total_output_value_a = uint256_t(1) << 253;
    test_data.note2.total_output_value_a = total_output_value_a;
    append_note(test_data.note1, data_tree);
    append_note(test_data.note2, defi_tree);
    claim_tx tx = create_claim_tx(test_data.note1, 0, test_data.note2);

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: total_output_value_a");
}

TEST_F(claim_tests, test_total_output_value_b_out_of_range_fails)
{
    test_data test_data(user);
    uint256_t total_output_value_b = uint256_t(1) << 253;
    test_data.note2.total_output_value_b = total_output_value_b;
    append_note(test_data.note1, data_tree);
    append_note(test_data.note2, defi_tree);
    claim_tx tx = create_claim_tx(test_data.note1, 0, test_data.note2);

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: total_output_value_b");
}

TEST_F(claim_tests, test_deposit_value_out_of_range_fails)
{
    test_data test_data(user);
    uint256_t deposit_value = uint256_t(1) << 253;
    test_data.note1.deposit_value = deposit_value;
    append_note(test_data.note1, data_tree);
    append_note(test_data.note2, defi_tree);
    claim_tx tx = create_claim_tx(test_data.note1, 0, test_data.note2);

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: deposit_value");
}

TEST_F(claim_tests, test_fee_out_of_range_fails)
{
    test_data test_data(user);
    uint256_t fee = uint256_t(1) << 253;
    test_data.note1.fee = fee;
    append_note(test_data.note1, data_tree);
    append_note(test_data.note2, defi_tree);
    claim_tx tx = create_claim_tx(test_data.note1, 0, test_data.note2);

    auto result = verify_logic(tx, cd);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: fee");
}

} // namespace claim
} // namespace proofs
} // namespace rollup