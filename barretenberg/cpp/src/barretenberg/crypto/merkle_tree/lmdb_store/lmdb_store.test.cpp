#include <cstdint>
#include <gtest/gtest.h>

#include <chrono>
#include <cstdlib>
#include <filesystem>
#include <stdexcept>
#include <vector>

#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/streams.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/crypto/merkle_tree/fixtures.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/callbacks.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "lmdb_store.hpp"

using namespace bb::stdlib;
using namespace bb::crypto::merkle_tree;

using Builder = bb::UltraCircuitBuilder;

using field_ct = field_t<Builder>;
using witness_ct = witness_t<Builder>;

const int SAMPLE_DATA_SIZE = 1024;

class LMDBStoreTest : public testing::Test {
  protected:
    void SetUp() override
    {
        // setup with 1MB max db size, 1 max database and 2 maximum concurrent readers
        _directory = random_temp_directory();
        std::filesystem::create_directories(_directory);
        _environment = std::make_unique<LMDBEnvironment>(_directory, 1024, 2, 2);
    }

    void TearDown() override { std::filesystem::remove_all(_directory); }

    static std::string _directory;

    std::unique_ptr<LMDBEnvironment> _environment;
};

std::string LMDBStoreTest::_directory;

TEST_F(LMDBStoreTest, can_write_to_and_read_from_store)
{
    {
        LMDBStore store(*_environment, "DB1");
        {
            std::vector<uint8_t> buf;
            write(buf, VALUES[0]);
            LMDBWriteTransaction::Ptr transaction = store.create_write_transaction();
            transaction->put_node(0, 0, buf);
            transaction->commit();
        }

        {
            LMDBReadTransaction::Ptr transaction = store.create_read_transaction();
            std::vector<uint8_t> buf2;
            bool success = transaction->get_node(0, 0, buf2);
            EXPECT_EQ(success, true);
            bb::fr value = from_buffer<bb::fr>(buf2, 0);
            EXPECT_EQ(value, VALUES[0]);
        }
    }

    {
        LMDBStore store(*_environment, "DB2");
        {
            std::vector<uint8_t> key;
            write(key, VALUES[0]);
            std::vector<uint8_t> value;
            write(value, VALUES[1]);
            LMDBWriteTransaction::Ptr transaction = store.create_write_transaction();
            transaction->put_value(key, value);
            transaction->commit();
        }

        {
            LMDBReadTransaction::Ptr transaction = store.create_read_transaction();
            std::vector<uint8_t> key;
            write(key, VALUES[0]);
            std::vector<uint8_t> value;
            bool success = transaction->get_value(key, value);
            EXPECT_EQ(success, true);
            bb::fr v = from_buffer<bb::fr>(value, 0);
            EXPECT_EQ(v, VALUES[1]);
        }
    }
}

TEST_F(LMDBStoreTest, reading_an_empty_key_reports_correctly)
{
    {
        LMDBStore store(*_environment, "DB1");

        {
            std::vector<uint8_t> buf;
            write(buf, VALUES[0]);
            LMDBWriteTransaction::Ptr transaction = store.create_write_transaction();
            transaction->put_node(0, 0, buf);
            transaction->commit();
        }

        {
            LMDBReadTransaction::Ptr transaction = store.create_read_transaction();
            std::vector<uint8_t> buf2;
            bool success = transaction->get_node(0, 1, buf2);
            EXPECT_EQ(success, false);
        }
    }

    {
        LMDBStore store(*_environment, "DB2");

        {
            std::vector<uint8_t> key;
            write(key, VALUES[0]);
            std::vector<uint8_t> value;
            write(value, VALUES[1]);
            LMDBWriteTransaction::Ptr transaction = store.create_write_transaction();
            transaction->put_value(key, value);
            transaction->commit();
        }
        {
            LMDBReadTransaction::Ptr transaction = store.create_read_transaction();
            std::vector<uint8_t> key2;
            write(key2, VALUES[5]);
            std::vector<uint8_t> value;
            bool success = transaction->get_value(key2, value);
            EXPECT_EQ(success, false);
        }
    }
}

