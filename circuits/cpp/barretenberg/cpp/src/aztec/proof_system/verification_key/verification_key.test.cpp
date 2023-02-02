#include <common/test.hpp>
#include <common/streams.hpp>
#include "verification_key.hpp"

using namespace barretenberg;
using namespace waffle;

TEST(verification_key, buffer_serialization)
{
    verification_key_data key;
    key.composer_type = static_cast<uint32_t>(ComposerType::STANDARD);
    key.n = 1234;
    key.num_public_inputs = 10;
    key.commitments["test1"] = g1::element::random_element();
    key.commitments["test2"] = g1::element::random_element();
    key.commitments["foo1"] = g1::element::random_element();
    key.commitments["foo2"] = g1::element::random_element();

    auto buf = to_buffer(key);
    auto result = from_buffer<verification_key_data>(buf);

    EXPECT_EQ(key, result);
}

TEST(verification_key, stream_serialization)
{
    verification_key_data key;
    key.composer_type = static_cast<uint32_t>(ComposerType::STANDARD);
    key.n = 1234;
    key.num_public_inputs = 10;
    key.commitments["test1"] = g1::element::random_element();
    key.commitments["test2"] = g1::element::random_element();
    key.commitments["foo1"] = g1::element::random_element();
    key.commitments["foo2"] = g1::element::random_element();

    std::stringstream s;
    write(s, key);

    verification_key_data result;
    read(static_cast<std::istream&>(s), result);

    EXPECT_EQ(key, result);
}