TEST_F(LMDBStoreTest, can_write_and_read_multiple)
{

    {
        LMDBStore store(*_environment, "DB1");

        {
            LMDBWriteTransaction::Ptr transaction = store.create_write_transaction();
            for (size_t i = 0; i < SAMPLE_DATA_SIZE; i++) {
                std::vector<uint8_t> buf;
                write(buf, VALUES[i]);
                transaction->put_node(10, i, buf);
            }
            transaction->commit();
        }

        {
            LMDBReadTransaction::Ptr transaction = store.create_read_transaction();
            for (size_t i = 0; i < SAMPLE_DATA_SIZE; i++) {
                std::vector<uint8_t> buf2;
                bool success = transaction->get_node(10, i, buf2);
                EXPECT_EQ(success, true);
                bb::fr value = from_buffer<bb::fr>(buf2, 0);
                EXPECT_EQ(value, VALUES[i]);
            }
        }
    }

    {
        LMDBStore store(*_environment, "DB2");
        uint32_t num_reads = 128;

        {
            LMDBWriteTransaction::Ptr transaction = store.create_write_transaction();
            for (size_t i = 0; i < num_reads; i++) {
                std::vector<uint8_t> key;
                write(key, VALUES[i]);
                std::vector<uint8_t> buf;
                write(buf, VALUES[i + 128]);
                transaction->put_value(key, buf);
            }
            transaction->commit();
        }

        {
            for (size_t i = 0; i < num_reads; i++) {
                LMDBReadTransaction::Ptr transaction = store.create_read_transaction();
                std::vector<uint8_t> key;
                write(key, VALUES[i]);
                std::vector<uint8_t> buf2;
                bool success = transaction->get_value(key, buf2);
                EXPECT_EQ(success, true);
                bb::fr value = from_buffer<bb::fr>(buf2, 0);
                EXPECT_EQ(value, VALUES[i + 128]);
            }
        }
    }
}

TEST_F(LMDBStoreTest, throws_if_write_transaction_is_reused)
{
    {
        LMDBStore store(*_environment, "DB1");
        {
            std::vector<uint8_t> buf;
            write(buf, VALUES[0]);
            LMDBWriteTransaction::Ptr transaction = store.create_write_transaction();
            transaction->put_node(0, 0, buf);
            transaction->commit();
            EXPECT_THROW(transaction->put_node(0, 1, buf), std::runtime_error);
        }
    }
}

TEST_F(LMDBStoreTest, can_retrieve_the_value_at_the_previous_key)
{

    // This test is performed using integer keys of uint128_t
    // We also add keys of different sizes but with larger and smaller values
    // This ensures we don't erroneously return keys of different sizes
    LMDBStore store(*_environment, "note hash tree", false, false, integer_key_cmp);

    std::vector<uint32_t> values{ 1, 2, 3, 4, 5 };

    auto& random_engine = bb::numeric::get_randomness();
    uint32_t num_keys = static_cast<uint32_t>(values.size());
    // ensure first key is at least 100
    uint128_t key = random_engine.get_random_uint32() + 100;
    std::vector<uint128_t> keys;
    for (uint32_t i = 0; i < num_keys; i++) {
        keys.push_back(key);
        // ensure space of at least 50
        key += random_engine.get_random_uint32();
        key += 50;
    }

    {
        LMDBWriteTransaction::Ptr transaction = store.create_write_transaction();
        for (size_t i = 0; i < num_keys; i++) {
            std::vector<uint8_t> value;
            write(value, values[i]);
            transaction->put_value(keys[i], value);
        }

        // Now put keys in the db that are smaller and larger in length and smaller and larger in value
        // First key is at least 100
        uint64_t lower64 = static_cast<uint64_t>(keys[0]) - 50;
        uint64_t higher64 = static_cast<uint64_t>(keys[4]) + 50;
        uint256_t lower256 = uint256_t::from_uint128(keys[0]) - 50;
        uint256_t higher256 = uint256_t::from_uint128(keys[4]) + 50;
        uint32_t value_to_write = 6;
        std::vector<uint8_t> value;
        write(value, value_to_write);
        transaction->put_value(lower64, value);
        transaction->put_value(higher64, value);
        transaction->put_value(lower256, value);
        transaction->put_value(higher256, value);
        transaction->commit();
    }

    {
        // Values are at keys keys[0] -> keys[4]

        // First look for the value at each key, should return the exact keys
        for (uint32_t i = 0; i < num_keys; i++) {
            LMDBReadTransaction::Ptr transaction = store.create_read_transaction();
            std::vector<uint8_t> data;
            uint128_t key_copy = keys[i];
            bool success = transaction->get_value_or_previous(key_copy, data);
            EXPECT_EQ(success, true);
            bb::fr value = from_buffer<uint32_t>(data, 0);
            EXPECT_EQ(value, values[i]);
            EXPECT_EQ(key_copy, keys[i]);
        }

        // Now look for the value at key <= key[1] + 5 (does not exist), should be at keys[1]
        {
            LMDBReadTransaction::Ptr transaction = store.create_read_transaction();
            std::vector<uint8_t> data;
            uint128_t key = keys[1] + 5;
            bool success = transaction->get_value_or_previous(key, data);
            EXPECT_EQ(success, true);
            bb::fr value = from_buffer<uint32_t>(data, 0);
            EXPECT_EQ(value, values[1]);
            EXPECT_EQ(key, keys[1]);
        }

        // Now look for the value at key <= keys[4] + 5 (beyond the range of the current set of keys), should be at
        // keys[4]

        {
            LMDBReadTransaction::Ptr transaction = store.create_read_transaction();
            std::vector<uint8_t> data;
            uint128_t key = keys[4] + 5;
            bool success = transaction->get_value_or_previous(key, data);
            EXPECT_EQ(success, true);
            bb::fr value = from_buffer<uint32_t>(data, 0);
            EXPECT_EQ(value, values[4]);
            EXPECT_EQ(key, keys[4]);
        }

        // Now look for the value at key <= keys[0] - 5 (does not exist, less than first key), should not exist
        {
            LMDBReadTransaction::Ptr transaction = store.create_read_transaction();
            std::vector<uint8_t> data;
            uint128_t key = keys[0] - 5;
            bool success = transaction->get_value_or_previous(key, data);
            EXPECT_EQ(success, false);
        }
    }
}

TEST_F(LMDBStoreTest, can_not_retrieve_previous_key_from_empty_db)
{
    LMDBStore store(*_environment, "note hash tree", false, false);
    LMDBReadTransaction::Ptr transaction = store.create_read_transaction();
    uint128_t key = 20;
    std::vector<uint8_t> data;
    bool success = transaction->get_value(key, data);
    EXPECT_EQ(success, false);
}

TEST_F(LMDBStoreTest, can_write_and_read_at_random_keys)
{
    LMDBStore store(*_environment, "note hash tree");

    std::vector<size_t> keys;

    {
        LMDBWriteTransaction::Ptr transaction = store.create_write_transaction();

        for (size_t i = 0; i < SAMPLE_DATA_SIZE; i++) {
            std::vector<uint8_t> buf;
            write(buf, VALUES[i]);
            size_t key = static_cast<size_t>(rand() % 10000000);
            keys.push_back(key);
            transaction->put_node(0, key, buf);
        }
        transaction->commit();
    }

    {
        LMDBReadTransaction::Ptr transaction = store.create_read_transaction();
        for (size_t i = 0; i < SAMPLE_DATA_SIZE; i++) {
            std::vector<uint8_t> buf2;
            bool success = transaction->get_node(0, keys[i], buf2);
            EXPECT_EQ(success, true);
            bb::fr value = from_buffer<bb::fr>(buf2, 0);
            EXPECT_EQ(value, VALUES[i]);
        }
    }
}

TEST_F(LMDBStoreTest, can_recreate_the_store_and_use_again)
{
    std::vector<size_t> keys;
    {
        LMDBStore store(*_environment, "note hash tree");

        LMDBWriteTransaction::Ptr transaction = store.create_write_transaction();

        for (size_t i = 0; i < SAMPLE_DATA_SIZE; i++) {
            std::vector<uint8_t> buf;
            write(buf, VALUES[i]);
            size_t key = static_cast<size_t>(rand() % 10000000);
            keys.push_back(key);
            transaction->put_node(0, key, buf);
        }
        transaction->commit();
    }

    {
        LMDBStore store(*_environment, "note hash tree");

        LMDBReadTransaction::Ptr transaction = store.create_read_transaction();
        for (size_t i = 0; i < SAMPLE_DATA_SIZE; i++) {
            std::vector<uint8_t> buf2;
            bool success = transaction->get_node(0, keys[i], buf2);
            EXPECT_EQ(success, true);
            bb::fr value = from_buffer<bb::fr>(buf2, 0);
            EXPECT_EQ(value, VALUES[i]);
        }
    }
}

void read_loop(LMDBStore& store, size_t key, std::atomic<size_t>& flag, bb::fr starting_value)
{
    bool seen = false;
    while (true) {
        LMDBReadTransaction::Ptr transaction = store.create_read_transaction();
        std::vector<uint8_t> buf;
        bool success = transaction->get_node(0, key, buf);
        EXPECT_EQ(success, true);
        bb::fr value = from_buffer<bb::fr>(buf, 0);
        if (value == starting_value && !seen) {
            // acknowledge that we have seen the old value
            flag--;
            seen = true;
        }
        if (value == starting_value + bb::fr(1)) {
            // exit now that we have seen the new value
            break;
        }
    }
}

TEST_F(LMDBStoreTest, can_read_from_multiple_threads)
{
    LMDBStore store(*_environment, "note hash tree");
    const int num_threads = 50;

    size_t key = static_cast<size_t>(rand() % 1000000);

    {
        // we write VALUES[0] to a slot
        LMDBWriteTransaction::Ptr transaction = store.create_write_transaction();
        std::vector<uint8_t> buf;
        write(buf, VALUES[0]);
        transaction->put_node(0, key, buf);
        transaction->commit();
    }

    {
        // we setup multiple threads to read the slot and check they shutdown when the value changes
        std::vector<std::thread> threads;
        std::atomic<size_t> flag = num_threads;
        for (size_t i = 0; i < num_threads; i++) {
            threads.emplace_back(read_loop, std::ref(store), key, std::ref(flag), VALUES[0]);
        }
        // wait until all threads have seen the old value
        while (flag != 0) {
            std::this_thread::sleep_for(std::chrono::milliseconds(10));
        }
        {
            // we write VALUES[0] + 1 to the slot
            LMDBWriteTransaction::Ptr transaction = store.create_write_transaction();
            std::vector<uint8_t> buf;
            write(buf, VALUES[0] + 1);
            transaction->put_node(0, key, buf);
            transaction->commit();
        }
        // now wait for all threads to exit having seen the new value
        for (size_t i = 0; i < num_threads; i++) {
            threads[i].join();
        }
    }
}

TEST_F(LMDBStoreTest, can_handle_different_key_spaces)
{
    LMDBStore store(*_environment, "DB1", false, false, integer_key_cmp);

    // create a set of keys
    std::vector<uint8_t> keys{ 10, 20, 30, 40, 50 };

    // create values for each key space
    std::vector<std::vector<uint32_t>> values{
        { 10, 20, 30, 40, 50 },
        { 100, 200, 300, 400, 500 },
        { 1000, 2000, 3000, 4000, 5000 },
        { 10000, 20000, 30000, 40000, 50000 },
    };

    auto write_values = [&]<typename T>(LMDBWriteTransaction& transaction, uint32_t values_index) {
        for (uint32_t j = 0; j < keys.size(); j++) {
            std::vector<uint8_t> data;
            write(data, values[values_index][j]);
            auto key = static_cast<T>(keys[j]);
            transaction.put_value(key, data);
        }
    };

    auto check_values = [&]<typename T>(LMDBReadTransaction& transaction, uint32_t values_index) {
        for (uint32_t j = 0; j < keys.size(); j++) {
            std::vector<uint8_t> data;
            auto key = static_cast<T>(keys[j]);
            transaction.get_value(key, data);
            auto retrieved_value = from_buffer<uint32_t>(data);
            ASSERT_EQ(retrieved_value, values[values_index][j]);
        }
    };

    {
        LMDBWriteTransaction::Ptr transaction = store.create_write_transaction();

        write_values.template operator()<NodeKeyType>(*transaction, 0);
        write_values.template operator()<LeafIndexKeyType>(*transaction, 1);
        write_values.template operator()<MetaKeyType>(*transaction, 2);
        write_values.template operator()<FrKeyType>(*transaction, 3);
        transaction->commit();
    }

    {
        LMDBReadTransaction::Ptr transaction = store.create_read_transaction();

        // we should be able to read values from different key spaces
        check_values.template operator()<NodeKeyType>(*transaction, 0);
        check_values.template operator()<LeafIndexKeyType>(*transaction, 1);
        check_values.template operator()<MetaKeyType>(*transaction, 2);
        check_values.template operator()<FrKeyType>(*transaction, 3);
    }
}

template <typename T> void TestSerialisation(const T& key, uint32_t expectedSize)
{
    std::vector<uint8_t> buf = serialise_key(key);
    // Should serialise to expected size
    EXPECT_EQ(expectedSize, buf.size());
    T newValue;
    deserialise_key(buf.data(), newValue);
    // Should be different objects
    EXPECT_NE(&newValue, &key);
    // Should have the same value
    EXPECT_EQ(newValue, key);
}

TEST_F(LMDBStoreTest, produces_correct_key_sizes)
{
    auto& random_engine = bb::numeric::get_randomness();
    {
        uint8_t value = random_engine.get_random_uint8();
        TestSerialisation(value, 1);
    }
    {
        uint64_t value = random_engine.get_random_uint64();
        TestSerialisation(value, 8);
    }
    {
        uint128_t value = random_engine.get_random_uint128();
        TestSerialisation(value, 16);
    }
    {
        uint256_t value = random_engine.get_random_uint256();
        TestSerialisation(value, 32);
    }
}